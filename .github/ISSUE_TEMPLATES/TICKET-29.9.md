## Descripción
Suite de tests end-to-end para ingestion pipeline.

## Tareas
- [ ] Test suite: 10 papers variados (1-200 págs, fórmulas, tablas, scans)
- [ ] Metrics: latency, accuracy, memoria, GPU usage
- [ ] CI: GitHub Action `ingestion-test.yml` (self-hosted con GPU)
- [ ] Regression: golden files para organizer output
- [ ] Security: plugin sandbox escape attempts

## Archivos
- `tests/e2e/ingestion_test.ts`
- `benches/ingestion_bench.rs`
- `.github/workflows/ingestion-test.yml`

## Labels
fase29, testing, e2e, priority:medium