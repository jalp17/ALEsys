## Descripción
Orquestador que coordina MinerU/Fallback + Organizer con progress reporting.

## Tareas
- [ ] `process(job)`: decide MinerU vs Fallback
- [ ] `process_batch(jobs)`: semáforo concurrencia (max_parallel)
- [ ] Progress reporting via `IngestionProgress` events
- [ ] Error handling: partial results, cleanup temp dirs
- [ ] Integración KnowledgeCuration: auto-split chapters > 5000 tokens

## Archivos
- `crates/core/src/ingestion/pdf_processor.rs`
- `crates/core/src/ingestion/progress.rs`

## Labels
fase29, ingestion, orchestration, priority:high