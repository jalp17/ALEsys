# Changelog

## [2.1.0] - 2026-07-26
### Added
- Pipeline de ingesta de PDFs con MinerU + PyMuPDF fallback (Fase 29)
- `PDFProcessor` orquestador con progress tracking
- `Organizer` para estructura limpia de salida
- `CitationExtractor` integrado en pipeline
- GraphRAG ingestion hook: chunking + embeddings batch + DB index
- API endpoints: `/ingestion/pdf`, `/ingestion/batch`, `/ingestion/status/:id`
- WebSocket `/ws/ingestion/:id` para progreso real-time
- `GET /ingestion/config` y `PUT /ingestion/config`
- WebUI: `IngestionPanel`, `BatchIngestion`, `IngestionHistory`
- Tests unitarios para ingestion module (32 tests)
- E2E test suite (7 escenarios) + benchmarks + CI workflow
- Integración `ingestion_db_test.rs` (PostgreSQL)
- `KnowledgeCuration` splitter para capítulos > 5000 chars

### Changed
- Core module structure: `crates/core/src/ingestion/` (plugin, mineru_wrapper, pymupdf_fallback, organizer, pdf_processor, models, progress)
- GraphRAG module: agregado `ingestion_hook.rs`
- Tracking de jobs: `ingestion_jobs` table en PostgreSQL

### Fixed
- N/A

### Deprecated
- N/A

### Removed
- N/A

### Security
- Auth: JWT + RBAC (`ingestion:write`, `ingestion:read`) en API
- Rate limiting: 5 concurrent jobs/user via semaphore

### Testing
- 351 tests unitarios passing (alesys-core)
- 7 escenarios E2E (offline + live)
- Benchmarks de ingesta en `benches/ingestion_bench.rs`
- CI: `.github/workflows/ingestion-test.yml`

- Auth: JWT + RBAC (`ingestion:write`, `ingestion:read`) en API
- Rate limiting: 5 concurrent jobs/user via semaphore
