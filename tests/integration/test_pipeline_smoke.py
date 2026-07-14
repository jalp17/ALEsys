"""Smoke tests end-to-end con PostgreSQL real en Docker."""

from __future__ import annotations

import os
from pathlib import Path

import pytest

from db_manager import DatabaseManager
from pipeline import Pipeline


REQUIRED_ENV = {
    "PGHOST": os.getenv("PGHOST", "localhost"),
    "PGPORT": os.getenv("PGPORT", "5432"),
    "PGDATABASE": os.getenv("PGDATABASE", "alesys_test"),
    "PGUSER": os.getenv("PGUSER", "alesys"),
    "PGPASSWORD": os.getenv("PGPASSWORD", "alesys"),
}


@pytest.fixture(scope="session")
def db_session() -> DatabaseManager:
    if not all(REQUIRED_ENV.values()):
        pytest.skip("PostgreSQL test env vars not set")
    manager = DatabaseManager()
    manager.initialize_tables()
    try:
        yield manager
    finally:
        manager.drop_tables()


def test_database_roundtrip(db_session: DatabaseManager, tmp_path: Path) -> None:
    doc_id = db_session.insert_document(
        filename="test.md",
        path=str(tmp_path / "test.md"),
        metadata={"size_bytes": 13},
    )
    fragment_id = db_session.insert_fragment(
        documento_id=doc_id, content="hola", embedding=[0.0] * 384, chunk_index=0
    )
    entity_id = db_session.insert_entity(
        fragmento_id=fragment_id, name="foo", type_="concepto"
    )
    relation_id = db_session.insert_relation(
        source_entity_id=entity_id,
        target_entity_id=entity_id,
        relation_type=" relaciona",
    )
    assert all([doc_id, fragment_id, entity_id, relation_id])


def test_batch_inserts(db_session: DatabaseManager, tmp_path: Path) -> None:
    doc_id = db_session.insert_document(
        filename="batch.md",
        path=str(tmp_path / "batch.md"),
        metadata={"size_bytes": 5},
    )
    fragments = [("hola", [0.0] * 384, 0), ("chau", [0.0] * 384, 1)]
    frag_ids = db_session.batch_insert_fragments(doc_id, fragments)
    assert len(frag_ids) == 2

    entities = [("foo", "concepto", {}), ("bar", "concepto", {})]
    ent_ids = db_session.batch_insert_entities(frag_ids[0], entities)
    assert len(ent_ids) == 2

    relations = [(ent_ids["foo"], ent_ids["bar"], "relaciona", {})]
    db_session.batch_insert_relations(relations)
