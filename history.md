# Historial de progreso — ALEsys / GraphRAG-PG

## 2026-06-09 — Refactorización del proyecto

- Se actualizó `README.md` para reflejar la nueva arquitectura GraphRAG-PG con PostgreSQL.
- Se confirmó el cambio en `AGENT.md` (sección NOTAS).
- Se creó `history.md` para registrar el progreso.
- Implementado `config.py` con dataclasses inmutables para DB, OpenRouter, embeddings, rutas y chunking.
- Implementado `db_manager.py` con psycopg v3: clase `DatabaseManager` con connection pooling, inicialización de tablas (`documentos`, `fragmentos`, `entidades`, `relaciones`), inserción con `ON CONFLICT`, y type hints.
- Actualizado `requirements.txt` con psycopg>=3.1.0, httpx, eliminando dependencias legacy.
- Implementado `embedder.py` con clase `Embedder` que carga `sentence-transformers/all-MiniLM-L6-v2` en CPU y genera vectores de 384 dimensiones con validación.
- Implementado `pipeline.py` con clase `Pipeline`: escaneo recursivo de `.md`, chunking inteligente (respeta saltos de línea), embedding batch, persistencia en PostgreSQL con barra de progreso Rich.
- Implementado `extractor.py` con clase `Extractor`: conexión a OpenRouter, prompt estructurado para extraer entidades/relaciones, manejo de errores con reintentos, parsing robusto de JSON (strip de markdown).
- Integrado extractor en `pipeline.py`: tras insertar fragmentos, extrae entidades/relaciones por chunk y persiste en tablas de grafo.

## 2026-06-09 — Fase 4: Consultas híbridas y migración de módulos legacy

- Creado `test_queries.py` con funciones:
  - `vector_search()` — búsqueda por similitud coseno en pgvector
  - `graph_search()` — navegación de entidades y relaciones
  - `hybrid_search()` — combina búsqueda vectorial con contexto de grafo
  - `ask()` — pregunta/respuesta con contexto RAG vía OpenRouter
- Migrado `main.py` a CLI de GraphRAG-PG con comandos: `db-init`, `db-drop`, `run`, `query`, `ask`, `list`
- Migrado `gui.py` a GUI de GraphRAG-PG: ejecución de pipeline, búsqueda híbrida, listado de documentos, logs
- Migrado `core/chat_agent.py` a PostgreSQL RAG: usa `DatabaseManager` + `Embedder` + pgvector en lugar de FAISS
- Deprecados `core/memory_manager.py` y `core/indexer.py` con avisos de reemplazo
- Actualizado `core/__init__.py` con documentación de módulos activos/deprecados

## 2026-06-09 — Corrección de inconsistencias y validación

- Añadido `ddgs` a `requirements.txt` (dependencia faltante de `core/web_search.py`)
- Corregidas rutas en `AGENT.md` §3 (`graph_rag_project/` → `ALEsys/`)
- Eliminados directorios legacy: `projects/`, `tests/test_project/`
- Creado `run_tests.sh` — script de validación automatizada
- Creado `estado_servidor.md` — resumen técnico para Obsidian

## 2026-06-09 — Corrección de bugs (P1-P3)

### P1 — Críticos
- **extractor.py**: Reemplazado `assert isinstance(result, dict)` por `if not isinstance` con retorno seguro de dict vacío. Añadido `AssertionError` a excepciones capturadas. Refactorizado `_call()` compartido y nuevo método `answer()` para Q&A real.
- **config.py**: Creado helper `_env_int()` para que `PGPORT` no crashee con valores no numéricos.
- **test_queries.py + core/chat_agent.py**: `ask()` ahora usa `Extractor.answer()` con prompt de Q&A, no el de extracción de entidades.

### P2 — Altos
- **pipeline.py**: `_process_file` ahora elimina fragmentos previos antes de re-indexar (sin duplicados). Añadido `dry_run` para previsualizar. Chunking protegido contra `overlap >= size`.
- **db_manager.py**: Conexión con 3 reintentos y timeout. Nuevo método `delete_fragments_by_document()`.
- **test_queries.py**: `graph_search()` ahora acepta `limit=` y lo pasa a SQL.
- **main.py**: `run` acepta `--dry-run`. `query --graph` usa `--top-k` como límite.
- **run_tests.sh**: Usa BD `alesys_test`, cleanup automático en `EXIT`, prueba de duplicados + dry-run.

### P3 — Medios
- **core/chat_agent.py**: Historial de conversación inyectado en el contexto del prompt.
- **embedder.py**: Nuevo método `unload()` para liberar el modelo de memoria.
- **db_manager.py**: Conexión por kwargs en vez de DSN string (seguro para passwords con caracteres especiales).
- **gui.py**: Error silencioso en `_do_refresh_docs` ahora se logea.
- **requirements.txt**: Pinning con `~=` y extras `[binary]` para psycopg.

---

## Drivers PostgreSQL — Comparativa

### psycopg2 (maduro, síncrono)
| Pro | Contra |
|-----|--------|
| + Más usado y probado en producción | - Síncrono (bloquea el event loop) |
| + Documentación y comunidad enormes | - Requiere compilar con libpq |
| + Compatible con SQLAlchemy, Alembic | - Sin soporte nativo de async/await |
| + `psycopg2-binary` evita compilación | - Conexiones bloqueantes en pipelines IO-bound |

### asyncpg (nativo asíncrono)
| Pro | Contra |
|-----|--------|
| + 100% asíncrono (asyncio) | - Menos ecosistema que psycopg2 |
| + 2-3x más rápido que psycopg2 | - No compatible con SQLAlchemy 1.x (sí con 2.x) |
| + No requiere libpq (puro Python+C) | - Curva de aprendizaje si vienes de psycopg2 |
| + Ideal para pipelines con IO (APIs, embeddings) | - Menos ejemplos en producción |

### Otras alternativas

| Driver | Tipo | Ideal para |
|--------|------|-----------|
| **psycopg** (v3, `psycopg`) | Síncrono + Async | Lo mejor de ambos: API unificada sync/async, rendimiento similar a asyncpg, soporte nativo de tipos PostgreSQL |
| **SQLAlchemy** (2.0) | ORM/ Core sobre psycopg2/asyncpg/psycopg | Si se prefiere abstracción ORM, migraciones con Alembic |
| **pg8000** | Síncrono puro Python | No requiere libpq, ideal para entornos sin compilador; más lento que psycopg2 |
| **aiopg** | Asíncrono sobre psycopg2 | Wrapper asyncio para psycopg2; mantenimiento menos activo |

### Recomendación tentativa
**psycopg** (v3) — porque ofrece API unificada sync/async, es mantenido activamente por la comunidad, tiene soporte nativo de `vector` y `jsonb`, y permite empezar síncrono en Fase 1 migrando a async después sin cambiar de librería.
