# Línea de Tiempo de Versiones — ALEsys

## Resumen

| Versión | Fecha | Descripción |
|---------|-------|-------------|
| v1.0.0 | 2026-07-14 | Arquitectura multi-backend LLM con auto-selección |
| v1.2.0 | 2026-07-17 | Fase 2: Generación de archivos |
| v1.3.0 | 2026-07-17 | Fase 3: Sesiones |
| v1.4.0 | 2026-07-17 | Fase 4: Correcciones críticas (batch 1) |
| v1.5.0 | 2026-07-17 | Fase 5: Optimización |
| v1.6.0 | 2026-07-17 | Fase 6: Visualizador de grafos |
| v1.7.0 | 2026-07-17 | Fase 7: Búsqueda híbrida |
| v1.8.0 | 2026-07-18 | Fase 7: Editor IDE |
| v1.9.0 | 2026-07-18 | Fase 8: Aplicación de escritorio Tauri |
| v1.10.0 | 2026-07-18 | Fase 7: Ejecución local |
| v1.11.0 | 2026-07-18 | Fase 9: Agente remoto |
| v1.12.0 | 2026-07-18 | Fase 9.2: Ejecución de agente |
| v1.13.0 | 2026-07-18 | Fase 9.3: Frontend de agente |
| v1.14.0 | 2026-07-18 | Fase 9.4: CI/CD de agente |
| v1.15.0 | 2026-07-19 | Fase 10: Seguridad |
| v1.16.0 | 2026-07-19 | Fase 11: Sistema de plugins |
| v1.17.0 | 2026-07-19 | Fase 11.2: Frontend de plugins |
| v1.18.0 | 2026-07-19 | Fase 12: Multi-agente |
| v1.19.0 | 2026-07-19 | Fase 12.2: UI del orquestador |
| v1.20.0 | 2026-07-19 | Fase 13: Voz y multimodal |
| v1.21.0 | 2026-07-19 | Fase 13.2: UI de voz |
| v1.22.0 | 2026-07-19 | Fase 14: Colaboración |
| v1.23.0 | 2026-07-19 | Fase 14.2: UI de colaboración |
| v1.24.0 | 2026-07-19 | Fase 15: Pair Programmer |
| v1.25.0 | 2026-07-19 | Fase 15.2: UI del Pair Programmer |
| v1.26.0 | 2026-07-19 | Fase 16: Aprendizaje |
| v1.27.0 | 2026-07-19 | Fase 17.2: UI + API del Asistente de Depuración |
| v1.28.0 | 2026-07-19 | Fase 18: Generación de tests |
| v1.29.0 | 2026-07-19 | Fase 19.2: Refactorización avanzada |
| v1.30.0 | 2026-07-19 | Fase 20.2: API + UI de Curación de Base de Conocimiento |
| v1.31.0 | 2026-07-19 | Fase 21: Colaboración multi-agente |
| v1.32.0 | 2026-07-19 | Fase 22: Analíticas avanzadas |
| v1.33.0 | 2026-07-19 | Fase 23: Automatización de workflows |
| v1.34.0 | 2026-07-19 | Fase 24: Búsqueda avanzada y filtros |
| v1.35.0 | 2026-07-19 | Fase 25: Optimización de performance |
| v1.36.0 | 2026-07-19 | Fase 26: Seguridad |
| v1.37.0 | 2026-07-19 | Fase 27: Despliegue en producción |
| v2.0.0 | 2026-07-19 | Fase 28: Integración final y pulido — v2.0.0 |
| v2.3.0 | 2026-11-17 | Ingesta Markdown async, Módulo I+D (Sprints 1-4), limpieza de API |

---

## Detalle por Era

### Era v1.x — Construcción del Core (Julio 2026)

Todas las versiones v1.x se liberaron entre el 14 y el 19 de julio de 2026 en un ciclo intensivo de desarrollo. Se construyeron 37 fases que cubren:

- **Fases 1-6**: Backend LLM, generación de archivos, sesiones, optimización, visualizador de grafos, búsqueda híbrida
- **Fases 7-9**: Editor IDE, ejecución local, agente remoto con frontend y CI/CD
- **Fases 10-14**: Seguridad, plugins, multi-agente, orquestador, voz, colaboración, pair programmer
- **Fases 15-27**: Aprendizaje, asistente de depuración, generación de tests, refactorización, curación de conocimiento, multi-agente, analíticas, workflows, búsqueda, performance, seguridad, despliegue

### Era v2.x — Madurez y Expansión (Julio 2026 — Noviembre 2026)

| Versión | Fecha | Cambios clave |
|---------|-------|---------------|
| **v2.0.0** | 2026-07-19 | Integración final y pulido. Incluye el gato 🐱 en el branding. |
| **v2.3.0** | 2026-11-17 | Ingesta Markdown async con job tracking y cancel, Módulo I+D científico (Sprints 1-4), rediseño del frontend (WorkspaceLayout, Command Palette, AppShell), documentación limpia, clasificación de PDFs, múltiples correcciones (UTF-8 slicing, fragmentos vector(1024), frontmatter NOT NULL, postcss.config.js). |

---

## Cambios en v2.3.0 (actual)

### Backend (Rust)
- Async Markdown ingestion con `MarkdownIndexer`, batching, progreso y cancelación
- Endpoints: `POST /ingestion/markdown`, `GET /ingestion/markdown/status/{id}`, `GET /ingestion/markdown/jobs`, `POST /ingestion/markdown/{id}/cancel`
- I+D Scientific Module: proyectos, notas, síntesis, export LaTeX, runs de experimentos
- DVC + MinIO para versionado de datos
- MLflow tracking integrado en Rust
- Classifier, Table Extractor, Verifier (RAGAS)
- Notebook + CollabEditor (Yjs CRDT)
- Multi-tenant con RLS, Inventory/LIMS
- lakeFS, Yjs backend WebSocket, métricas Prometheus

### Frontend (React + TypeScript)
- WorkspaceLayout con tabs Chat/Docs/Graph
- Command Palette (⌘K)
- AppShell rediseñado (nav rail, sección panel, inspector)
- MarkdownIngestionPanel con polling en tiempo real
- Tailwind CSS compilado correctamente (postcss.config.js)

### Documentación
- README limpiado (sin emojis, un solo gato 🐱, sección de screenshots)
- docs/INGESTION_PIPELINE.md y docs/LAZY_LLM_LOAD.md limpiados
- docs/screenshots/ para capturas de la interfaz

### Clasificación de PDFs
- `pdf_procesados/` — 14 libros con Markdown ya convertido (listos para indexar)
- `pdf_pendientes/` — 14 PDFs sin convertir (requieren MinerU/GPU), incluyendo la serie Landau (10 volúmenes)

---

## Cómo leer este documento

Este archivo se actualiza con cada nuevo tag. Para ver los tags disponibles:

```bash
git tag --list | sort -V
```

Para ver los cambios de una versión específica:

```bash
git log <tag-anterior>..<tag-actual> --oneline
```
