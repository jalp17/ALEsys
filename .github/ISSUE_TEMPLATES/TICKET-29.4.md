## Descripción
Reorganizar salida MinerU en estructura book/chapter.md + images/.

## Tareas
- [ ] `reorganize(mineru_output_dir, target_dir)` → `OrganizedOutput`
- [ ] Parse MD: extraer `![]()` refs → Set<Path>
- [ ] Move imágenes referenciadas a `target_dir/images/`
- [ ] Move MD a `target_dir/chapter.md`
- [ ] Cleanup: `rm -rf auto/`, duplicados, dirs vacíos
- [ ] Log generation: `_reorg_logs/{book}_{timestamp}.log`

## Archivos
- `crates/core/src/ingestion/organizer.rs`
- `crates/core/src/ingestion/tests/organizer_test.rs`

## Labels
fase29, ingestion, automation, priority:high