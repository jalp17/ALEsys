import logging
import signal
import sys
import time
from pathlib import Path
from typing import Optional

from rich.console import Console
from rich.logging import RichHandler
from rich.progress import BarColumn, Progress, TextColumn, TimeElapsedColumn

from config import CHUNKING, PATHS, OPENROUTER
from db_manager import DatabaseManager
from embedder import Embedder
from extractor import Extractor

logger = logging.getLogger(__name__)
console = Console()

# Signal handling para cleanup ordenado
def _signal_handler(signum, frame):
    logger.info("Interrupción recibida (Ctrl+C), cerrando conexiones...")
    console.print("\n[yellow]Interrumpido por usuario. Limpiando...[/yellow]")
    sys.exit(130)

signal.signal(signal.SIGINT, _signal_handler)
signal.signal(signal.SIGTERM, _signal_handler)


def chunk_text(text: str, size: int, overlap: int) -> list[tuple[str, int]]:
    if overlap >= size:
        overlap = size // 2
    chunks: list[tuple[str, int]] = []
    start = 0
    index = 0
    while start < len(text):
        end = min(start + size, len(text))
        if end < len(text):
            newline = text.rfind("\n", start, end)
            if newline > start + size // 2:
                end = newline + 1
        chunk_text_ = text[start:end].strip()
        if chunk_text_:
            chunks.append((chunk_text_, index))
            index += 1
        start = end - overlap if end < len(text) else len(text)
    return chunks


class Pipeline:
    def __init__(
        self,
        books_dir: Optional[str] = None,
        chunk_size: Optional[int] = None,
        chunk_overlap: Optional[int] = None,
        dry_run: bool = False,
    ):
        self.books_dir = Path(books_dir or PATHS.books_dir)
        self.chunk_size = chunk_size or CHUNKING.size
        self.chunk_overlap = chunk_overlap or CHUNKING.overlap
        self.dry_run = dry_run
        self.db = DatabaseManager()
        self.embedder = Embedder()
        self.extractor = Extractor()

    def run(self) -> None:
        if not OPENROUTER.api_key:
            raise ValueError(
                "OPENROUTER_API_KEY no configurada. "
                "Exporta la variable de entorno OPENROUTER_API_KEY o configúrala en .env"
            )
        self.db.initialize_tables()
        md_files = sorted(self.books_dir.rglob("*.md"))
        if not md_files:
            logger.warning("No se encontraron archivos .md en %s", self.books_dir)
            return

        logger.info(
            "Procesando %d archivos desde %s%s",
            len(md_files), self.books_dir,
            " [DRY RUN]" if self.dry_run else "",
        )

        if self.dry_run:
            for md_path in md_files:
                text = md_path.read_text(encoding="utf-8", errors="replace")
                chunks = chunk_text(text, self.chunk_size, self.chunk_overlap)
                logger.info(
                    "  %s: %d bytes, %d fragmentos",
                    md_path.relative_to(self.books_dir), len(text), len(chunks),
                )
            logger.info("Dry run completado: %d archivos analizados", len(md_files))
            return

        total_chunks = 0
        total_entities = 0
        total_relations = 0
        start_time = time.monotonic()

        with Progress(
            TextColumn("[progress.description]{task.description}"),
            BarColumn(),
            TextColumn("{task.completed}/{task.total}"),
            TimeElapsedColumn(),
            console=console,
        ) as progress:
            task = progress.add_task("Indexando libros...", total=len(md_files))

            for md_path in md_files:
                try:
                    stats = self._process_file(md_path)
                    total_chunks += stats["chunks"]
                    total_entities += stats["entities"]
                    total_relations += stats["relations"]
                except Exception:
                    logger.exception("Error procesando %s", md_path)
                progress.advance(task)

        elapsed = time.monotonic() - start_time
        logger.info(
            "Pipeline completada: %d fragmentos, %d entidades, %d relaciones en %d archivos (%.1fs)",
            total_chunks,
            total_entities,
            total_relations,
            len(md_files),
            elapsed,
        )

        self.extractor.close()

    def _process_file(self, md_path: Path) -> dict[str, int]:
        rel_path = md_path.relative_to(self.books_dir)
        filename = str(rel_path)
        text = md_path.read_text(encoding="utf-8", errors="replace")

        doc_id = self.db.insert_document(
            filename=filename,
            path=str(md_path.resolve()),
            metadata={"size_bytes": len(text)},
        )

        self.db.delete_fragments_by_document(doc_id)

        chunks = chunk_text(text, self.chunk_size, self.chunk_overlap)
        if not chunks:
            return {"chunks": 0, "entities": 0, "relations": 0}

        texts = [c[0] for c in chunks]
        indices = [c[1] for c in chunks]
        vectors = self.embedder.encode_batch(texts)

        entity_count = 0
        relation_count = 0

        for content, idx, vec in zip(texts, indices, vectors):
            fragment_id = self.db.insert_fragment(doc_id, content, vec, idx)

            extracted = self.extractor.extract(content)
            entity_ids: dict[str, int] = {}
            for ent in extracted.get("entidades", []):
                eid = self.db.insert_entity(
                    fragment_id,
                    name=ent["nombre"],
                    type_=ent.get("tipo", "desconocido"),
                )
                entity_ids[ent["nombre"]] = eid
                entity_count += 1

            for rel in extracted.get("relaciones", []):
                src_id = entity_ids.get(rel.get("origen", ""))
                dst_id = entity_ids.get(rel.get("destino", ""))
                if src_id and dst_id:
                    self.db.insert_relation(src_id, dst_id, rel.get("tipo", "relacionado"))
                    relation_count += 1

        logger.debug("Procesado %s: %d fragmentos, %d entidades, %d relaciones", filename, len(chunks), entity_count, relation_count)
        return {"chunks": len(chunks), "entities": entity_count, "relations": relation_count}


def main() -> None:
    logging.basicConfig(
        level=logging.INFO,
        format="%(message)s",
        handlers=[RichHandler(rich_tracebacks=True)],
    )
    pipeline = Pipeline()
    pipeline.run()


if __name__ == "__main__":
    main()
