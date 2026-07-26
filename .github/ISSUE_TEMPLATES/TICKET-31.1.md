---
name: TICKET-31.1
about: ResearchLayout Skeleton + Layout Manager
title: "feat(31.1): ResearchLayout skeleton con layout manager y paneles base"
labels: fase31, research-layout
assignees: ''
---

## Descripción
Estructura base del layout de investigación con sistema de paneles arrastrables/redimensionables.

## Tareas
- [ ] Crear `webui/src/layouts/ResearchLayout/` estructura
- [ ] Implementar `PanelManager` con grid flexible (react-grid-layout o similar)
- [ ] Paneles base: `LiteraturePanel`, `CitationPanel`, `NotesPanel`, `SynthesisPanel`
- [ ] Persistencia de layout en localStorage / backend
- [ ] Toggle fullscreen por panel
- [ ] Tests unitarios de layout

## Archivos
- `webui/src/layouts/ResearchLayout/ResearchLayout.tsx`
- `webui/src/layouts/ResearchLayout/PanelManager.tsx`
- `webui/src/layouts/ResearchLayout/panels/LiteraturePanel.tsx`
- `webui/src/layouts/ResearchLayout/panels/CitationPanel.tsx`
- `webui/src/layouts/ResearchLayout/panels/NotesPanel.tsx`
- `webui/src/layouts/ResearchLayout/panels/SynthesisPanel.tsx`
- `webui/src/layouts/ResearchLayout/hooks/useLayoutPersistence.ts`

## Labels
fase31, research-layout, frontend, priority:high
