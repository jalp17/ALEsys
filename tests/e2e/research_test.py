#!/usr/bin/env python3
"""E2E tests for ALEsys Research module."""
import os
import sys
import json
import urllib.request
import urllib.error

BASE = os.environ.get("ALESYS_API_URL", "http://localhost:3000")
TOKEN = os.environ.get("ALESYS_TOKEN", "dev-token-change-in-production")

def req(method, path, data=None):
    url = BASE + path
    body = json.dumps(data).encode() if data else None
    r = urllib.request.Request(url, data=body, method=method)
    r.add_header("Authorization", f"Bearer {TOKEN}")
    r.add_header("Content-Type", "application/json")
    try:
        with urllib.request.urlopen(r) as resp:
            return resp.status, json.loads(resp.read())
    except urllib.error.HTTPError as e:
        return e.code, json.loads(e.read())

def test_health():
    status, data = req("GET", "/health")
    assert status == 200, f"health failed: {status}"
    print("PASS: health")

def test_create_project():
    status, data = req("POST", "/api/v1/research/projects", {"name": "E2E Test", "description": "automated"})
    assert status in (200, 201), f"create project failed: {status} {data}"
    pid = data["id"]
    print(f"PASS: create project {pid}")
    return pid

def test_list_projects():
    status, data = req("GET", "/api/v1/research/projects")
    assert status == 200, f"list projects failed: {status}"
    assert isinstance(data, list)
    print(f"PASS: list projects ({len(data)})")

def test_create_note(pid):
    status, data = req("POST", "/api/v1/research/notes", {
        "project_id": pid, "title": "E2E Note", "content": "test content", "note_type": "idea", "tags": ["e2e"]
    })
    assert status in (200, 201), f"create note failed: {status} {data}"
    print(f"PASS: create note {data['id']}")
    return data["id"]

def test_list_notes(pid):
    status, data = req("GET", f"/api/v1/research/notes?project_id={pid}")
    assert status == 200, f"list notes failed: {status}"
    assert len(data) >= 1
    print(f"PASS: list notes ({len(data)})")

def test_create_run(pid):
    status, data = req("POST", "/api/v1/research/runs", {"project_id": pid, "name": "E2E Run", "protocol": "test protocol"})
    assert status in (200, 201), f"create run failed: {status} {data}"
    print(f"PASS: create run {data['id']}")
    return data["id"]

def test_list_inventory():
    status, data = req("GET", "/api/v1/inventory")
    assert status == 200, f"list inventory failed: {status}"
    print(f"PASS: list inventory ({len(data)})")

def test_metrics():
    status, _ = req("GET", "/metrics")
    assert status == 200, f"metrics failed: {status}"
    print("PASS: metrics")

def main():
    try:
        test_health()
        pid = test_create_project()
        test_list_projects()
        nid = test_create_note(pid)
        test_list_notes(pid)
        rid = test_create_run(pid)
        test_list_inventory()
        test_metrics()
        print("\nALL E2E TESTS PASSED")
        return 0
    except AssertionError as e:
        print(f"\nFAIL: {e}")
        return 1
    except Exception as e:
        print(f"\nERROR: {e}")
        return 1

if __name__ == "__main__":
    sys.exit(main())
