"""Pruebas unitarias para búsquedas."""

from __future__ import annotations

from unittest.mock import MagicMock

import pytest

from db_manager import DatabaseManager
from test_queries import graph_search, hybrid_search, vector_search


@pytest.fixture
def db() -> DatabaseManager:
    manager = DatabaseManager.__new__(DatabaseManager)
    manager._conn = MagicMock()
    manager._conn.closed = False
    manager._conn.cursor.return_value.__enter__ = lambda self: self
    manager._conn.cursor.return_value.__exit__ = lambda self, *_: None
    return manager


def test_vector_search_returns_expected_shape(db: DatabaseManager, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr("test_queries.DatabaseManager", lambda: db)
    results = vector_search("campo eléctrico", top_k=2)
    assert isinstance(results, list)
    assert all("filename" in item for item in results)


def test_graph_search_returns_entities_and_relations(db: DatabaseManager, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr("test_queries.DatabaseManager", lambda: db)
    results = graph_search("Faraday", limit=5)
    assert "entities" in results
    assert "relations" in results


def test_hybrid_search_includes_entities(db: DatabaseManager, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr("test_queries.DatabaseManager", lambda: db)
    results = hybrid_search("campo eléctrico", top_k=2)
    assert isinstance(results, list)
    assert all("entities" in item for item in results)
