---
name: TICKET-31.7
about: API Endpoints Research
title: "feat(31.7): API endpoints para ResearchLayout (CRUD projects, notes, network)"
labels: fase31, research-layout
assignees: ''
---

## Descripción
Endpoints REST + WebSocket para funcionalidad de investigación.

## Endpoints
| Método | Endpoint | Descripción |
|--------|----------|-------------|
| GET    | `/api/v1/research/projects` | Listar proyectos |
| POST   | `/api/v1/research/projects` | Crear proyecto |
| GET    | `/api/v1/research/projects/:id` | Detalle proyecto |
| PUT    | `/api/v1/research/projects/:id` | Actualizar proyecto |
| DELETE | `/api/v1/research/projects/:id` | Eliminar proyecto |
| GET    | `/api/v1/research/projects/:id/stats` | Estadísticas |
| GET    | `/api/v1/research/projects/:id/export` | Export completo |
| GET    | `/api/v1/research/notes` | Listar notas (filtros) |
| POST   | `/api/v1/research/notes` | Crear nota |
| GET    | `/api/v1/research/network` | Grafo citas (nodos/aristas) |
| GET    | `/api/v1/research/literature/search` | Búsqueda literatura |
| WS     | `/ws/research/:projectId` | Sync real-time colaborativo |

## Tareas
- [ ] Handlers en `crates/api/src/handlers_research.rs`
- [ ] Rutas en `crates/api/src/routes.rs`
- [ ] Auth: JWT + RBAC (`research:read`, `research:write`)
- [ ] Rate limiting: 50 req/min
- [ ] Tests de integración

## Archivos
- `crates/api/src/handlers_research.rs`
- `crates/api/src/routes.rs` (update)
- `crates/api/tests/research_test.rs`

## Labels
fase31, api, research, priority:high
