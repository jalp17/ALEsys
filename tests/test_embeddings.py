"""Pruebas unitarias para embedder."""

from __future__ import annotations

import json
from pathlib import Path
from types import SimpleNamespace

import pytest

from embedder import _cache_key, _cache_set


@pytest.fixture
def fake_config(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> SimpleNamespace:
    fake = SimpleNamespace(
        EMBEDDING=SimpleNamespace(model_name="test/model"),
        CACHE_DIR=tmp_path / "cache",
    )
    monkeypatch.setattr("embedder.CACHE_DIR", fake.CACHE_DIR)
    return fake


def test_cache_key_is_deterministic() -> None:
    assert _cache_key("abc") == _cache_key("abc")
    assert _cache_key("abc") != _cache_key("def")


def test_roundtrip_writes_and_reads_same_vector(fake_config: SimpleNamespace) -> None:
    vector = [0.1, 0.2, 0.3, 0.4]
    key = _cache_key("texto")
    _cache_set(key, fake_config.EMBEDDING.model_name, vector)

    model_dir = fake_config.CACHE_DIR / fake_config.EMBEDDING.model_name.replace("/", "_")
    cache_path = model_dir / f"{key}.json"
    assert cache_path.exists()

    loaded = json.loads(cache_path.read_text(encoding="utf-8"))
    assert loaded == vector
