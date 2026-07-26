---
fecha: 2026-07-26
tipo: documentacion
proyecto: alesys-research-assistant
area: desarrollo-web
etiquetas: [alesys, fase29, github-issues, tickets, completado]
---

# Fase 29: GitHub Issues Template

Archivo con plantillas para issues de Fase 29. **Todos los tickets fueron completados e integrados en `master` (commit `43254af`).**

Uso como referencia para futuras fases:
1. Crear `FaseXX-ISSUES.md` con formato similar
2. Usar prefijo `TICKET-{FASE}.{NUM}` (ej: `TICKET-30.1`)
3. Marcar tareas con `- [x]` cuando completen
4. Al finalizar fase, actualizar este archivo con estado final

## Uso

```bash
# Crear issues individualmente
gh issue create --title "TICKET-29.1: PDFProcessor Orchestrator" \
    --body-file /mnt/src_file/desarrollo_git/ALEsys/.github/ISSUE_TEMPLATES/TICKET-29.1.md \
    --label "fase29,plugin-system,high-priority"

# O importar todos con script
chmod +x .github/scripts/create-fase29-issues.sh
.github/scripts/create-fase29-issues.sh
```

---

## Issues Template (Copy to .github/ISSUE_TEMPLATES/)

### TICKET-29.1: Plugin Skeleton + Config

**Título:** feat(29.1): IngestionPlugin skeleton con configuración

**Body:**
```markdown
## Descripción
Crear estructura base del módulo `ingestion/` con plugin Fase 11 compatible.

## Tareas
- [x] Crear `crates/core/src/ingestion/` estructura
- [x] Implementar `IngestionPlugin` trait Plugin (Fase 11)
- [x] Config schema con `mineru_model_path`, `output_base_dir`, `fallback_enabled`, `default_ocr_langs`
- [x] `on_init`: verificar Python 3.10+, `magic-pdf --version`, CUDA
- [x] Tests unitarios básicos

## Archivos
- `crates/core/src/ingestion/mod.rs`
- `crates/core/src/ingestion/plugin.rs`
- `crates/core/src/ingestion/config.rs`
- `crates/core/src/ingestion/tests/plugin_test.rs`

## Labels
fase29, plugin-system, priority:high
```

---

### TICKET-29.2: MinerUWrapper

**Título:** feat(29.2): MinerUWrapper subprocess + streaming

**Body:**
```markdown
## Descripción
Wrapper Python subprocess para MinerU con streaming de logs y timeout.

## Tareas
- [x] `execute_magic_pdf(pdf_path, output_dir, options)` → `Result<MinerUOutput>`
- [x] Streaming logs con `tracing` (info/debug/error)
- [x] Timeout configurable (default 20h), kill graceful
- [x] Auto-descarga modelos si no existen
- [x] GPU detection: `nvidia-smi` + `torch.cuda.is_available()`
- [x] Retry logic: 1 reintento con fallback

## Archivos
- `crates/core/src/ingestion/mineru_wrapper.rs`
- `crates/core/src/ingestion/tests/mineru_wrapper_test.rs`

## Labels
fase29, ingestion, priority:high
```

---

### TICKET-29.3: PyMuPDFFallback

**Título:** feat(29.3): PyMuPDF fallback sin GPU

**Body:**
```markdown
## Descripción
Fallback de extracción PDF usando PyMuPDF + pdfplumber (sin GPU).

## Tareas
- [x] `extract_text(pdf_path)` → `Vec<PageText>` (pdfplumber)
- [x] `extract_images(pdf_path, output_dir)` → `Vec<ImageRef>` (pymupdf)
- [x] `extract_tables(pdf_path)` → `Vec<Table>`
- [x] OCR opcional: `tesseract` subprocess por imagen
- [x] Benchmark vs MinerU output

## Archivos
- `crates/core/src/ingestion/pymupdf_fallback.rs`
- `crates/core/src/ingestion/tests/fallback_test.rs`

## Labels
fase29, ingestion, priority:high
```

---

### TICKET-29.4: Organizer (Reorganización)

**Título:** feat(29.4): Organizer port de reordenar_db_p.py

**Body:**
```markdown
## Descripción
Reorganizar salida MinerU en estructura book/chapter.md + images/.

## Tareas
- [x] `reorganize(mineru_output_dir, target_dir)` → `OrganizedOutput`
- [x] Parse MD: extraer `![]()` refs → Set<Path>
- [x] Move imágenes referenciadas a `target_dir/images/`
- [x] Move MD a `target_dir/chapter.md`
- [x] Cleanup: `rm -rf auto/`, duplicados, dirs vacíos
- [x] Log generation: `_reorg_logs/{book}_{timestamp}.log`

## Archivos
- `crates/core/src/ingestion/organizer.rs`
- `crates/core/src/ingestion/tests/organizer_test.rs`

## Labels
fase29, ingestion, automation, priority:high
```

---

### TICKET-29.5: PDFProcessor Orchestrator

**Título:** feat(29.5): PDFProcessor orchestrator

**Body:**
```markdown
## Descripción
Orquestador que coordina MinerU/Fallback + Organizer con progress reporting.

## Tareas
- [x] `process(job)`: decide MinerU vs Fallback
- [x] `process_batch(jobs)`: semáforo concurrencia (max_parallel)
- [x] Progress reporting via `IngestionProgress` events
- [x] Error handling: partial results, cleanup temp dirs
- [x] Integración KnowledgeCuration: auto-split chapters > 5000 tokens

## Archivos
- `crates/core/src/ingestion/pdf_processor.rs`
- `crates/core/src/ingestion/progress.rs`

## Labels
fase29, ingestion, orchestration, priority:high
```

---

### TICKET-29.6: API Endpoints + WebSocket

**Título:** feat(29.6): API ingestion endpoints + WS

**Body:**
```markdown
## Descripción
REST + WebSocket endpoints para ingestion.

## Endpoints
- POST /ingestion/pdf
- POST /ingestion/batch
- GET /ingestion/status/:id
- WS /ingestion/ws/:id (streaming progress)
- GET /ingestion/config
- PUT /ingestion/config

## Tareas
- [x] REST endpoints con request/response schemas
- [x] WebSocket streaming progress
- [x] Auth: JWT + RBAC (`ingestion:write`, `ingestion:read`)
- [x] Rate limiting: 5 concurrent jobs/user

## Archivos
- `crates/api/src/handlers/ingestion.rs`
- `crates/api/src/routes.rs` (update)

## Labels
fase29, api, websocket, priority:medium
```

---

### TICKET-29.7: Frontend Ingestion Panel

**Título:** feat(29.7): Frontend IngestionPanel con WS

**Body:**
```markdown
## Descripción
Panel de ingesta en webui: drag-drop, opciones, progress bars, history.

## Componentes
- `IngestionPanel.tsx`: drag-drop PDF, selector topic, opciones OCR/formulas
- `BatchIngestion.tsx`: multi-file, progress bars, queue management
- `IngestionHistory.tsx`: lista jobs, estado, link a output dir

## Tareas
- [x] Drag-drop PDF files
- [x] Selector topic + opciones avanzadas
- [x] WebSocket connection para progress real-time
- [x] Integración ResearchLayout (Fase 31)

## Archivos
- `webui/src/pages/ingestion/IngestionPanel.tsx`
- `webui/src/pages/ingestion/BatchIngestion.tsx`
- `webui/src/pages/ingestion/IngestionHistory.tsx`

## Labels
fase29, frontend, ui, priority:medium
```

---

### TICKET-29.8: GraphRAG Integration Hook

**Título:** feat(29.8): GraphRAG ingestion hook

**Body:**
```markdown
## Descripción
Hook post-ingestión para indexar chunks automáticamente.

## Tareas
- [x] Hook post-ingestión: `GraphRAG::index_documents(chapters)`
- [x] Metadatos: `topic`, `session_id`, `source_pdf`, `chapter_title`
- [x] Chunking strategy: por sección + overlap 200 tokens
- [x] Embeddings: ONNX Runtime batch 32

## Archivos
- `crates/core/src/graphrag/ingestion_hook.rs`
- `crates/core/src/ingestion/pdf_processor.rs` (hook integration)

## Labels
fase29, graphrag, embeddings, priority:medium
```

---

### TICKET-29.9: Tests E2E + Benchmarks

**Título:** test(29.9): E2E ingestion tests + benchmarks

**Body:**
```markdown
## Descripción
Suite de tests end-to-end para ingestion pipeline.

## Tareas
- [x] Test suite: 10 papers variados (1-200 págs, fórmulas, tablas, scans)
- [x] Metrics: latency, accuracy, memoria, GPU usage
- [x] CI: GitHub Action `ingestion-test.yml` (self-hosted con GPU)
- [x] Regression: golden files para organizer output
- [x] Security: plugin sandbox escape attempts

## Archivos
- `tests/e2e/ingestion_test.py`
- `benches/ingestion_bench.rs`
- `.github/workflows/ingestion-test.yml`

## Labels
fase29, testing, e2e, priority:medium
```

---

### TICKET-29.10: Docs + Scripts

**Título:** docs(29.10): Documentación + setup scripts

**Body:**
```markdown
## Descripción
Documentación y scripts de setup para MinerU + ingestion.

## Tareas
- [x] `docs/INGESTION_PIPELINE.md`: arquitectura, config, troubleshooting
- [x] `scripts/setup-mineru.sh`: Python env, MinerU install, modelos download
- [x] `scripts/benchmark-ingestion.sh`: correr suite papers test
- [x] Update `README.md` + `CONTRIBUTING.md`
- [x] CHANGELOG v2.1.0

## Archivos
- `docs/INGESTION_PIPELINE.md`
- `scripts/setup-mineru.sh`
- `CHANGELOG.md` (v2.1.0)

## Labels
fase29, documentation, scripts, priority:low
```

---

## Script de Creación Masiva (Opcional)

```bash
#!/bin/bash
# .github/scripts/create-fase29-issues.sh

for file in .github/ISSUE_TEMPLATES/TICKET-29.*.md; do
    title=$(grep "^# " "$file" | head -1 | sed 's/^# //')
    label=$(grep "^## Labels" "$file" | sed 's/## Labels//')
    gh issue create --title "$title" --body-file "$file" --label "$label"
done
```

---

**Tags:** #alesys #fase29 #github-issues #tickets #automation