"""Pruebas unitarias para embedder."""

from __future__ import annotations

from pathlib import Path
from types import SimpleNamespace

import pytest

from embedder import _cache_get, _cache_key, _cache_set, _prune_cache_dir


@pytest.fixture
def tmp_cache(tmp_path: Path) -> Path:
    return tmp_path / "cache"


def test_cache_key_is_deterministic() -> None:
    assert _cache_key("texto") == _cache_key("texto")
    assert _cache_key("texto") != _cache_key("otro texto")


def test_cache_set_and_get_roundtrip(tmp_cache: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr("embedder.CACHE_DIR", tmp_cache)
    key = _cache_key("dummy")
    _cache_set(key, "model", [0.1, 0.2, 0.3])
    assert _cache_get(key, "model") == [0.1, 0.2, 0.3]


def test_cache_get_missing_returns_none(tmp_cache: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr("embedder.CACHE_DIR", tmp_cache)
    assert _cache_get("missing", "model") is None


def test_prune_cache_dir_removes_oldest(tmp_path: Path) -> None:
    model_dir = tmp_path / "model"
    model_dir.mkdir()
    old = model_dir / "old.json"
    old.write_text("[0.0]", encoding="utf-8")
    new = model_dir / "new.json"
    new.write_text("[0.1]", encoding="utf-8")
    max_bytes = 1
    _prune_cache_dir(model_dir)
    assert not old.exists()
    assert new.exists()


def test_prune_cache_dir_handles_missing_dir(tmp_path: Path) -> None:
    _prune_cache_dir(tmp_path / "missing")
