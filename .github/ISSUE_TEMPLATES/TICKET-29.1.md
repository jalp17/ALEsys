## Descripción
Crear estructura base del módulo `ingestion/` con plugin Fase 11 compatible.

## Tareas
- [ ] Crear `crates/core/src/ingestion/` estructura
- [ ] Implementar `IngestionPlugin` trait Plugin (Fase 11)
- [ ] Config schema con `mineru_model_path`, `output_base_dir`, `fallback_enabled`, `default_ocr_langs`
- [ ] `on_init`: verificar Python 3.10+, `magic-pdf --version`, CUDA
- [ ] Tests unitarios básicos

## Archivos
- `crates/core/src/ingestion/mod.rs`
- `crates/core/src/ingestion/plugin.rs`
- `crates/core/src/ingestion/config.rs`
- `crates/core/src/ingestion/tests/plugin_test.rs`

## Labels
fase29, plugin-system, priority:high