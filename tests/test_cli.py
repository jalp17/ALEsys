"""Pruebas de CLI para ALEsys usando click."""

from __future__ import annotations

from click.testing import CliRunner
from _pytest.monkeypatch import MonkeyPatch

from cli import cli


@pytest.fixture
def runner() -> CliRunner:
    return CliRunner()


@pytest.fixture
def mocked_env(monkeypatch: MonkeyPatch) -> None:
    monkeypatch.setenv("PGHOST", "127.0.0.1")
    monkeypatch.setenv("PGPORT", "5433")
    monkeypatch.setenv("PGDATABASE", "alesys_test")
    monkeypatch.setenv("PGUSER", "alesys")
    monkeypatch.setenv("PGPASSWORD", "alesys")
    monkeypatch.setenv("OPENROUTER_API_KEY", "test-key")
    monkeypatch.setenv("OPENROUTER_MODEL", "test-model")
    monkeypatch.setenv("EMBEDDING_BACKEND", "sentence-transformers")
    monkeypatch.setenv("EMBEDDING_MODEL", "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2")
    monkeypatch.setenv("BOOKS_DIR", "/tmp/alesys_books_nonexistent")


def test_db_init(runner: CliRunner, mocked_env: None) -> None:
    result = runner.invoke(cli, ["db-init"])
    assert result.exit_code == 0
    assert "Tablas inicializadas correctamente" in result.output


def test_db_drop(runner: CliRunner, mocked_env: None) -> None:
    result = runner.invoke(cli, ["db-drop", "--confirm"])
    assert result.exit_code == 0
    assert "Tablas eliminadas" in result.output


def test_db_drop_requires_confirmation(runner: CliRunner, mocked_env: None) -> None:
    result = runner.invoke(cli, ["db-drop"])
    assert result.exit_code != 0
