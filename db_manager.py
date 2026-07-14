import json
import logging
import time
from typing import Any, Optional

import psycopg
from psycopg import Connection, Cursor
from psycopg.rows import dict_row

from config import DB, EMBEDDING

logger = logging.getLogger(__name__)


class DatabaseManager:
    def __init__(self):
        self._conn: Optional[Connection] = None

    def _conninfo(self) -> dict[str, Any]:
        return {
            "host": DB.host,
            "port": DB.port,
            "dbname": DB.dbname,
            "user": DB.user,
            "password": DB.password,
            "connect_timeout": 5,
        }

    def connect(self) -> Connection:
        if self._conn is not None and not self._conn.closed:
            return self._conn
        last_error = None
        for attempt in range(3):
            try:
                self._conn = psycopg.connect(row_factory=dict_row, **self._conninfo())
                self._conn.autocommit = True
                logger.info("Conectado a PostgreSQL %s:%s/%s", DB.host, DB.port, DB.dbname)
                return self._conn
            except psycopg.OperationalError as e:
                last_error = e
                logger.warning("Conexión fallida (intento %d/3): %s", attempt + 1, e)
                if attempt < 2:
                    time.sleep(1)
        raise ConnectionError(f"No se pudo conectar a PostgreSQL tras 3 intentos: {last_error}")

    @property
    def conn(self) -> Connection:
        return self.connect()

    @property
    def cursor(self) -> Cursor:
        return self.conn.cursor()

    def close(self) -> None:
        if self._conn is not None and not self._conn.closed:
            self._conn.close()
            logger.info("Conexión PostgreSQL cerrada")

    def __enter__(self) -> "DatabaseManager":
        self.connect()
        return self

    def __exit__(self, *args: Any) -> None:
        self.close()

    def initialize_tables(self) -> None:
        cur = self.cursor
        cur.execute("CREATE EXTENSION IF NOT EXISTS vector")
        cur.execute(
            """
            CREATE TABLE IF NOT EXISTS documentos (
                id          SERIAL PRIMARY KEY,
                filename    TEXT NOT NULL UNIQUE,
                path        TEXT NOT NULL,
                metadata    JSONB DEFAULT '{}',
                created_at  TIMESTAMPTZ DEFAULT NOW()
            )
            """
        )
        cur.execute(
            f"""
            CREATE TABLE IF NOT EXISTS fragmentos (
                id            SERIAL PRIMARY KEY,
                documento_id  INTEGER NOT NULL REFERENCES documentos(id) ON DELETE CASCADE,
                content       TEXT NOT NULL,
                embedding     VECTOR({EMBEDDING.dimension}),
                chunk_index   INTEGER NOT NULL,
                created_at    TIMESTAMPTZ DEFAULT NOW()
            )
            """
        )
        cur.execute(
            """
            CREATE TABLE IF NOT EXISTS entidades (
                id            SERIAL PRIMARY KEY,
                fragmento_id  INTEGER NOT NULL REFERENCES fragmentos(id) ON DELETE CASCADE,
                name          TEXT NOT NULL,
                type          TEXT NOT NULL,
                metadata      JSONB DEFAULT '{}',
                created_at    TIMESTAMPTZ DEFAULT NOW()
            )
            """
        )
        cur.execute(
            """
            CREATE TABLE IF NOT EXISTS relaciones (
                id                SERIAL PRIMARY KEY,
                source_entity_id  INTEGER NOT NULL REFERENCES entidades(id) ON DELETE CASCADE,
                target_entity_id  INTEGER NOT NULL REFERENCES entidades(id) ON DELETE CASCADE,
                relation_type     TEXT NOT NULL,
                metadata          JSONB DEFAULT '{}',
                created_at        TIMESTAMPTZ DEFAULT NOW()
            )
            """
        )
        logger.info("Tablas inicializadas correctamente")

    def insert_document(self, filename: str, path: str, metadata: Optional[dict] = None) -> int:
        cur = self.cursor
        metadata_json = json.dumps(metadata or {})
        cur.execute(
            """
            INSERT INTO documentos (filename, path, metadata)
            VALUES (%s, %s, %s::jsonb)
            ON CONFLICT (filename) DO UPDATE SET
                path      = EXCLUDED.path,
                metadata  = EXCLUDED.metadata
            RETURNING id
            """,
            (filename, path, metadata_json),
        )
        row = cur.fetchone()
        return row["id"]

    def insert_fragment(
        self,
        documento_id: int,
        content: str,
        embedding: list[float],
        chunk_index: int,
    ) -> int:
        cur = self.cursor
        cur.execute(
            """
            INSERT INTO fragmentos (documento_id, content, embedding, chunk_index)
            VALUES (%s, %s, %s, %s)
            RETURNING id
            """,
            (documento_id, content, embedding, chunk_index),
        )
        row = cur.fetchone()
        return row["id"]

    def insert_entity(
        self,
        fragmento_id: int,
        name: str,
        type_: str,
        metadata: Optional[dict] = None,
    ) -> int:
        cur = self.cursor
        metadata_json = json.dumps(metadata or {})
        cur.execute(
            """
            INSERT INTO entidades (fragmento_id, name, type, metadata)
            VALUES (%s, %s, %s, %s::jsonb)
            RETURNING id
            """,
            (fragmento_id, name, type_, metadata_json),
        )
        row = cur.fetchone()
        return row["id"]

    def insert_relation(
        self,
        source_entity_id: int,
        target_entity_id: int,
        relation_type: str,
        metadata: Optional[dict] = None,
    ) -> int:
        cur = self.cursor
        metadata_json = json.dumps(metadata or {})
        cur.execute(
            """
            INSERT INTO relaciones (source_entity_id, target_entity_id, relation_type, metadata)
            VALUES (%s, %s, %s, %s::jsonb)
            RETURNING id
            """,
            (source_entity_id, target_entity_id, relation_type, metadata_json),
        )
        row = cur.fetchone()
        return row["id"]

    def delete_fragments_by_document(self, documento_id: int) -> None:
        cur = self.cursor
        cur.execute(
            "DELETE FROM fragmentos WHERE documento_id = %s", (documento_id,)
        )

    def drop_tables(self) -> None:
        cur = self.cursor
        cur.execute("DROP TABLE IF EXISTS relaciones CASCADE")
        cur.execute("DROP TABLE IF EXISTS entidades CASCADE")
        cur.execute("DROP TABLE IF EXISTS fragmentos CASCADE")
        cur.execute("DROP TABLE IF EXISTS documentos CASCADE")
        logger.info("Tablas eliminadas")
