---
name: TICKET-31.8
about: Integration Tests + E2E
title: "test(31.8): Tests E2E ResearchLayout + integración completa"
labels: fase31, research-layout
assignees: ''
---

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
