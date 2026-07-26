---
fecha: 2026-07-26
tipo: documentacion
proyecto: alesys-research-assistant
area: desarrollo-web
etiquetas: [alesys, fase30, github-issues, tickets, completado]
---

# Fase 30: Bibliografía y Citas Académicas

Archivo con plantillas para issues de Fase 30. **Todos los tickets fueron completados e integrados en `master` (commit `14ea111`).**

El código ya existe en `crates/core/src/bibliography/` e integrado en el pipeline de ingesta.

## Uso como referencia para futuras fases

1. Crear `FaseXX-ISSUES.md` con formato similar
2. Usar prefijo `TICKET-{FASE}.{NUM}` (ej: `TICKET-31.1`)
3. Marcar tareas con `- [x]` cuando completen
4. Al finalizar fase, actualizar este archivo con estado final

---

## Issues Template

### TICKET-30.1: Bibliography Skeleton + Types

**Título:** feat(30.1): Bibliography module skeleton con tipos Citation y CitationStyle

**Body:**
```markdown
## Descripción
Estructura base del módulo `bibliography/` con tipos fundamentales.

## Tareas
- [x] Crear `crates/core/src/bibliography/` estructura
- [x] Implementar `Citation` struct con campos bibliográficos (title, authors, year, doi, url, pages, volume, issue, citation_key)
- [x] `CitationStyle` enum (APA, MLA, Chicago, IEEE, BibTeX)
- [x] `CitationExtractorConfig` schema con opciones de extracción
- [x] Tests unitarios básicos

## Archivos
- `crates/core/src/bibliography/mod.rs`
- `crates/core/src/bibliography/models.rs`
- `crates/core/src/bibliography/tests/models_test.rs`

## Labels
fase30, bibliography, priority:high
```

---

### TICKET-30.2: Citation Extractor

**Título:** feat(30.2): CitationExtractor - extracción de citas desde markdown

**Body:**
```markdown
## Descripción
Extractor de citas bibliográficas desde texto markdown usando regex patterns y NLP.

## Tareas
- [x] Match citation patterns: `[^1]`, `[^Smith2023]`, `(Smith, 2023)`, `Smith et al. (2023)`
- [x] Parse DOI desde texto raw (regex DOI standard)
- [x] Extraer referencias bibliográficas de sección "References" / "Bibliography"
- [x] Soporte para múltiples estilos de cita en el mismo documento
- [x] `extract_bibliography(markdown)` → `BibliographyResult` con citations + references section
- [x] Configuración: `min_confidence`, `extract_references_section`, `doi_only`

## Archivos
- `crates/core/src/bibliography/extractor.rs`
- `crates/core/src/bibliography/tests/extractor_test.rs`

## Labels
fase30, bibliography, nlp, priority:high
```

---

### TICKET-30.3: Citation Formatter

**Título:** feat(30.3): CitationFormatter - formato APA 7, MLA 9, Chicago, IEEE, BibTeX

**Body:**
```markdown
## Descripción
Formateo de citas en estilos académicos estándar.

## Tareas
- [x] Implementar formato APA 7 (Author, Year, Title, Journal, Volume, Issue, Pages, DOI)
- [x] Implementar formato MLA 9 (Author. "Title." Journal, vol, issue, year, pp. DOI)
- [x] Implementar formato Chicago (Author. "Title." Journal Volume, Issue (Year): Pages. DOI)
- [x] Implementar formato IEEE ([1] Author, "Title," Journal, vol. Volume, no. Issue, pp. Pages, Year. DOI)
- [x] Implementar formato BibTeX (@article{key, author = {...}, title = {...}, journal = {...}, ...})
- [x] `format_citation(citation, style)` → `String`
- [x] `format_bibliography(citations, style)` → `String` con sorting alfabético
- [x] Tests con casos edge: múltiples autores, sin DOI, sin páginas, etc.

## Archivos
- `crates/core/src/bibliography/formatter.rs`
- `crates/core/src/bibliography/tests/formatter_test.rs`

## Labels
fase30, bibliography, formatting, priority:high
```

---

### TICKET-30.4: Bibliography Storage (PostgreSQL)

**Título:** feat(30.4): Bibliography storage en PostgreSQL con tabla bibliography_citations

**Body:**
```markdown
## Descripción
Persistencia de citas extraídas en PostgreSQL con schema completo.

## Tareas
- [x] Migración `20241201_create_bibliography.sql` (tabla `bibliography`)
- [x] Migración `20260101_create_bibliography_citations.sql` (tabla `bibliography_citations`)
- [x] `store(citation)` → inserción/UPSERT con conflicto en `doi` o `citation_key`
- [x] `list_by_chapter(chapter_id)` → consulta paginada con filtros
- [x] `list_by_source_pdf(source_pdf)` → todas las citas de un PDF
- [x] `search_by_author(author)` → búsqueda parcial case-insensitive
- [x] `search_by_year(year)` → filtro por año
- [x] Integración con SQLx y migraciones automáticas
- [x] Índices: `doi`, `citation_key`, `chapter_id`, `source_pdf`, `author_year`

## Archivos
- `crates/core/src/bibliography/storage.rs`
- `crates/core/migrations/20241201_create_bibliography.sql`
- `crates/core/migrations/20260101_create_bibliography_citations.sql`
- `crates/core/src/bibliography/tests/storage_test.rs`

## Labels
fase30, bibliography, postgres, priority:high
```

---

### TICKET-30.5: Citation Deduplicator

**Título:** feat(30.5): CitationDeduplicator - deduplicación inteligente de citas

**Body:**
```markdown
## Descripción
Algoritmo de deduplicación de citas basado en similitud de título, año y DOI.

## Tareas
- [x] Algoritmo similitud título + año + DOI (Levenshtein normalizado)
- [x] Umbral configurable (default 0.85)
- [x] Merge deduplicadas con prioridad: DOI > citation_key > título + año
- [x] Preservar metadatos más completos (DOI, URL, páginas, autores)
- [x] `deduplicate(citations)` → `Vec<Citation>` únicas + reporte de merges
- [x] Tests con datos duplicados reales: mismas citas con variaciones de formato
- [x] Integración en `CitationExtractorConfig` para auto-dedupe opcional

## Archivos
- `crates/core/src/bibliography/deduplicator.rs`
- `crates/core/src/bibliography/tests/deduplicator_test.rs`

## Labels
fase30, bibliography, deduplication, priority:medium
```

---

## Integración con Pipeline de Ingesta

La funcionalidad de bibliografía está integrada en el pipeline de ingesta (Fase 29):

| Integración | Archivo |
|-------------|---------|
| PDFProcessor llama a extractor | `crates/core/src/ingestion/pdf_processor.rs:100` |
| Citas incluidas en `IngestionResult` | `crates/core/src/ingestion/models.rs:53` |
| Configuración en `IngestionConfig` | `crates/core/src/ingestion/config.rs` |
| Plugin expone funcionalidad | `crates/core/src/ingestion/plugin.rs` |

---

## Tests

```bash
# Tests unitarios bibliografía
cargo test -p alesys-core --lib bibliography::

# Tests de integración (requiere DB)
cargo test -p alesys-api --test bibliography_db_test

# Tests E2E completos
python3 tests/e2e/ingestion_test.py --include-bibliography
```

---

## Script de Creación Masiva (Opcional)

```bash
#!/bin/bash
# .github/scripts/create-fase30-issues.sh

for file in .github/ISSUE_TEMPLATES/TICKET-30.*.md; do
    title=$(grep "^# " "$file" | head -1 | sed 's/^# //')
    label=$(grep "^## Labels" "$file" | sed 's/## Labels//')
    gh issue create --title "$title" --body-file "$file" --label "$label"
done
```

---

**Tags:** #alesys #fase30 #github-issues #tickets #bibliography #completado