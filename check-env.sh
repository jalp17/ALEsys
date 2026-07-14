#!/usr/bin/env bash
# ALEsys - Validación de entorno antes de ejecutar pipeline
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=== ALEsys - Validación de Entorno ==="
echo ""

ERRORS=0
WARNINGS=0

# Función para validar
check() {
    local desc="$1"
    local cmd="$2"
    if eval "$cmd" >/dev/null 2>&1; then
        echo -e "  ${GREEN}✓${NC} $desc"
    else
        echo -e "  ${RED}✗${NC} $desc"
        ((ERRORS++))
    fi
}

warn() {
    local desc="$1"
    local cmd="$2"
    if eval "$cmd" >/dev/null 2>&1; then
        echo -e "  ${GREEN}✓${NC} $desc"
    else
        echo -e "  ${YELLOW}⚠${NC} $desc"
        ((WARNINGS++))
    fi
}

# 1. Python
check "Python 3.10+" "python3 --version | grep -E '3\.(1[0-9]|[2-9][0-9])'"

# 2. pip
check "pip disponible" "pip3 --version 2>/dev/null || pip --version 2>/dev/null"

# 3. Dependencias Python
check "psycopg[binary]" "python3 -c 'import psycopg' 2>/dev/null"
check "sentence-transformers" "python3 -c 'import sentence_transformers' 2>/dev/null"
check "httpx" "python3 -c 'import httpx' 2>/dev/null"
check "ddgs" "python3 -c 'import ddgs' 2>/dev/null"
check "rich" "python3 -c 'import rich' 2>/dev/null"

# 4. Variables de entorno críticas
echo ""
echo "--- Variables de Entorno ---"
if [[ -n "${OPENROUTER_API_KEY:-}" ]]; then
    echo -e "  ${GREEN}✓${NC} OPENROUTER_API_KEY configurada (${#OPENROUTER_API_KEY} chars)"
else
    echo -e "  ${RED}✗${NC} OPENROUTER_API_KEY no configurada"
    ((ERRORS++))
fi

# 5. PostgreSQL
echo ""
echo "--- PostgreSQL ---"
PGHOST="${PGHOST:-localhost}"
PGPORT="${PGPORT:-5432}"
PGUSER="${PGUSER:-alesys}"
PGPASSWORD="${PGPASSWORD:-alesys}"
PGDATABASE="${PGDATABASE:-alesys}"

check "Conexión a PostgreSQL" "python3 -c '
import psycopg
conn = psycopg.connect(
    host=\"'$PGHOST'\", port='$PGPORT', dbname=\"'$PGDATABASE'\",
    user=\"'$PGUSER'\", password=\"'$PGPASSWORD'\", connect_timeout=3
)
conn.close()
' 2>/dev/null"

# 6. pgvector extension
if python3 -c "
import psycopg
conn = psycopg.connect(
    host=\"'$PGHOST'\", port='$PGPORT', dbname=\"'$PGDATABASE'\",
    user=\"'$PGUSER'\", password=\"'$PGPASSWORD'\", connect_timeout=3
)
cur = conn.cursor()
cur.execute('SELECT 1 FROM pg_extension WHERE extname = \"vector\"')
result = cur.fetchone()
conn.close()
exit(0 if result else 1)
" 2>/dev/null; then
    echo -e "  ${GREEN}✓${NC} Extensión pgvector instalada"
else
    echo -e "  ${RED}✗${NC} Extensión pgvector NO instalada"
    ((ERRORS++))
fi

# 7. Directorio de libros
echo ""
echo "--- Directorio de Libros ---"
BOOKS_DIR="${BOOKS_DIR:-/home/jesus/knowledge_database/biblioteca_ia_rag/libros_ext4/books/}"
if [[ -d "$BOOKS_DIR" ]]; then
    COUNT=$(find "$BOOKS_DIR" -name "*.md" -type f | wc -l)
    echo -e "  ${GREEN}✓${NC} Directorio existe: $BOOKS_DIR ($COUNT archivos .md)"
else
    echo -e "  ${YELLOW}⚠${NC} Directorio no encontrado: $BOOKS_DIR"
    ((WARNINGS++))
fi

# 8. Módulos ALEsys
echo ""
echo "--- Módulos ALEsys ---"
for mod in config db_manager embedder extractor pipeline test_queries main gui; do
    check "Módulo $mod" "python3 -c 'import $mod' 2>/dev/null"
done

# Resumen
echo ""
echo "=== Resumen ==="
if [[ $ERRORS -eq 0 && $WARNINGS -eq 0 ]]; then
    echo -e "${GREEN}✓ Entorno listo para ejecutar ALEsys${NC}"
    exit 0
elif [[ $ERRORS -eq 0 ]]; then
    echo -e "${YELLOW}⚠ Entorno funcional con $WARNINGS advertencia(s)${NC}"
    exit 0
else
    echo -e "${RED}✗ $ERRORS error(es) crítico(s), $WARNINGS advertencia(s)${NC}"
    echo ""
    echo "Soluciona los errores antes de ejecutar la pipeline."
    exit 1
fi