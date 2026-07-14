# 📊 ALEsys / GraphRAG-PG — Estado del Proyecto

**Fecha:** 2026-06-09
**Servidor:** Fedora Server 43 (AMD Ryzen 5, 16GB RAM)
**Base de Datos:** PostgreSQL + pgvector (Docker `postgres_db:5432`)
**Repositorio:** `/home/jesus/knowledge_database/desarrollo_git/ALEsys/`

---

## ✅ Fases completadas (1-4)

### Fase 1 — Conexión y Base de Datos
- `config.py`: Dataclasses inmutables para DB, OpenRouter, embeddings, rutas y chunking
- `db_manager.py`: Clase `DatabaseManager` con psycopg v3, 4 tablas relacionales + pgvector
- Driver seleccionado: **psycopg v3** (sync/async unificado)

### Fase 2 — Escaneo y Vectores Locales
- `embedder.py`: `sentence-transformers/all-MiniLM-L6-v2` en CPU, aserción 384 dimensiones
- `pipeline.py`: Escaneo recursivo de `.md`, chunking inteligente, embedding batch, barra Rich

### Fase 3 — Extracción de Grafos mediante IA
- `extractor.py`: Conexión OpenRouter (`google/gemini-2.5-flash-free`), prompt estructurado, reintentos con backoff, parseo robusto
- Integración en pipeline: entidades/relaciones → tablas PostgreSQL

### Fase 4 — Consultas de Prueba
- `test_queries.py`: 4 modos de consulta (vectorial, grafo, híbrido, RAG)
- `main.py` migrado: CLI con `db-init`, `db-drop`, `run`, `query`, `ask`, `list`
- `gui.py` migrado: GUI tkinter con búsqueda híbrida y ejecución de pipeline
- `core/chat_agent.py` migrado: PostgreSQL + pgvector en lugar de FAISS
- `core/memory_manager.py` e `indexer.py` deprecados

---

## 📐 Arquitectura actual

```
ALEsys/
├── config.py          ← Configuración centralizada
├── db_manager.py      ← PostgreSQL + pgvector (psycopg v3)
├── embedder.py        ← Embeddings locales CPU (384 dim)
├── extractor.py       ← Extracción entidades/relaciones (OpenRouter)
├── pipeline.py        ← Orquestador principal
├── test_queries.py    ← Consultas híbridas
├── main.py            ← CLI
├── gui.py             ← GUI tkinter
├── core/
│   ├── chat_agent.py  ← RAG conversacional
│   └── web_search.py  ← DuckDuckGo
├── requirements.txt
└── history.md
```

---

## 🛠 Pendientes / Mejoras

- [ ] Ejecutar `run_tests.sh` para validar pipeline completa contra BD real
- [ ] Control de duplicados en pipeline (evitar re-indexar libros ya procesados)
- [ ] Tests unitarios con pytest
- [ ] `.vscode/launch.json` para depuración con debugpy

---

## 📦 Dependencias

| Paquete | Versión | Propósito |
|---------|---------|-----------|
| `psycopg` | >=3.1.0 | Driver PostgreSQL |
| `sentence-transformers` | >=2.2.0 | Embeddings locales |
| `httpx` | >=0.27.0 | Cliente HTTP para OpenRouter |
| `ddgs` | >=6.0.0 | Búsqueda web DuckDuckGo |
| `rich` | >=13.0.0 | CLI con formato y progreso |

---

*Generado automáticamente — 2026-06-09*
