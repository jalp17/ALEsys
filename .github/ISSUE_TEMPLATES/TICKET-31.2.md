---
name: TICKET-31.2
about: Literature Explorer Panel
title: "feat(31.2): LiteratureExplorer - navegador de documentos ingeridos con filtros"
labels: fase31, research-layout
assignees: ''
---

## Descripción
Panel para explorar y filtrar documentos provenientes del pipeline de ingesta.

## Tareas
- [ ] Tree view: Colección → Documento → Capítulos
- [ ] Filtros: tema, fecha ingesta, fuente PDF, tiene citas
- [ ] Búsqueda full-text en contenido ingerido
- [ ] Preview de capítulo al click (markdown renderizado)
- [ ] Drag-and-drop capítulo a `NotesPanel` o `SynthesisPanel`
- [ ] Integración API: `GET /api/v1/ingestion/history`, `GET /api/v1/search/advanced`
- [ ] Paginación virtual para 1000+ documentos

## Archivos
- `webui/src/layouts/ResearchLayout/panels/LiteratureExplorer.tsx`
- `webui/src/layouts/ResearchLayout/hooks/useLiteratureSearch.ts`
- `webui/src/api/literature.ts` (nuevo)

## Labels
fase31, research-layout, literature, priority:high
