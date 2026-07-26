---
name: TICKET-31.3
about: Citation Network Visualizer
title: "feat(31.3): CitationNetwork - grafo interactivo de citas (Fase 30 + GraphRAG)"
labels: fase31, research-layout
assignees: ''
---

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
