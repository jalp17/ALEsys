#!/usr/bin/env python3
"""
main.py — CLI principal de ALEsys.

Subcomandos:
    init  <nombre> <ruta>    — Crea un nuevo proyecto
    index <nombre>           — Indexa el código del proyecto
    chat  <nombre>           — Inicia chat interactivo con el proyecto
    list                     — Lista proyectos disponibles
    info  <nombre>           — Muestra información del proyecto

Ejemplo:
    python main.py init mi_app /home/user/codigo/mi_app
    python main.py index mi_app
    python main.py chat mi_app
"""

import argparse
import json
import logging
import sys
from pathlib import Path

# Setup logging global ANTES de importar módulos
def setup_logging(verbose: bool = False) -> None:
    """Configura logging con formato detallado y colores."""
    level = logging.DEBUG if verbose else logging.INFO
    
    # Formato con timestamp y módulo
    fmt = "%(asctime)s │ %(name)-28s │ %(levelname)-7s │ %(message)s"
    date_fmt = "%H:%M:%S"
    
    logging.basicConfig(
        level=level,
        format=fmt,
        datefmt=date_fmt,
        handlers=[logging.StreamHandler(sys.stderr)],
    )
    
    # Silenciar logs verbosos de librerías externas
    logging.getLogger("sentence_transformers").setLevel(logging.WARNING)
    logging.getLogger("transformers").setLevel(logging.WARNING)
    logging.getLogger("torch").setLevel(logging.WARNING)
    logging.getLogger("urllib3").setLevel(logging.WARNING)
    logging.getLogger("httpx").setLevel(logging.WARNING)


BASE_DIR = Path(__file__).resolve().parent
PROJECTS_DIR = BASE_DIR / "projects"


def cmd_init(args) -> None:
    """Crea un nuevo proyecto con su config.json."""
    from rich.console import Console
    console = Console()
    
    project_dir = PROJECTS_DIR / args.name
    config_path = project_dir / "config.json"
    vector_db_dir = project_dir / "vector_db"
    
    # Verificar que la ruta fuente existe
    source_path = Path(args.path).resolve()
    if not source_path.exists():
        console.print(f"[red]Error: Ruta no encontrada: {source_path}[/red]")
        sys.exit(1)
    
    # Crear estructura
    project_dir.mkdir(parents=True, exist_ok=True)
    vector_db_dir.mkdir(parents=True, exist_ok=True)
    
    # Detectar lenguaje dominante
    language = args.language or _detect_language(source_path)
    
    config = {
        "project_name": args.name,
        "source_path": str(source_path),
        "language": language,
        "extensions": [
            ".py", ".js", ".ts", ".jsx", ".tsx", ".java", ".cpp", ".c",
            ".h", ".hpp", ".cs", ".go", ".rs", ".rb", ".php",
            ".html", ".css", ".scss", ".vue", ".svelte",
            ".json", ".yaml", ".yml", ".toml",
            ".md", ".txt",
            ".sh", ".bash", ".sql",
        ],
        "exclude_dirs": [
            "node_modules", ".git", "__pycache__", "venv", ".venv",
            ".idea", ".vscode", "dist", "build", ".next", "target",
            "vendor", ".tox", ".mypy_cache", ".pytest_cache",
            ".eggs", "*.egg-info",
        ],
        "exclude_files": [
            "*.pyc", "*.pyo", "*.lock", "*.log",
            "*.min.js", "*.min.css", "*.map",
            "*.wasm", "*.so", "*.dll", "*.exe",
            "*.jpg", "*.png", "*.gif", "*.svg", "*.ico",
            "*.zip", "*.tar", "*.gz",
        ],
        "chunk_size": 1500,
        "chunk_overlap": 200,
        "max_file_size_kb": 500,
    }
    
    # Guardar (no sobrescribir si ya existe, a menos que --force)
    if config_path.exists() and not args.force:
        console.print(
            f"[yellow]Proyecto '{args.name}' ya existe.[/yellow]\n"
            f"Usa --force para sobrescribir la configuración."
        )
        return
    
    with open(config_path, "w", encoding="utf-8") as f:
        json.dump(config, f, ensure_ascii=False, indent=2)
    
    console.print(f"[green]✓ Proyecto creado: {args.name}[/green]")
    console.print(f"  Configuración: {config_path}")
    console.print(f"  Código fuente: {source_path}")
    console.print(f"  Lenguaje detectado: {language}")
    console.print(f"\n  Siguiente paso: [bold]python main.py index {args.name}[/bold]")


def cmd_index(args) -> None:
    """Indexa el código fuente de un proyecto."""
    from core.indexer import ProjectIndexer
    
    indexer = ProjectIndexer(
        project_name=args.name,
        embedding_model=args.embedding_model,
        parallel_load=not args.no_parallel,
        context_size=args.context_size,
    )
    if args.watch:
        indexer.watch_and_index(skip_summaries=args.skip_summaries)
    else:
        indexer.run(skip_summaries=args.skip_summaries)


def cmd_chat(args) -> None:
    """Inicia chat interactivo con un proyecto."""
    from core.chat_agent import ChatAgent
    
    agent = ChatAgent(
        project_name=args.name,
        model_name=args.model,
        enable_web_search=not args.no_web,
        parallel_load=not args.no_parallel,
        context_size=args.context_size,
    )
    agent.start_chat()


def cmd_list(args) -> None:
    """Lista todos los proyectos disponibles."""
    from rich.console import Console
    from rich.table import Table
    
    console = Console()
    
    if not PROJECTS_DIR.exists():
        console.print("[yellow]No hay proyectos creados aún.[/yellow]")
        return
    
    projects = [d for d in PROJECTS_DIR.iterdir() if d.is_dir()]
    if not projects:
        console.print("[yellow]No hay proyectos creados aún.[/yellow]")
        return
    
    table = Table(title="Proyectos Disponibles")
    table.add_column("Nombre", style="cyan bold")
    table.add_column("Lenguaje", style="green")
    table.add_column("Indexado", style="yellow")
    table.add_column("Vectores", justify="right")
    table.add_column("Ruta Fuente")
    
    for proj_dir in sorted(projects):
        config_path = proj_dir / "config.json"
        index_info_path = proj_dir / "vector_db" / "index_info.json"
        
        name = proj_dir.name
        language = "?"
        indexed = "No"
        vectors = "-"
        source = "?"
        
        if config_path.exists():
            try:
                with open(config_path) as f:
                    cfg = json.load(f)
                language = cfg.get("language", "?")
                source = cfg.get("source_path", "?")
                # Truncar ruta larga
                if len(source) > 40:
                    source = "..." + source[-37:]
            except Exception:
                pass
        
        if index_info_path.exists():
            try:
                with open(index_info_path) as f:
                    info = json.load(f)
                indexed = info.get("indexed_at", "Sí")
                vectors = str(info.get("num_vectors", "?"))
            except Exception:
                indexed = "Sí"
        
        table.add_row(name, language, indexed, vectors, source)
    
    console.print(table)


def cmd_info(args) -> None:
    """Muestra información detallada de un proyecto."""
    from rich.console import Console
    from rich.panel import Panel
    
    console = Console()
    
    project_dir = PROJECTS_DIR / args.name
    config_path = project_dir / "config.json"
    info_path = project_dir / "vector_db" / "index_info.json"
    
    if not config_path.exists():
        console.print(f"[red]Proyecto '{args.name}' no encontrado.[/red]")
        return
    
    with open(config_path) as f:
        config = json.load(f)
    
    info_text = f"""[bold]Proyecto:[/bold] {config.get('project_name', args.name)}
[bold]Ruta fuente:[/bold] {config.get('source_path', '?')}
[bold]Lenguaje:[/bold] {config.get('language', '?')}
[bold]Chunk size:[/bold] {config.get('chunk_size', '?')} chars
[bold]Overlap:[/bold] {config.get('chunk_overlap', '?')} chars"""
    
    if info_path.exists():
        with open(info_path) as f:
            info = json.load(f)
        info_text += f"""

[bold]═══ Índice ═══[/bold]
[bold]Vectores:[/bold] {info.get('num_vectors', '?')}
[bold]Dimensión:[/bold] {info.get('embedding_dim', '?')}
[bold]Modelo embeddings:[/bold] {info.get('embedding_model', '?')}
[bold]Modelo analista:[/bold] {info.get('analyst_model', '?')}
[bold]Indexado:[/bold] {info.get('indexed_at', '?')}"""
        
        stats = info.get("stats", {})
        if stats:
            info_text += f"""

[bold]═══ Estadísticas ═══[/bold]
[bold]Archivos escaneados:[/bold] {stats.get('files_scanned', '?')}
[bold]Archivos indexados:[/bold] {stats.get('files_indexed', '?')}
[bold]Chunks creados:[/bold] {stats.get('chunks_created', '?')}
[bold]Tiempo total:[/bold] {stats.get('total_time_s', 0):.1f}s"""
    
    console.print(Panel(info_text, title=f"Info: {args.name}", border_style="cyan"))


def _detect_language(source_path: Path) -> str:
    """Detecta el lenguaje dominante basándose en la extensión más común."""
    ext_map = {
        ".py": "python", ".js": "javascript", ".ts": "typescript",
        ".java": "java", ".cpp": "c++", ".c": "c", ".cs": "c#",
        ".go": "go", ".rs": "rust", ".rb": "ruby", ".php": "php",
        ".swift": "swift", ".kt": "kotlin", ".scala": "scala",
        ".lua": "lua", ".r": "r", ".R": "r",
        ".html": "html", ".css": "css", ".vue": "vue",
    }
    
    counts = {}
    try:
        for fpath in source_path.rglob("*"):
            if fpath.is_file():
                ext = fpath.suffix.lower()
                if ext in ext_map:
                    lang = ext_map[ext]
                    counts[lang] = counts.get(lang, 0) + 1
    except Exception:
        pass
    
    if counts:
        return max(counts, key=counts.get)
    return "unknown"


def main():
    parser = argparse.ArgumentParser(
        prog="ALEsys",
        description="ALEsys — Asistente de Desarrollo Multi-Proyecto con IA Local (RAG)",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Ejemplos:
  python main.py init mi_app /ruta/al/código
  python main.py index mi_app
  python main.py chat mi_app
  python main.py list
  python main.py info mi_app
        """,
    )
    parser.add_argument("-v", "--verbose", action="store_true", help="Logs detallados (DEBUG)")
    
    subparsers = parser.add_subparsers(dest="command", required=True)
    
    # init
    p_init = subparsers.add_parser("init", help="Crear nuevo proyecto")
    p_init.add_argument("name", help="Nombre del proyecto")
    p_init.add_argument("path", help="Ruta al directorio del código fuente")
    p_init.add_argument("-l", "--language", help="Lenguaje principal (auto-detectado si no se indica)")
    p_init.add_argument("-f", "--force", action="store_true", help="Sobrescribir config existente")
    p_init.set_defaults(func=cmd_init)
    
    # index
    p_index = subparsers.add_parser("index", help="Indexar código del proyecto")
    p_index.add_argument("name", help="Nombre del proyecto")
    p_index.add_argument(
        "--skip-summaries", action="store_true",
        help="Omitir resúmenes LLM (más rápido, sin GPU)"
    )
    p_index.add_argument(
        "--embedding-model", default="imocha-ai-org/ssf-skill-extractor",
        help="Modelo de embeddings (default: ssf-skill-extractor)"
    )
    p_index.add_argument(
        "--context-size", type=int, default=2048,
        help="Cantidad de contexto (n_ctx) a pasar al LLM analista"
    )
    p_index.add_argument(
        "--no-parallel", action="store_true",
        help="Forzar carga secuencial de modelos en lugar de paralela"
    )
    p_index.add_argument("--watch", action="store_true", help="Vigilar la carpeta fuente y reindexar al detectar cambios")
    p_index.set_defaults(func=cmd_index)
    
    # chat
    p_chat = subparsers.add_parser("chat", help="Chat interactivo con el proyecto")
    p_chat.add_argument("name", help="Nombre del proyecto")
    p_chat.add_argument("-m", "--model", help="Modelo conversacional GGUF específico")
    p_chat.add_argument("--no-web", action="store_true", help="Desactivar búsqueda web")
    p_chat.add_argument(
        "--context-size", type=int, default=4096,
        help="Máximo de tokens de contexto para el modelo conversacional"
    )
    p_chat.add_argument(
        "--no-parallel", action="store_true",
        help="Deshabilitar precarga paralela de modelos en chat"
    )
    p_chat.set_defaults(func=cmd_chat)
    
    # list
    p_list = subparsers.add_parser("list", help="Listar proyectos")
    p_list.set_defaults(func=cmd_list)
    
    # info
    p_info = subparsers.add_parser("info", help="Info de un proyecto")
    p_info.add_argument("name", help="Nombre del proyecto")
    p_info.set_defaults(func=cmd_info)
    
    args = parser.parse_args()
    setup_logging(args.verbose)
    
    try:
        args.func(args)
    except KeyboardInterrupt:
        print("\n\nOperación cancelada por el usuario.")
        sys.exit(0)
    except Exception as e:
        logging.getLogger("ALEsys").error(f"Error: {e}", exc_info=args.verbose)
        sys.exit(1)


if __name__ == "__main__":
    main()
