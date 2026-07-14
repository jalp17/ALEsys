import logging
from typing import Any, Optional

from rich.console import Console
from rich.table import Table

from config import EMBEDDING
from db_manager import DatabaseManager
from embedder import Embedder
from extractor import Extractor

logger = logging.getLogger(__name__)
console = Console()


def vector_search(query: str, top_k: int = 5) -> list[dict[str, Any]]:
    embedder = Embedder()
    db = DatabaseManager()
    db.initialize_tables()

    vector = embedder.encode(query)
    cur = db.cursor
    cur.execute(
        """
        SELECT f.id, f.content, f.chunk_index, d.filename,
               1 - (f.embedding <=> %s::vector) AS similarity
        FROM fragmentos f
        JOIN documentos d ON d.id = f.documento_id
        ORDER BY f.embedding <=> %s::vector
        LIMIT %s
        """,
        (vector, vector, top_k),
    )
    rows = cur.fetchall()
    return [
        {
            "id": r["id"],
            "content": r["content"][:200],
            "chunk_index": r["chunk_index"],
            "filename": r["filename"],
            "similarity": round(r["similarity"], 4),
        }
        for r in rows
    ]


def graph_search(entity_name: str, limit: int = 50) -> dict[str, Any]:
    db = DatabaseManager()
    db.initialize_tables()
    cur = db.cursor

    cur.execute(
        "SELECT id, name, type, metadata FROM entidades WHERE name ILIKE %s LIMIT %s",
        (f"%{entity_name}%", limit),
    )
    entities = cur.fetchall()

    results = {"entities": [], "relations": []}
    for ent in entities:
        results["entities"].append({
            "id": ent["id"],
            "name": ent["name"],
            "type": ent["type"],
        })
        cur.execute(
            """
            SELECT r.relation_type, e2.name AS target_name, e2.type AS target_type
            FROM relaciones r
            JOIN entidades e2 ON e2.id = r.target_entity_id
            WHERE r.source_entity_id = %s
            """,
            (ent["id"],),
        )
        for rel in cur.fetchall():
            results["relations"].append({
                "source": ent["name"],
                "relation": rel["relation_type"],
                "target": rel["target_name"],
            })
    return results


def hybrid_search(query: str, top_k: int = 5) -> list[dict[str, Any]]:
    fragments = vector_search(query, top_k)
    db = DatabaseManager()
    cur = db.cursor

    for frag in fragments:
        cur.execute(
            """
            SELECT e.name, e.type
            FROM entidades e
            WHERE e.fragmento_id = %s
            """,
            (frag["id"],),
        )
        frag["entities"] = [{"name": r["name"], "type": r["type"]} for r in cur.fetchall()]
    return fragments


def ask(question: str, top_k: int = 5) -> str:
    fragments = hybrid_search(question, top_k)

    if not fragments:
        return "No se encontró información relevante en la base de datos."

    context_parts = []
    for i, f in enumerate(fragments, 1):
        context_parts.append(f"--- Fragmento {i} (archivo: {f['filename']}, similitud: {f['similarity']}) ---")
        context_parts.append(f["content"])
        if f.get("entities"):
            context_parts.append("Entidades: " + ", ".join(e["name"] for e in f["entities"]))

    context = "\n\n".join(context_parts)
    extractor = Extractor()
    result = extractor.answer(question, context)
    extractor.close()
    return result


def show_results(results: list[dict[str, Any]]) -> None:
    if not results:
        console.print("[yellow]Sin resultados[/yellow]")
        return
    table = Table(title="Resultados de búsqueda")
    table.add_column("Archivo", style="cyan")
    table.add_column("Similitud", style="green")
    table.add_column("Contenido", style="white")
    for r in results:
        table.add_row(r["filename"], str(r["similarity"]), r["content"])
    console.print(table)


if __name__ == "__main__":
    import sys

    logging.basicConfig(level=logging.WARNING)
    if len(sys.argv) < 2:
        print("Uso: python test_queries.py <consulta> [--graph <entidad>] [--hybrid]")
        sys.exit(1)

    query = sys.argv[1]
    if "--graph" in sys.argv:
        idx = sys.argv.index("--graph")
        name = sys.argv[idx + 1] if idx + 1 < len(sys.argv) else query
        results = graph_search(name)
        console.print(results)
    elif "--hybrid" in sys.argv:
        results = hybrid_search(query)
        show_results(results)
    else:
        results = vector_search(query)
        show_results(results)
