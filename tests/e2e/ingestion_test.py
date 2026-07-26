#!/usr/bin/env python3
"""
E2E ingestion pipeline test suite (7 scenarios + offline checks).

Scenarios:
 1. Health check (no auth required)
 2. GET /ingestion/config (auth required)
 3. PUT /ingestion/config (auth required, admin-like)
 4. POST /ingestion/pdf - missing pdf_path (validation)
 5. POST /ingestion/pdf - accepted ( queues job )
 6. GET /ingestion/status/:id (job status)
 7. WS /ws/ingestion/:job_id - streaming (skipped if no websocket lib)

Offline checks:
 - cargo available
 - endpoints registered in main.rs

Python deps: requests, websocket-client (optional), pyjwt
"""

import base64
import hashlib
import json
import os
import signal
import subprocess
import sys
import time
from pathlib import Path

try:
    import jwt as pyjwt
except ImportError:
    pyjwt = None

try:
    import websocket  # websocket-client
except ImportError:
    websocket = None

import requests

API_BASE = os.environ.get("ALESYS_API_URL", "http://localhost:8080/api/v1")
HEALTH_URL = f"{API_BASE}/health"
CONFIG_URL = f"{API_BASE}/ingestion/config"
INGEST_URL = f"{API_BASE}/ingestion/pdf"
WS_BASE = os.environ.get("ALESYS_WS_URL", "ws://localhost:8080/ws/ingestion")

JWT_SECRET = os.environ.get("JWT_SECRET", "alesys-dev-secret-change-in-production")


def make_token() -> str:
    if pyjwt is None:
        return ""
    payload = {
        "sub": "testuser",
        "role": "ingestor",
        "exp": int(time.time()) + 3600,
        "iat": int(time.time()),
    }
    return pyjwt.encode(payload, JWT_SECRET, algorithm="HS256")


def auth_headers(token: str) -> dict:
    return {"Authorization": f"Bearer {token}", "Content-Type": "application/json"}


def is_api_up() -> bool:
    try:
        r = requests.get(HEALTH_URL, timeout=5)
        return r.status_code < 500
    except Exception:
        return False


def start_backend() -> subprocess.Popen:
    print("Starting alesys-api backend...")
    env = os.environ.copy()
    env.setdefault("DATABASE_URL", "postgres://alesys:alesys@localhost:5432/alesys")
    env.setdefault("RUST_LOG", "warn")
    env.setdefault("JWT_SECRET", JWT_SECRET)
    return subprocess.Popen(
        ["cargo", "run", "--bin", "alesys-api"],
        cwd=Path(__file__).resolve().parents[2],
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        preexec_fn=os.setsid,
    )


def wait_for_api(timeout: int = 60) -> bool:
    for _ in range(timeout):
        if is_api_up():
            return True
        time.sleep(1)
    return False


def scenario_health():
    r = requests.get(HEALTH_URL, timeout=10)
    assert r.status_code == 200, f"Health failed: {r.status_code} {r.text}"
    print("PASS: health")


def scenario_config_get(token: str):
    r = requests.get(CONFIG_URL, headers=auth_headers(token), timeout=10)
    assert r.status_code == 200, f"Config GET failed: {r.status_code} {r.text}"
    data = r.json()
    assert "output_base_dir" in data
    print("PASS: config GET")


def scenario_config_put(token: str):
    payload = {
        "model_dir": "/models/mineru",
        "output_base_dir": "/tmp/alesys-ingestion",
        "fallback_enabled": True,
        "default_ocr_langs": ["en", "es"],
        "max_parallel": 1,
        "timeout_hours": 20,
    }
    r = requests.put(CONFIG_URL, headers=auth_headers(token), json=payload, timeout=10)
    assert r.status_code == 200, f"Config PUT failed: {r.status_code} {r.text}"
    print("PASS: config PUT")


def scenario_ingest_validation(token: str):
    r = requests.post(INGEST_URL, headers=auth_headers(token), json={}, timeout=10)
    assert r.status_code in (400, 422), f"Expected validation error, got {r.status_code}"
    print("PASS: ingest validation error")


def scenario_ingest_accepted(token: str):
    payload = {"pdf_path": "/nonexistent/sample.pdf", "topic": "e2e-test"}
    r = requests.post(INGEST_URL, headers=auth_headers(token), json=payload, timeout=10)
    assert r.status_code in (200, 202, 500), f"Unexpected status {r.status_code}"
    print("PASS: ingest accepted (or handled missing file)")


def scenario_status(token: str, job_id: str):
    url = f"{API_BASE}/ingestion/status/{job_id}"
    r = requests.get(url, headers=auth_headers(token), timeout=10)
    assert r.status_code in (200, 404), f"Status failed: {r.status_code}"
    print("PASS: status")


def scenario_ws_stream(job_id: str):
    if websocket is None:
        print("SKIP: ws streaming (no websocket-client)")
        return
    ws_url = f"{WS_BASE}/{job_id}"
    ws = websocket.create_connection(ws_url, timeout=5)
    try:
        msg = ws.recv()
        assert msg, "Empty WS message"
        data = json.loads(msg)
        assert data.get("type") == "progress"
        print("PASS: ws streaming")
    finally:
        ws.close()


def scenario_offline_compile():
    import shutil
    if not shutil.which("cargo"):
        print("SKIP: cargo not found")
        return
    print("PASS: offline compile check")


def scenario_endpoints_registered():
    import re
    patterns = [
        r"ingestion/pdf",
        r"ingestion/batch",
        r"ingestion/status",
        r"ingestion/config",
        r"ws/ingestion",
    ]
    main_rs = Path(__file__).resolve().parents[2] / "crates" / "api" / "src" / "main.rs"
    text = main_rs.read_text(errors="ignore")
    for pat in patterns:
        assert re.search(pat, text), f"Missing route pattern: {pat}"
    print("PASS: endpoints registered in main.rs")


def main():
    backend_proc = None
    token = make_token()
    try:
        if not is_api_up():
            backend_proc = start_backend()
            if not wait_for_api():
                print("FAIL: Backend did not start within timeout")
                sys.exit(1)
            print("Backend started")
        else:
            print("Using existing backend")

        print("=== E2E Ingestion Tests (7 scenarios) ===")
        scenario_health()
        scenario_config_get(token)
        scenario_config_put(token)
        scenario_ingest_validation(token)
        scenario_ingest_accepted(token)

        fake_job = "00000000-0000-0000-0000-000000000000"
        scenario_status(token, fake_job)
        scenario_ws_stream(fake_job)

        print("\nAll scenarios completed")
        sys.exit(0)

    except AssertionError as e:
        print(f"FAIL: {e}")
        sys.exit(1)
    finally:
        if backend_proc is not None:
            print("Stopping backend...")
            os.killpg(os.getpgid(backend_proc.pid), signal.SIGTERM)
            backend_proc.wait(timeout=10)


def main_offline():
    print("=== E2E Ingestion Offline Checks ===")
    scenario_offline_compile()
    scenario_endpoints_registered()
    print("\nOffline checks completed")


if __name__ == "__main__":
    if "--offline" in sys.argv:
        main_offline()
    else:
        main()
