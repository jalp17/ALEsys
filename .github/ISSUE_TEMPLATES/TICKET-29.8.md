## Descripción
Hook post-ingestión para indexar chunks automáticamente.

## Tareas
- [ ] Hook post-ingestión: `GraphRAG::index_documents(chapters)`
- [ ] Metadatos: `topic`, `session_id`, `source_pdf`, `chapter_title`
- [ ] Chunking strategy: por sección + overlap 200 tokens
- [ ] Embeddings: ONNX Runtime batch 32

## Archivos
- `crates/core/src/graphrag/ingestion_hook.rs`
- `crates/core/src/ingestion/pdf_processor.rs` (hook integration)

## Labels
fase29, graphrag, embeddings, priority:medium