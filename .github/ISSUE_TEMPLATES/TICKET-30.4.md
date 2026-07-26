## Descripción
Persistir citas extraídas en PostgreSQL con tabla bibliography_citations.

## Tareas
- [ ] Tabla `bibliography_citations` con schema completo
- [ ] `store(citation)` inserción/conflito UPSERT
- [ ] `list_by_chapter(chapter_id)` consulta paginada
- [ ] Integración con migraciones SQLx
- [ ] Tests de persistencia

## Archivos
- `crates/core/src/bibliography/storage.rs`
- `migrations/20240101000000_create_bibliography_citations.sql`
- `crates/core/src/bibliography/tests/storage_test.rs`

## Labels
fase30, bibliography, priority:medium