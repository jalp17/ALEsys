## Descripción
Deduplicación de citas basada en título, año y DOI.

## Tareas
- [ ] Algoritmo similitud título+year+DOI
- [ ] Umbral configurale (default 0.8)
- [ ] Merge deduplicadas con prioridad DOI
- [ ] Tests con datos duplicados reales

## Archivos
- `crates/core/src/bibliography/deduplicator.rs`
- `crates/core/src/bibliography/tests/deduplicator_test.rs`

## Labels
fase30, bibliography, priority:low