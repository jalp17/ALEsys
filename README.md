# ALEsys v2.1.0 - AI IDE Framework

ALEsys es un framework RAG + agente configurable con capacidades avanzadas de IA para desarrollo de software, investigación científica y gestión de conocimiento.

---

## Tabla de Contenidos

- [Arquitectura](#arquitectura)
- [Ingesta de PDFs y Markdown](#ingesta-de-pdfs-y-markdown)
- [Setup Inicial](#setup-inicial)
- [API Reference](#api-reference)
- [Interfaz de Usuario](#interfaz-de-usuario)
- [Licencia](#licencia)

---

## Arquitectura

```
┌─────────────────────────────────────────────────────────────┐
│                    ALEsys ECOSYSTEM                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────────────┐         ┌─────────────────────┐    │
│  │   TAURI DESKTOP     │         │   WEBUI MULTI-USU   │    │
│  │   (1 usuario)       │         │   (Múltiples users) │    │
│  └──────────┬──────────┘         └──────────┬──────────┘    │
│             │                                │               │
│             │  MISMO CÓDIGO FRONTEND         │               │
│             │  (React + TypeScript)          │               │
│             └────────────┬───────────────────┘               │
│                          │                                    │
│                          ▼                                    │
│  ┌──────────────────────────────────────────────────────────┐│
│  │           ALESYS CORE (Rust Backend)                     ││
│  │  - API REST + WebSocket                                  ││
│  │  - GraphRAG (PostgreSQL + pgvector)                      ││
│  │  - 27 Módulos Especializados                             ││
│  │  - 335 Tests Unitarios                                   ││
│  └──────────────────────────────────────────────────────────┘│
│                          │                                    │
│                          ▼                                    │
│  ┌──────────────────────────────────────────────────────────┐│
│  │     PostgreSQL + pgvector + Grafos                       ││
│  └──────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

### Stack Tecnológico

| Componente | Tecnología |
|------------|------------|
| Backend | Rust (axum, sqlx, pgvector, mistralrs, ort) |
| Frontend | React + TypeScript + TailwindCSS |
| Base de Datos | PostgreSQL 16 + pgvector |
| Desktop | Tauri v2 |
| Multi-usuario | PHP 8.2 |

---

## Interfaz de Usuario

ALEsys incluye una interfaz web moderna con los siguientes paneles:

| Panel | Descripción |
|-------|-------------|
| Workspaces | Cuadernos de temas estilo AnythingLLM con tabs Chat/Docs/Graph |
| Ingestión | Ingesta de PDFs y Markdown con progreso en tiempo real |
| Investigación | Proyectos de I+D, notas, síntesis y export LaTeX |
| Graph Viewer | Visualización del grafo de conocimiento |
| Inventario | Gestión de activos LIMS con tracking de expiración |

### Capturas de Pantalla

Coloca tus capturas de pantalla en `docs/screenshots/` y referencia desde aquí:

```
docs/screenshots/
├── dashboard-home.png
├── workspace-chat.png
├── ingestion-panel.png
├── graph-viewer.png
└── research-synthesis.png
```

Ejemplo de inclusión en Markdown:

```markdown
![Dashboard Principal](docs/screenshots/dashboard-home.png)
![Panel de Ingesta](docs/screenshots/ingestion-panel.png)
```

---

## Ingesta de PDFs y Markdown

### Características

- **MinerU + PyMuPDF fallback**: Extracción híbrida con OCR y detección de fórmulas
- **Organizer**: Estructura limpia de salida (`auto/*` → `book_{id}/`)
- **GraphRAG hook**: Indexación automática post-ingesta (chunking + embeddings batch 32)
- **WebUI**: Drag-drop, progress bars en tiempo real, historial de jobs
- **Auth**: JWT + RBAC (`ingestion:write`, `ingestion:read`)
- **Ingesta de Markdown**: Indexación directa de libros y vaults pre-procesados (sin GPU)

### Modos de Ingesta

| Modo | Descripción | GPU |
|-------------|---------------|-----|
| `auto` | Selecciona automáticamente | Opcional |
| `mineru` | Mejor calidad, fórmulas/tablas | Recomendada |
| `pymupdf` | Rápido, sin GPU | No |
| `markdown` | Indexación directa de archivos .md | No |

### Setup

```bash
# Instalar MinerU con GPU
./scripts/setup-mineru.sh

# Ejecutar benchmarks
./scripts/benchmark-ingestion.sh
```

### Documentación

- [Pipeline de Ingesta](docs/INGESTION_PIPELINE.md)

---

## Setup Inicial

### Prerrequisitos

- Rust 1.80+
- Node.js 20+
- pnpm 9+
- Docker + Docker Compose
- PostgreSQL 16 (opcional, si no usas Docker)

### Instalación

```bash
# 1. Clone el repositorio
git clone https://github.com/tu-usuario/ALEsys
cd ALEsys

# 2. Ejecutar setup
./scripts/setup-dev.sh

# 3. Configurar variables de entorno
cp docker/.env.example .env
# Edita .env con tus configuraciones

# 4. Iniciar servicios
docker compose -f docker/docker-compose.yml up -d

# 5. Iniciar desarrollo
pnpm dev
```

### Verificar instalación

```bash
# Backend Rust
cargo run --bin alesys-cli -- --help

# Frontend Web
open http://localhost:5173

# Tests
cargo test --workspace --no-default-features --features test
```

---

## API Reference

### Endpoints Principales

#### Chat & GraphRAG
- `POST /api/v1/chat` - Chat con GraphRAG + sesiones
- `GET /api/v1/sessions` - Listar sesiones activas
- `POST /api/v1/sessions` - Crear sesión
- `GET /ws/chat` - WebSocket para streaming

#### Desarrollo
- `POST /api/v1/generate` - Generar archivos
- `POST /pair-programmer/analyze` - Analizar código
- `POST /pair-programmer/refactor` - Refactorizar
- `POST /debug/analyze` - Analizar errores
- `POST /test-generate` - Generar tests
- `POST /refactoring/analyze` - Analizar refactorización
- `POST /refactoring/preview` - Preview de cambios

#### Conocimiento
- `POST /kb/merge` - Fusionar documentos
- `POST /kb/split` - Dividir documentos
- `POST /kb/archive` - Archivar documentos
- `POST /kb/duplicates` - Detectar duplicados
- `POST /kb/quality` - Evaluar calidad
- `POST /search/advanced` - Búsqueda avanzada
- `POST /search/faceted` - Búsqueda con facetas
- `POST /search/suggest` - Autocompletado

#### Multi-Agente
- `GET /agents` - Listar agentes
- `GET /agents/stats` - Estadísticas de agentes
- `POST /agents/:id/execute` - Ejecutar agente
- `GET /collab/status` - Estado de colaboración
- `GET /collab/tasks` - Tasks de colaboración
- `POST /collab/consensus` - Consenso entre agentes

#### Operaciones
- `GET /analytics/usage` - Métricas de uso
- `GET /analytics/performance` - Performance
- `GET /analytics/users` - Analíticas por usuario
- `GET /analytics/reports` - Reportes
- `GET /workflows` - Listar workflows
- `POST /workflows` - Crear workflow
- `POST /workflows/:id/run` - Ejecutar workflow

#### Ingesta de PDFs y Markdown (Fase 29)
- `POST /api/v1/ingestion/pdf` - Ingesta individual con MinerU + PyMuPDF fallback
- `POST /api/v1/ingestion/batch` - Ingesta por lotes
- `POST /api/v1/ingestion/markdown` - Ingesta de Markdown (async, job-based)
- `GET /api/v1/ingestion/status/:id` - Estado de job (PDF)
- `GET /api/v1/ingestion/markdown/status/{id}` - Estado de job (Markdown)
- `GET /api/v1/ingestion/markdown/jobs` - Listar jobs Markdown
- `POST /api/v1/ingestion/markdown/{id}/cancel` - Cancelar job Markdown
- `WS /ws/ingestion/:id` - Progreso en tiempo real
- `GET /api/v1/ingestion/config` - Configuración actual
- `PUT /api/v1/ingestion/config` - Actualizar configuración

#### LLM (Carga Lazy)
- `GET /api/v1/llm/status` - Estado del modelo LLM
- `POST /api/v1/llm/load` - Cargar modelo en memoria
- `POST /api/v1/llm/unload` - Descargar modelo de memoria

#### Sistema
- `GET /health` - Health check
- `GET /metrics` - Métricas del sistema
- `GET /plugins` - Listar plugins
- `POST /plugins/:id/enable` - Habilitar plugin
- `POST /plugins/:id/disable` - Deshabilitar plugin

---

## Testing

| Tipo | Herramienta | Cuándo |
|------|-------------|--------|
| Unitarios | `cargo test -p alesys-core --lib` | Cada cambio en lógica Rust |
| Integración | `cargo test -p alesys-api --test <suite>` | Backend + PostgreSQL |
| E2E | Python + Playwright | Pipeline completo (PDF → GraphRAG) |
| Benchmarks | `cargo bench` | Regresión de performance |

### Comandos

```bash
# Unit tests (core)
cargo test -p alesys-core --lib

# API tests
cargo test -p alesys-api

# E2E offline (sin backend)
python3 tests/e2e/ingestion_test.py --offline

# E2E live (con backend)
python3 tests/e2e/ingestion_test.py

# Benchmarks
cargo bench -p alesys-core ingestion_bench
```

**Total: 351 tests unitarios · 7 escenarios E2E · Benchmarks de ingesta**

---

## Métricas del Proyecto

| Métrica | Valor |
|---------|-------|
| Versión | v2.1.0 |
| Módulos Core | 27 |
| Endpoints API | ~50 |
| Tests Unitarios | 351 |
| Líneas de Código | ~15,000+ |
| Fases Completadas | 29 |

---

## Licencia

GNU AGPL v3.0

---

## CHANGELOG v2.1.0

### Agregado
- Fase 29: Pipeline de ingesta de PDFs (MinerU + PyMuPDF fallback)
- 10 tickets TICKET-29.1 a 29.10 completados
- GraphRAG ingestion hook con KnowledgeCuration splitter
- API endpoints: `/ingestion/pdf`, `/ingestion/batch`, `/ingestion/status/:id`, `/ws/ingestion/:id`, `/ingestion/config`
- WebUI: drag-drop, progress bars, historial de jobs
- 351 tests unitarios passing (32 ingestion tests)
- 7 escenarios E2E + benchmarks + CI workflow
- Documentación Fase 29 y pipeline de ingesta

### Mejorado
- Performance con cache LRU + TTL
- Security hardening (rate limiting, validación)
- Health checks y backup manager
- Búsqueda avanzada con facetas

### Cambiado
- Metodología de tests: unit → integración → e2e → benchmarks
- Flujo de tickets: issues por fase
- Version establecida a v2.1.0

---

### Fase I+D Científico (Sprints 1-4) — Nuevas capacidades

**Sprint 1 — Fundamentos:**
- Research Module persistido en PostgreSQL (migración + handlers sqlx + audit columns)
- DVC + MinIO para versionado de PDFs/datos (remote local)
- MLflow self-hosted en docker-compose + tracking client en Rust
- Schema metadatos experimentales en chunks
- DockerSandbox prototipo

**Sprint 2 — Ejecución y análisis:**
- Clasificador LLM-structured-output
- Table Extractor (PyMuPDF → Camelot → TATR) → CSV/Parquet
- Verificación factual (RAGAS LLM judge + numeric consistency + citations)
- Notebook component (celdas code/markdown, run, output)
- CollabEditor Yjs (CRDT + y-websocket + y-monaco + awareness)

**Sprint 3 — Calidad y colaboración:**
- TableExtractor integrado en ingestion pipeline
- Verifier integrado en /api/v1/chat
- Protocol Templates + Experiment Runs (ELN)
- Export LaTeX
- Notebook + CollabEditor integrados en SynthesisPanel

**Sprint 4 — Enterprise y escalabilidad:**
- Rate limiting robusto + métricas Prometheus
- Multi-tenant aislamiento (owner_id columns + RLS policies)
- Inventory/LIMS module
- Yjs backend WebSocket
- lakeFS en docker-compose
- LLM streaming mejorado
- Firecracker on-prem documentación
- E2E tests + CI workflow

**Nuevos endpoints:**
- `/api/v1/research/runs` (GET/POST), `/api/v1/research/runs/:id` (PUT/DELETE)
- `/api/v1/research/export/:project_id/latex`
- `/api/v1/inventory` (GET/POST), `/api/v1/inventory/:id` (PUT/DELETE)

**Nuevos servicios Docker:**
- `mlflow` (:5000), `yjs-server` (:1234), `lakefs` (:8000)

**Nuevos módulos Rust:**
- `classifier`, `verification`, `tracking`, `table_extractor` (en ingestion), `executor::sandbox`

**Tests:** 361 unitarios passing (core), tsc limpio (webui)

---

Hecho por el equipo ALEsys