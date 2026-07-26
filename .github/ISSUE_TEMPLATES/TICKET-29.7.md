## Descripción
Panel de ingesta en webui: drag-drop, opciones, progress bars, history.

## Componentes
- `IngestionPanel.tsx`: drag-drop PDF, selector topic, opciones OCR/formulas
- `BatchIngestion.tsx`: multi-file, progress bars, queue management
- `IngestionHistory.tsx`: lista jobs, estado, link a output dir

## Tareas
- [ ] Drag-drop PDF files
- [ ] Selector topic + opciones avanzadas
- [ ] WebSocket connection para progress real-time
- [ ] Integración ResearchLayout (Fase 31)

## Archivos
- `webui/src/pages/ingestion/IngestionPanel.tsx`
- `webui/src/pages/ingestion/BatchIngestion.tsx`
- `webui/src/pages/ingestion/IngestionHistory.tsx`

## Labels
fase29, frontend, ui, priority:medium