# Pipeline de Ingesta de PDFs (Fase 29)

## Arquitectura

```
PDF Input
    │
    ▼
PDFProcessor (orquestador)
    │
    ├──▶ MinerUWrapper ──▶ GPU/CPU inference ──▶ Markdown + imágenes
    │
    └──▶ PyMuPDFFallback ──▶ Extracción directa ──▶ Markdown + imágenes
              │
              ▼
         Organizer ──▶ Estructura limpia: auto/* → book_{id}/
              │
              ▼
         CitationExtractor ──▶ Bibliografía
              │
              ▼
         GraphRAG Hook ──▶ Chunking + Embeddings + DB index
```

## Configuración

```toml
[ingestion]
model_dir = "/path/to/models"
output_base_dir = "./output"
max_parallel = 4
```

### Modos de ingesta

| Modo | Descripción | GPU requerida |
|------|-------------|---------------|
| `auto` | Selecciona MinerU si hay GPU, fallback si no | Opcional |
| `mineru` | Fuerza MinerU (mejor calidad para fórmulas/tablas) | Recomendada |
| `pymupdf` | Fuerza PyMuPDF (rápido, sin GPU) | No |

## Troubleshooting

### MinerU no arranca

```bash
# Verificar GPU
nvidia-smi

# Verificar modelos
ls -la /path/to/models

# Reinstalar
pip install --upgrade magic-pdf[gpu]
```

### PyMuPDF fallback lento

- Aumentar `max_parallel` en config
- Desactivar OCR si no es necesario: `ocr_languages = []`

### Organizer no limpia `auto/`

- Verificar permisos de escritura en `output_base_dir`
- Revisar `book_root` para espacios en nombres de archivo

## API Endpoints

| Método | Endpoint | Descripción |
|--------|----------|-------------|
| POST | `/api/v1/ingestion/pdf` | Ingesta individual |
| POST | `/api/v1/ingestion/batch` | Ingesta por lotes |
| GET | `/api/v1/ingestion/status/:id` | Estado de job (DB-backed) |
| WS | `/ws/ingestion/:id` | Progreso en tiempo real (polling 500ms) |
| GET | `/api/v1/ingestion/config` | Configuración actual |
| PUT | `/api/v1/ingestion/config` | Actualizar configuración |

### Tracking de Jobs

Los jobs se persisten en PostgreSQL (`ingestion_jobs`):

- Estado: `pending` → `processing` → `completed` / `failed`
- Progreso: 0-100% con events `progress` y `completed` / `failed`
- Reconnects: cliente WS puede reconectar y reanudar desde DB
- Migración: `crates/core/migrations/20260726_create_ingestion_jobs.sql`

## GraphRAG Hook

Post-ingestión automática:

1. Detecta capítulos en markdown organizado
2. Si capítulo > 5000 chars: aplica `KnowledgeCuration::BySize` (2500 chars/chunk)
3. Embeddings batch 32 via ONNX Runtime
4. Inserta en `documentos` y `fragmentos` con metadatos

Metadatos almacenados:
- `topic`
- `session_id`
- `source_pdf`
- `chapter_title`

### KnowledgeCuration Splitter

El hook usa `DocumentSplitter::BySize { max_chars: 2500 }` para capítulos largos, preservando contexto semántico sin exceder límites de embedding.


## Estado Fase 29

| Ticket | Estado | Nota |
|--------|--------|------|
| 29.1 | ✅ | Plugin skeleton + config |
| 29.2 | ✅ | MinerU wrapper + streaming |
| 29.3 | ✅ | PyMuPDF fallback |
| 29.4 | ✅ | Organizer |
| 29.5 | ✅ | PDFProcessor orchestrator |
| 29.6 | ✅ | API + WS handlers |
| 29.7 | ✅ | Frontend panel |
| 29.8 | ✅ | GraphRAG hook |
| 29.9 | ✅ | E2E + benchmarks + CI |
| 29.10 | ✅ | Docs + scripts |

**Commit:** `43254af` en `master`