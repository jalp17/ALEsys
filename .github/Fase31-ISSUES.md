---
fecha: 2026-07-26
tipo: documentacion
proyecto: alesys-research-assistant
area: desarrollo-web
etiquetas: [alesys, fase31, github-issues, tickets, planificado]
---

# Fase 31: ResearchLayout Integration

Archivo con plantillas para issues de Fase 31. **Fase planificada - no implementada.**

Contexto: Integración del layout de investigación (`ResearchLayout`) con el pipeline de ingesta (Fase 29) y bibliografía (Fase 30). ResearchLayout es un workspace visual para análisis de literatura, revisión sistemática y síntesis de conocimiento.

---

## Cambios Recientes

- **2026-07-27:** Upgrade de React 18.3.1 → 19.2.8 para resolver mismatch de versiones en vitest. Tests unitarios de `ResearchLayout` pasan (3/3).

---

## Issues Template

### TICKET-31.1: ResearchLayout Skeleton + Layout Manager

**Título:** feat(31.1): ResearchLayout skeleton con layout manager y paneles base

**Body:**
```markdown
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
```

---

### TICKET-31.2: Literature Explorer Panel

**Título:** feat(31.2): LiteratureExplorer - navegador de documentos ingeridos con filtros

**Body:**
```markdown
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
```

---

### TICKET-31.3: Citation Network Visualizer

**Título:** feat(31.3): CitationNetwork - grafo interactivo de citas (Fase 30 + GraphRAG)

**Body:**
```markdown
## Descripción
Visualizador de red de citas usando datos de bibliografía (Fase 30) y GraphRAG.

## Tareas
- [ ] Grafo citas: nodos = papers, aristas = cita a / citado por
- [ ] Integración `bibliography_citations` + GraphRAG `fragmentos` edges
- [ ] Layout force-directed (cytoscape.js o d3-force)
- [ ] Filtros: por estilo cita, año, autor, cluster
- [ ] Click nodo → detalle en `CitationPanel` (metadata, abstract, DOI link)
- [ ] Export: PNG, GraphML, JSON
- [ ] Integración API: `GET /api/v1/bibliography/network`, `GET /api/v1/graphrag/neighbors`

## Archivos
- `webui/src/layouts/ResearchLayout/panels/CitationNetwork.tsx`
- `webui/src/layouts/ResearchLayout/hooks/useCitationGraph.ts`
- `crates/api/src/handlers_bibliography.rs` (nuevo endpoint network)
- `crates/api/src/handlers_graphrag.rs` (endpoint neighbors)

## Labels
fase31, research-layout, graphrag, visualization, priority:high
```

---

### TICKET-31.4: Synthesis Workspace

**Título:** feat(31.4): SynthesisWorkspace - editor colaborativo para revisión sistemática

**Body:**
```markdown
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
```

---

### TICKET-31.5: Notes & Annotations Panel

**Título:** feat(31.5): NotesPanel - anotaciones vinculadas a capítulos y citas

**Body:**
```markdown
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
```

---

### TICKET-31.6: Research Project Management

**Título:** feat(31.6): ResearchProject - gestión de proyectos de investigación multi-documento

**Body:**
```markdown
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
```

---

### TICKET-31.7: API Endpoints Research

**Título:** feat(31.7): API endpoints para ResearchLayout (CRUD projects, notes, network)

**Body:**
```markdown
## Descripción
Endpoints REST + WebSocket para funcionalidad de investigación.

## Endpoints
| Método | Endpoint | Descripción |
|--------|----------|-------------|
| GET    | `/api/v1/research/projects` | Listar proyectos |
| POST   | `/api/v1/research/projects` | Crear proyecto |
| GET    | `/api/v1/research/projects/:id` | Detalle proyecto |
| PUT    | `/api/v1/research/projects/:id` | Actualizar proyecto |
| DELETE | `/api/v1/research/projects/:id` | Eliminar proyecto |
| GET    | `/api/v1/research/projects/:id/stats` | Estadísticas |
| GET    | `/api/v1/research/projects/:id/export` | Export completo |
| GET    | `/api/v1/research/notes` | Listar notas (filtros) |
| POST   | `/api/v1/research/notes` | Crear nota |
| GET    | `/api/v1/research/network` | Grafo citas (nodos/aristas) |
| GET    | `/api/v1/research/literature/search` | Búsqueda literatura |
| WS     | `/ws/research/:projectId` | Sync real-time colaborativo |

## Tareas
- [ ] Handlers en `crates/api/src/handlers_research.rs`
- [ ] Rutas en `crates/api/src/routes.rs`
- [ ] Auth: JWT + RBAC (`research:read`, `research:write`)
- [ ] Rate limiting: 50 req/min
- [ ] Tests de integración

## Archivos
- `crates/api/src/handlers_research.rs`
- `crates/api/src/routes.rs` (update)
- `crates/api/tests/research_test.rs`

## Labels
fase31, api, research, priority:high
```

---

### TICKET-31.8: Integration Tests + E2E

**Título:** test(31.8): Tests E2E ResearchLayout + integración completa

**Body:**
```markdown
## Descripción
Suite de tests end-to-end para ResearchLayout.

## Tareas
- [ ] Test: crear proyecto → ingesta PDF → explorar literatura → crear notas → sintetizar → exportar
- [ ] Test: red de citas con 50+ papers
- [ ] Test: editor síntesis con 100+ citas insertadas
- [ ] Test: colaboración multi-usuario en proyecto compartido
- [ ] Test: export DOCX/PDF/LaTeX válido
- [ ] Benchmarks: carga grafo 5000 nodos, búsqueda 10k docs
- [ ] CI: `.github/workflows/research-test.yml`

## Archivos
- `tests/e2e/research_test.py`
- `benches/research_bench.rs`
- `.github/workflows/research-test.yml`

## Labels
fase31, testing, e2e, priority:medium
```

---

### TICKET-31.9: Documentation + Setup

**Título:** docs(31.9): Documentación ResearchLayout + setup

**Body:**
```markdown
## Descripción
Documentación de usuario y desarrollador para ResearchLayout.

## Tareas
- [ ] `docs/RESEARCH_LAYOUT.md`: arquitectura, API, componentes
- [ ] `docs/RESEARCH_WORKFLOW.md`: guía usuario (proyecto → ingesta → síntesis → export)
- [ ] Actualizar `README.md` con sección ResearchLayout
- [ ] Actualizar `CONTRIBUTING.md` con tickets Fase 31
- [ ] Ejemplos: `notebooks/research_workflow.ipynb`

## Archivos
- `docs/RESEARCH_LAYOUT.md`
- `docs/RESEARCH_WORKFLOW.md`
- `notebooks/research_workflow.ipynb`

## Labels
fase31, documentation, priority:low
```

---

## Dependencias entre Tickets

```
TICKET-31.1 (Layout Base)
    │
    ├──→ TICKET-31.2 (Literature Explorer) ──┐
    ├──→ TICKET-31.3 (Citation Network) ────┤──→ TICKET-31.4 (Synthesis)
    ├──→ TICKET-31.5 (Notes) ───────────────┘
    │
    ├──→ TICKET-31.6 (Project Management)
    │
    ├──→ TICKET-31.7 (API Endpoints) ────────┐
    │                                         │
    └─────────────────────────────────────────┘
                            │
                    TICKET-31.8 (Tests E2E)
                            │
                    TICKET-31.9 (Docs)
```

---

## Script de Creación Masiva (Opcional)

```bash
#!/bin/bash
# .github/scripts/create-fase31-issues.sh

for file in .github/ISSUE_TEMPLATES/TICKET-31.*.md; do
    title=$(grep "^# " "$file" | head -1 | sed 's/^# //')
    label=$(grep "^## Labels" "$file" | sed 's/## Labels//')
    gh issue create --title "$title" --body-file "$file" --label "$label"
done
```

---

**Tags:** #alesys #fase31 #github-issues #tickets #research-layout #planificado