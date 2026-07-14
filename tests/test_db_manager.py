"""Pruebas unitarias para DatabaseManager."""

from __future__ import annotations

from unittest.mock import MagicMock

import pytest

from db_manager import DatabaseManager


@pytest.fixture
def db() -> DatabaseManager:
    manager = DatabaseManager.__new__(DatabaseManager)
    manager._conn = MagicMock()
    manager._conn.closed = False
    return manager


def test_connect_reuses_open_connection(db: DatabaseManager) -> None:
    db._conn = MagicMock()
    db._conn.closed = False
    assert db.connect() is db._conn


def test_connect_retries_on_operational_error(db: DatabaseManager, monkeypatch: pytest.MonkeyPatch) -> None:
    import psycopg

    connection_mock = MagicMock()
    connection_mock.closed = False
    monkeypatch.setattr(psycopg, "connect", lambda *_, **__: (_ for _ in ()).throw(psycopg.OperationalError("fail")))

    with pytest.raises(ConnectionError):
        db.connect()


def test_close_does_nothing_when_already_closed(db: DatabaseManager) -> None:
    closed_conn = MagicMock()
    closed_conn.closed = True
    db._conn = closed_conn
    db.close()
    assert db._conn is closed_conn


def test_context_manager_calls_connect_and_close(db: DatabaseManager, monkeypatch: pytest.MonkeyPatch) -> None:
    db._conn = MagicMock()
    db._conn.closed = False
    connect_calls: list[str] = []
    close_calls: list[str] = []

    def fake_connect() -> MagicMock:
        connect_calls.append("connect")
        return db._conn

    def fake_close() -> None:
        close_calls.append("close")

    monkeypatch.setattr(db, "connect", fake_connect)
    monkeypatch.setattr(db, "close", fake_close)

    with db as manager:
        assert manager is db
        assert connect_calls == ["connect"]
    assert close_calls == ["close"]
