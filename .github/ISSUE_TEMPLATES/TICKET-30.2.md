## Descripción
Extraer citas de texto markdown usando regex patterns y NLP.

## Tareas
- [ ] Match citation patterns como `[^1]`, `[^Smith2023]`
- [ ] Parse DOI desde texto raw
- [ ] Extraer referencias bibliográficas de sección "References"
- [ ] Soporte para múltiples estilos (APA, MLA, Chicago, IEEE)
- [ ] Tests con papers reales

## Archivos
- `crates/core/src/bibliography/extractor.rs`
- `crates/core/src/bibliography/tests/extractor_test.rs`

## Labels
fase30, bibliography, priority:high