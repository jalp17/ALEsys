# Referencia de API

Módulos principales y puntos de entrada recomendados.

## Módulos

- **`config`**: carga automática de `.env` y constantes (`DB`, `OPENROUTER`, `EMBEDDING`, `PATHS`, `CHUNKING`).
- **`db_manager.DatabaseManager`**: conexión, tablas, inserts simples y batch.
- **`embedder`**: `get_embedder()` y funciones de cache (`_cache_key`, `_cache_set`).
- **`extractor.Extractor`**: `extract()` y `answer()` contra OpenRouter.
- **`pipeline.Pipeline`**: orquestación de escaneo, chunking, embeddings y grafo.
- **`test_queries`**: `vector_search`, `graph_search`, `hybrid_search`, `ask`.

## CLI

Comandos expuestos en `cli.py`:

- `python -m cli db-init`
- `python -m cli db-drop --confirm`
- `python -m cli run`
- `python -m cli query "texto"`
- `python -m cli ask "pregunta"`

## Contratos esperados

- Entradas y salidas se basan en tipos JSON y parámetros psycopg.
- La extracción tolera fallos: ante JSON inválido o error HTTP devuelve `{"entidades": [], "relaciones": []}`.
