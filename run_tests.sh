#!/usr/bin/env bash
set -euo pipefail

CLEANUP_DB=false
if [ "${PGDATABASE:-}" = "" ]; then
    export PGDATABASE="alesys_test"
    CLEANUP_DB=true
fi

cleanup() {
    if [ "$CLEANUP_DB" = true ]; then
        echo ""
        echo "--- Cleanup ---"
        python3 -c "
from db_manager import DatabaseManager
try:
    db = DatabaseManager()
    db.drop_tables()
    print('  ✓ Tablas de prueba eliminadas')
except Exception:
    pass
" 2>/dev/null || true
    fi
    rm -rf /tmp/alesys_test_books 2>/dev/null || true
}
trap cleanup EXIT

echo "=== ALEsys GraphRAG-PG — Suite de validación ==="
echo "Base de datos: ${PGDATABASE}"

echo ""
echo "--- 1. Entorno ---"
python3 --version
pip3 --version 2>/dev/null || pip --version

echo ""
echo "--- 2. Dependencias ---"
pip3 install -r requirements.txt || true

echo ""
echo "--- 3. Sintaxis ---"
for mod in config.py db_manager.py embedder.py extractor.py pipeline.py test_queries.py main.py gui.py core/__init__.py core/chat_agent.py core/web_search.py; do
    python3 -m py_compile "$mod" 2>/dev/null \
        && echo "  ✓ $mod" \
        || echo "  ✗ $mod (ERROR)"
done

echo ""
echo "--- 4. Imports ---"
python3 -c "
from config import DB, EMBEDDING, OPENROUTER, PATHS, CHUNKING
from db_manager import DatabaseManager
from embedder import Embedder
from extractor import Extractor
from pipeline import Pipeline
from test_queries import vector_search, graph_search, hybrid_search
print('  ✓ Todos los imports correctos')
"

echo ""
echo "--- 5. Conexión PostgreSQL ---"
python3 -c "
from db_manager import DatabaseManager
db = DatabaseManager()
try:
    db.initialize_tables()
    print('  ✓ Conexión exitosa y tablas creadas')
except Exception as e:
    print(f'  ✗ Error: {e}')
    print('  Asegúrate de que PostgreSQL esté corriendo en localhost:5432')
    print('  Comando: docker start postgres_db')
"

echo ""
echo "--- 6. Embeddings ---"
python3 -c "
from embedder import get_embedder
e = get_embedder()
v = e.encode('prueba de embedding científico')
assert len(v) == 384, f'Dimensión incorrecta: {len(v)}'
print(f'  ✓ Embedding generado: {len(v)} dimensiones')
"

echo ""
echo "--- 7. Pipeline (modo prueba) ---"
SANDBOX="/tmp/alesys_test_books"
mkdir -p "$SANDBOX"
cat > "$SANDBOX/test.md" << 'EOF'
# Física Cuántica

La mecánica cuántica es una rama fundamental de la física.
El principio de incertidumbre de Heisenberg establece que
es imposible conocer simultáneamente la posición y el momento
de una partícula con precisión arbitraria.

## Ecuación de Schrödinger

La ecuación de Schrödinger describe cómo cambia el estado
cuántico de un sistema físico con el tiempo.
EOF

echo "  → Prueba 1: dry-run"
python3 -c "
from pipeline import Pipeline
p = Pipeline(books_dir='$SANDBOX', chunk_size=500, chunk_overlap=50, dry_run=True)
p.run()
print('  ✓ Dry-run ejecutado correctamente')
"

echo "  → Prueba 2: pipeline completa"
python3 -c "
from pipeline import Pipeline
p = Pipeline(books_dir='$SANDBOX', chunk_size=500, chunk_overlap=50)
p.run()
print('  ✓ Pipeline ejecutada correctamente')
"

echo "  → Prueba 3: re-ejecución (sin duplicados)"
python3 -c "
from db_manager import DatabaseManager
db = DatabaseManager()
cur = db.cursor
cur.execute('SELECT COUNT(*) AS c FROM fragmentos')
before = cur.fetchone()['c']
from pipeline import Pipeline
p = Pipeline(books_dir='$SANDBOX', chunk_size=500, chunk_overlap=50)
p.run()
cur.execute('SELECT COUNT(*) AS c FROM fragmentos')
after = cur.fetchone()['c']
assert before == after, f'Duplicados: {before} → {after}'
print(f'  ✓ Sin duplicados: {before} fragmentos')
"
rm -rf "$SANDBOX"

echo ""
echo "--- 8. Consultas ---"
python3 -c "
from test_queries import vector_search, graph_search
r = vector_search('mecánica cuántica', top_k=3)
print(f'  ✓ Búsqueda vectorial: {len(r)} resultados')
for item in r:
    print(f'    [{item[\"similarity\"]}] {item[\"filename\"]}')
"

python3 -c "
from test_queries import graph_search
r = graph_search('Heisenberg', limit=10)
print(f'  ✓ Búsqueda en grafo: {len(r[\"entities\"])} entidades, {len(r[\"relations\"])} relaciones')
for e in r['entities']:
    print(f'    {e[\"name\"]} ({e[\"type\"]})')
"

echo ""
echo "=== Validación completada ==="
