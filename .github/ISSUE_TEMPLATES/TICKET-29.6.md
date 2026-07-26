## Descripción
REST + WebSocket endpoints para ingestion.

## Endpoints
- POST /ingestion/pdf
- POST /ingestion/batch
- GET /ingestion/status/:id
- WS /ingestion/ws/:id (streaming progress)
- GET /ingestion/config
- PUT /ingestion/config

## Tareas
- [ ] REST endpoints con request/response schemas
- [ ] WebSocket streaming progress
- [ ] Auth: JWT + RBAC (`ingestion:write`, `ingestion:read`)
- [ ] Rate limiting: 5 concurrent jobs/user

## Archivos
- `crates/api/src/handlers/ingestion.rs`
- `crates/api/src/routes.rs` (update)

## Labels
fase29, api, websocket, priority:medium