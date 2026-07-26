#!/usr/bin/env bash
set -euo pipefail

# benchmark-ingestion.sh
# Ejecuta la suite de benchmarks del pipeline de ingesta.

set -a
source .env
set +a

echo "=== ALEsys - Benchmark Ingesta ==="
echo "Papers de prueba:"
ls -1 tests/fixtures/*.pdf 2>/dev/null || echo "(no fixtures encontrados)"

echo ""
echo "=== Ejecutando cargo bench ==="
cargo bench -p alesys-core ingestion_bench

echo ""
echo "=== Ejecutando e2e tests ==="
python3 tests/e2e/ingestion_test.py

echo ""
echo "=== Benchmark completado ==="
echo "Resultados guardados en target/criterion/"
