## Descripción
Fallback de extracción PDF usando PyMuPDF + pdfplumber (sin GPU).

## Tareas
- [ ] `extract_text(pdf_path)` → `Vec<PageText>` (pdfplumber)
- [ ] `extract_images(pdf_path, output_dir)` → `Vec<ImageRef>` (pymupdf)
- [ ] `extract_tables(pdf_path)` → `Vec<Table>`
- [ ] OCR opcional: `tesseract` subprocess por imagen
- [ ] Benchmark vs MinerU output

## Archivos
- `crates/core/src/ingestion/pymupdf_fallback.rs`
- `crates/core/src/ingestion/tests/fallback_test.rs`

## Labels
fase29, ingestion, priority:high