# Changelog

Todos los cambios notables en este proyecto se documentan en este archivo.

El formato está basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.0.0/),
y este proyecto se adhiere a [Semantic Versioning](https://semver.org/lang/es/).

## [v1.0.0] - 2026-06-12

### Added
- **GraphRAG-PG Pipeline**: Ingesta híbrida Markdown → PostgreSQL (pgvector + grafo de conocimiento)
- **Módulos Core**:
  - `config.py`: Configuración centralizada (DB, OpenRouter, Embeddings, Paths, Chunking)
  - `db_manager.py`: Capa de acceso a PostgreSQL con pgvector + tablas relacionales de grafo
  - `embedder.py`: Embeddings locales CPU con sentence-transformers (384 dimensiones)
  - `extractor.py`: Extracción de entidades/relaciones vía OpenRouter (Gemini 2.5 Flash)
  - `pipeline.py`: Orquestador completo con deduplicación, dry-run, retry automático
  - `test_queries.py`: Consultas vectoriales, grafo, híbridas y RAG (`ask`)
- **CLI** (`main.py`): Comandos `db-init`, `db-drop`, `run`, `query`, `ask`, `list`
- **GUI Tkinter** (`gui.py`): 3 pestañas (Pipeline, Search, Chat) con threading
- **ChatAgent** (`core/chat_agent.py`): Contexto RAG híbrido (vector + grafo) + historial
- **Búsqueda Web** (`core/web_search.py`): DuckDuckGo para contexto adicional
- **Tests automatizados** (`run_tests.sh`): Sintaxis, imports, DB, embeddings, pipeline, consultas, deduplicación
- **Docker**: Multi-stage (builder + runtime), non-root user, healthcheck
- **Docker Compose**: PostgreSQL pgvector + ALEsys para desarrollo local
- **GitHub Actions CI/CD**:
  - `ci.yml`: Lint, tests con PostgreSQL service, dependency review
  - `docker.yml`: Build multi-arch (amd64/arm64), push a ghcr.io, firma cosign
  - `security.yml`: CodeQL, Trivy, Gitleaks, TruffleHog, pip-audit

### Security
- Fix SQL injection en queries vectoriales (parámetros psycopg en lugar de f-strings)
- Validación temprana de `OPENROUTER_API_KEY` en pipeline (fail-fast con mensaje claro)
- Timeout en conexiones PostgreSQL (5 segundos)
- Manejo de señales (SIGINT/SIGTERM) para cleanup ordenado de recursos
- Secret scanning con TruffleHog (PR diff + full history), Gitleaks
- Dependency review en PRs con políticas de licencias y severidad
- `.env` excluido de git, soporte para python-dotenv

### Fixed
- **P1**: Extractor - assert → safe-check + método `answer()` para RAG
- **P2**: Config - `PGPORT` con `_env_int()` para parsing seguro
- **P3**: Pipeline - deduplicación (ON CONFLICT), retry DB (3 intentos), modo dry-run, `graph_search` LIMIT
- Módulos legacy migrados: `main.py`, `gui.py`, `core/chat_agent.py`
- `core/memory_manager.py` y `core/indexer.py` deprecados (mantenidos como referencia)

### Changed
- Dependencias actualizadas a últimas versiones estables:
  - `psycopg[binary]~=3.3.4` (was 3.1.0)
  - `sentence-transformers~=5.5.1` (was 2.2.0)
  - `httpx~=0.28.1` (was 0.27.0)
  - `ddgs~=9.14.4` (was 6.0.0)
  - `rich~=15.0.0` (was 13.0.0)
- Arquitectura migrada de RAG multi-proyecto (FAISS + llama.cpp) a GraphRAG-PG (PostgreSQL + pgvector + OpenRouter)

### Removed
- `tests/test_project/` (calculadora de ejemplo legacy)
- Dependencias legacy: `faiss-cpu`, `torch`, `llama-cpp-python`, `psutil`, `watchdog`
- Ollama como proxy de embeddings (ahora sentence-transformers directo en CPU)

---

## [Unreleased]

### Planned (Fase 6+)
- Batch inserts con `psycopg.execute_batch()` para performance
- Connection pooling con `psycopg.pool`
- Cacheo de embeddings en disco
- Validación de chunk size contra límites de modelo (8000 tokens)
- Threading en GUI para operaciones bloqueantes
- Unit tests con pytest + mocking OpenRouter
- Integration tests con pytest-docker
- Documentación completa (instalación, tutorial, API)
- Empaquetado PyPI (`pip install alesys`)
- Modo incremental (solo archivos modificados)
- API REST con FastAPI
- Export/Import de base de datos
- Notificaciones por webhook (Discord/Slack)