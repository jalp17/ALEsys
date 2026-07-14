#!/usr/bin/env python3
import argparse
import logging
import signal
import sys

from rich.console import Console
from rich.logging import RichHandler

from db_manager import DatabaseManager
from pipeline import Pipeline

console = Console()

# Signal handling para cleanup ordenado
def _signal_handler(signum, frame):
    logger = logging.getLogger("ALEsys")
    logger.info("Interrupción recibida (Ctrl+C)")
    console.print("\n[yellow]Interrumpido por usuario[/yellow]")
    sys.exit(130)

signal.signal(signal.SIGINT, _signal_handler)
signal.signal(signal.SIGTERM, _signal_handler)


def setup_logging(verbose: bool = False) -> None:
    level = logging.DEBUG if verbose else logging.INFO
    logging.basicConfig(
        level=level,
        format="%(message)s",
        datefmt="%H:%M:%S",
        handlers=[RichHandler(rich_tracebacks=True)],
    )
    for name in ("sentence_transformers", "httpx", "urllib3"):
        logging.getLogger(name).setLevel(logging.WARNING)


def cmd_db_init(args) -> None:
    db = DatabaseManager()
    db.initialize_tables()
    console.print("[green]✓ Tablas inicializadas correctamente[/green]")


def cmd_db_drop(args) -> None:
    if not args.force:
        confirm = input("¿Eliminar todas las tablas? (yes/no): ")
        if confirm.lower() != "yes":
            console.print("[yellow]Cancelado[/yellow]")
            return
    db = DatabaseManager()
    db.drop_tables()
    console.print("[green]✓ Tablas eliminadas[/green]")


def cmd_run(args) -> None:
    pipeline = Pipeline(
        books_dir=args.input,
        chunk_size=args.chunk_size,
        chunk_overlap=args.chunk_overlap,
        dry_run=args.dry_run,
    )
    pipeline.run()


def cmd_query(args) -> None:
    from test_queries import hybrid_search, vector_search

    if args.graph:
        from test_queries import graph_search
        results = graph_search(args.graph, limit=args.top_k)
        console.print(results)
    elif args.hybrid:
        results = hybrid_search(args.query, args.top_k)
    else:
        results = vector_search(args.query, args.top_k)

    if isinstance(results, list):
        from test_queries import show_results
        show_results(results)


def cmd_ask(args) -> None:
    from test_queries import ask
    response = ask(args.question, args.top_k)
    console.print(response)


def cmd_list(args) -> None:
    db = DatabaseManager()
    db.initialize_tables()
    cur = db.cursor
    cur.execute(
        """
        SELECT d.id, d.filename, d.created_at,
               COUNT(f.id) AS fragmentos,
               COUNT(DISTINCT e.id) AS entidades
        FROM documentos d
        LEFT JOIN fragmentos f ON f.documento_id = d.id
        LEFT JOIN entidades e ON e.fragmento_id = f.id
        GROUP BY d.id
        ORDER BY d.created_at DESC
        """
    )
    rows = cur.fetchall()
    if not rows:
        console.print("[yellow]No hay documentos indexados[/yellow]")
        return

    from rich.table import Table
    table = Table(title="Documentos indexados")
    table.add_column("ID", style="cyan")
    table.add_column("Archivo", style="green")
    table.add_column("Fragmentos", justify="right")
    table.add_column("Entidades", justify="right")
    table.add_column("Creado", style="dim")
    for r in rows:
        table.add_row(str(r["id"]), r["filename"], str(r["fragmentos"]), str(r["entidades"]), str(r["created_at"])[:19])
    console.print(table)


def main() -> None:
    parser = argparse.ArgumentParser(
        prog="ALEsys",
        description="ALEsys — GraphRAG-PG: Pipeline de ingesta híbrida sobre PostgreSQL",
    )
    parser.add_argument("-v", "--verbose", action="store_true")

    sub = parser.add_subparsers(dest="command", required=True)

    p_db_init = sub.add_parser("db-init", help="Inicializar tablas en PostgreSQL")
    p_db_init.set_defaults(func=cmd_db_init)

    p_db_drop = sub.add_parser("db-drop", help="Eliminar tablas")
    p_db_drop.add_argument("-f", "--force", action="store_true", help="Saltar confirmación")
    p_db_drop.set_defaults(func=cmd_db_drop)

    p_run = sub.add_parser("run", help="Ejecutar pipeline de ingesta")
    p_run.add_argument("--input", help="Directorio de libros (default: BOOKS_DIR)")
    p_run.add_argument("--chunk-size", type=int, default=1000)
    p_run.add_argument("--chunk-overlap", type=int, default=200)
    p_run.add_argument("--dry-run", action="store_true", help="Solo previsualizar archivos, sin indexar")
    p_run.set_defaults(func=cmd_run)

    p_query = sub.add_parser("query", help="Buscar en la base de datos")
    p_query.add_argument("query", nargs="?", default="", help="Texto de búsqueda")
    p_query.add_argument("--graph", help="Buscar entidad en el grafo")
    p_query.add_argument("--hybrid", action="store_true", help="Búsqueda híbrida (vector + grafo)")
    p_query.add_argument("--top-k", type=int, default=5)
    p_query.set_defaults(func=cmd_query)

    p_ask = sub.add_parser("ask", help="Preguntar con contexto RAG")
    p_ask.add_argument("question", help="Pregunta sobre los documentos indexados")
    p_ask.add_argument("--top-k", type=int, default=5)
    p_ask.set_defaults(func=cmd_ask)

    p_list = sub.add_parser("list", help="Listar documentos indexados")
    p_list.set_defaults(func=cmd_list)

    args = parser.parse_args()
    setup_logging(args.verbose)

    try:
        args.func(args)
    except KeyboardInterrupt:
        print("\nOperación cancelada.")
        sys.exit(0)
    except Exception as e:
        logger = logging.getLogger("ALEsys")
        logger.error("Error: %s", e, exc_info=args.verbose)
        sys.exit(1)


if __name__ == "__main__":
    main()
