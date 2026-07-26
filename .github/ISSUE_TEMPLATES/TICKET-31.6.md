---
name: TICKET-31.6
about: Research Project Management
title: "feat(31.6): ResearchProject - gestión de proyectos de investigación multi-documento"
labels: fase31, research-layout
assignees: ''
---

## Descripción
Contenedor de alto nivel para organizar trabajo de investigación.

## Tareas
- [ ] `ResearchProject`: nombre, descripción, fecha, owner, colaboradores
- [ ] Colección de documentos asociados (desde ingesta)
- [ ] Configuración de estilo cita global (APA/MLA/Chicago/IEEE)
- [ ] Dashboard: progreso, estadísticas (docs, citas, notas, palabras)
- [ ] Compartir proyecto: lectura / escritura (RBAC)
- [ ] Duplicar proyecto como plantilla
- [ ] Export proyecto completo (ZIP con markdown, bibliografía, notas)

## Archivos
- `webui/src/pages/research/ResearchProjectDashboard.tsx`
- `webui/src/pages/research/ResearchProjectSettings.tsx`
- `crates/core/migrations/20260726_create_research_projects.sql` (nuevo)
- `crates/core/src/research/project.rs` (nuevo)
- `crates/api/src/handlers_research.rs` (endpoints CRUD)

## Labels
fase31, research-layout, project-management, priority:medium
