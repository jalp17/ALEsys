---
name: TICKET-31.5
about: Notes & Annotations Panel
title: "feat(31.5): NotesPanel - anotaciones vinculadas a capítulos y citas"
labels: fase31, research-layout
assignees: ''
---

## Descripción
Sistema de notas estructuradas vinculadas a contenido ingerido.

## Tareas
- [ ] Nota tipada: `Highlight`, `Comment`, `Question`, `Todo`, `Insight`
- [ ] Anclaje: nota vinculada a `chapter_id` + offset (texto seleccionado) o `citation_id`
- [ ] Tags personalizados + colores
- [ ] Búsqueda/filtro notas por tag, tipo, documento, fecha
- [ ] Export notas como markdown estructurado
- [ ] Integración: click nota → navega a capítulo en `LiteraturePanel`
- [ ] Persistencia en `notes` table (nueva migración)

## Archivos
- `webui/src/layouts/ResearchLayout/panels/NotesPanel.tsx`
- `webui/src/layouts/ResearchLayout/hooks/useNotes.ts`
- `crates/core/migrations/20260726_create_research_notes.sql` (nuevo)
- `crates/api/src/handlers_research.rs` (nuevo)
- `crates/core/src/research/` (nuevo módulo)

## Labels
fase31, research-layout, annotations, priority:medium
