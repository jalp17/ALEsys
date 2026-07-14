#!/usr/bin/env bash
# ALEsys — Script de validación de entorno
set -euo pipefail

OK=0
FAIL=0

check() {
    local desc="$1"
    local cmd="$2"
    if eval "$cmd" >/dev/null 2>&1; then
        echo "  ✓ $desc"
        OK=$((OK+1))
    else
        echo "  ✗ $desc"
        FAIL=$((FAIL+1))
    fi
}

echo "=== ALEsys — Verificación de entorno ==="
echo ""

# 1. Python
echo "--- Python ---"
check "Python 3.10+" "python3 -c 'import sys; assert sys.version_info >= (3, 10)'"

# 2. Dependencias Python
echo ""
echo "--- Dependencias Python ---"
check "psycopg"         "python3 -c 'import psycopg'"
check "httpx"           "python3 -c 'import httpx'"
check "rich"            "python3 -c 'import rich'"
check "dotenv"          "python3 -c 'import dotenv'"
check "sentence_transformers" "python3 -c 'import sentence_transformers'"

# 3. PostgreSQL
echo ""
echo "--- PostgreSQL ---"
PGHOST="${PGHOST:-localhost}"
PGPORT="${PGPORT:-5432}"
PGDATABASE="${PGDATABASE:-alesys}"
PGUSER="${PGUSER:-alesys}"
PGPASSWORD="${PGPASSWORD:-alesys}"

check "Conexión a PostgreSQL ($PGHOST:$PGPORT)" \
    "python3 -c \"import psycopg; conn = psycopg.connect(host='$PGHOST', port=$PGPORT, dbname='$PGDATABASE', user='$PGUSER', password='$PGPASSWORD', connect_timeout=5); conn.close()\""

# 4. pgvector
echo ""
echo "--- pgvector ---"
check "Extensión pgvector" \
    "python3 -c \"import psycopg; conn = psycopg.connect(host='$PGHOST', port=$PGPORT, dbname='$PGDATABASE', user='$PGUSER', password='$PGPASSWORD', connect_timeout=5); cur = conn.cursor(); cur.execute('SELECT 1 FROM pg_extension WHERE extname = %s', ('vector',)); assert cur.fetchone() is not None; conn.close()\""

# 5. Variable de entorno OPENROUTER_API_KEY
echo ""
echo "--- API Key ---"
if [ -n "${OPENROUTER_API_KEY:-}" ]; then
    echo "  ✓ OPENROUTER_API_KEY configurada (${#OPENROUTER_API_KEY} caracteres)"
    OK=$((OK+1))
else
    echo "  ✗ OPENROUTER_API_KEY no configurada"
    echo "    Exporta: export OPENROUTER_API_KEY=tu_clave"
    echo "    O crea un archivo .env con OPENROUTER_API_KEY=tu_clave"
    FAIL=$((FAIL+1))
fi

# 6. Archivo .env (opcional)
echo ""
echo "--- Archivo .env ---"
if [ -f .env ]; then
    echo "  ✓ Archivo .env encontrado"
    OK=$((OK+1))
else
    echo "  ⚠ Archivo .env no encontrado (opcional)"
fi

# 7. Disco
echo ""
echo "--- Almacenamiento ---"
AVAIL=$(df -BM . | awk 'NR==2 {print $4}' | tr -d 'M')
if [ "$AVAIL" -ge 500 ]; then
    echo "  ✓ Espacio disponible: ${AVAIL}MB"
    OK=$((OK+1))
else
    echo "  ✗ Espacio insuficiente: ${AVAIL}MB (mínimo 500MB)"
    FAIL=$((FAIL+1))
fi

# Resumen
echo ""
echo "=== Resultado: $OK éxitos, $FAIL fallos ==="
if [ "$FAIL" -gt 0 ]; then
    echo "Corrija los fallos antes de ejecutar ALEsys."
    exit 1
else
    echo "Entorno listo para usar ALEsys."
    exit 0
fi
