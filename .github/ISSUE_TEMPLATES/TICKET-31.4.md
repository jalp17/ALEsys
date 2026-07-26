---
name: TICKET-31.4
about: Synthesis Workspace
title: "feat(31.4): SynthesisWorkspace - editor colaborativo para revisión sistemática"
labels: fase31, research-layout
assignees: ''
---

## Descripción
Workspace para redactar síntesis/literature review con citas integradas.

## Tareas
- [ ] Editor markdown con toolbar (headings, listas, tablas, citas)
- [ ] Insertar cita desde `CitationPanel` → `@cite{key}` o `[[citation:key]]`
- [ ] Render inline de cita: tooltip con metadata completa al hover
- [ ] Bibliografía auto-generada al final (estilo configurable APA/MLA/Chicago/IEEE)
- [ ] Secciones predefinidas: Introducción, Métodos, Resultados, Discusión, Conclusiones
- [ ] Versionado local (historial undo/redo) + auto-save
- [ ] Export: Markdown, DOCX, PDF (pandoc), LaTeX
- [ ] Integración `CitationFormatter` (Fase 30) para bibliografía final

## Archivos
- `webui/src/layouts/ResearchLayout/panels/SynthesisWorkspace.tsx`
- `webui/src/layouts/ResearchLayout/components/CitationAutocomplete.tsx`
- `webui/src/layouts/ResearchLayout/hooks/useSynthesisEditor.ts`
- `webui/src/utils/export.ts` (nuevo)

## Labels
fase31, research-layout, editor, synthesis, priority:high
