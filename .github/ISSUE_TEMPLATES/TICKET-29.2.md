## Descripción
Wrapper Python subprocess para MinerU con streaming de logs y timeout.

## Tareas
- [ ] `execute_magic_pdf(pdf_path, output_dir, options)` → `Result<MinerUOutput>`
- [ ] Streaming logs con `tracing` (info/debug/error)
- [ ] Timeout configurable (default 20h), kill graceful
- [ ] Auto-descarga modelos si no existen
- [ ] GPU detection: `nvidia-smi` + `torch.cuda.is_available()`
- [ ] Retry logic: 1 reintento con fallback

## Archivos
- `crates/core/src/ingestion/mineru_wrapper.rs`
- `crates/core/src/ingestion/tests/mineru_wrapper_test.rs`

## Labels
fase29, ingestion, priority:high