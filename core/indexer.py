"""
indexer.py — Fase 1: Escaneo, análisis y vectorización de código fuente.

Pipeline:
1. Lee config.json del proyecto
2. Escanea archivos ignorando exclusiones
3. Divide cada archivo en chunks con overlap
4. Carga CDLM-0.5B para generar resúmenes de cada chunk
5. Vectoriza (resumen + código) con sentence-transformers
6. Guarda índice FAISS + metadatos JSON en vector_db/
7. Descarga el modelo al finalizar

Uso:
    from core.indexer import ProjectIndexer
    indexer = ProjectIndexer("mi_proyecto")
    indexer.run()
"""

import json
import logging
import os
import time
from pathlib import Path
from typing import Optional

import faiss
import numpy as np

from core.memory_manager import MemoryManager

logger = logging.getLogger("IA-Dev-System.Indexer")

# Ruta base del sistema
BASE_DIR = Path(__file__).resolve().parent.parent
PROJECTS_DIR = BASE_DIR / "projects"
# Modelos en la compilación local de llama.cpp
MODELS_DIR = Path.home() / "llama.cpp" / "build-vulkan" / "bin" / "models"


class ProjectIndexer:
    """Indexador de proyectos de código fuente.
    
    Escanea, analiza con LLM y vectoriza código para búsqueda semántica.
    """

    # Prompt para que CDLM analice código
    ANALYSIS_PROMPT_TEMPLATE = """Analyze the following source code and provide a concise technical summary.
Include: purpose, key functions/classes, dependencies, and important patterns.
Respond in a structured format.

File: {filename}
Language: {language}

```
{code}
```

Technical Summary:"""

    def __init__(
        self,
        project_name: str,
        models_dir: Optional[str] = None,
        analyst_model: str = "CDLM-0.5B.Q8_0.gguf",
        embedding_model: str = "imocha-ai-org/ssf-skill-extractor",
        embedding_fallback: str = "sentence-transformers/all-MiniLM-L6-v2",
    ):
        """
        Args:
            project_name: Nombre del proyecto a indexar
            models_dir: Directorio de modelos GGUF (override)
            analyst_model: Nombre del modelo GGUF para análisis
            embedding_model: Modelo de sentence-transformers para embeddings
            embedding_fallback: Modelo fallback si el principal falla
        """
        self.project_name = project_name
        self.project_dir = PROJECTS_DIR / project_name
        self.config_path = self.project_dir / "config.json"
        self.vector_db_dir = self.project_dir / "vector_db"
        self.models_dir = Path(models_dir) if models_dir else MODELS_DIR
        self.analyst_model_name = analyst_model
        self.embedding_model_name = embedding_model
        self.embedding_fallback_name = embedding_fallback

        self.config: dict = {}
        self._embedding_model = None
        self._memory_manager = MemoryManager()

        # Estadísticas
        self.stats = {
            "files_scanned": 0,
            "files_indexed": 0,
            "chunks_created": 0,
            "chunks_summarized": 0,
            "errors": 0,
            "total_time_s": 0.0,
            "indexing_time_s": 0.0,
            "embedding_time_s": 0.0,
        }

    def _load_config(self) -> dict:
        """Carga y valida la configuración del proyecto."""
        if not self.config_path.exists():
            raise FileNotFoundError(
                f"Config no encontrada: {self.config_path}\n"
                f"Crea el proyecto primero con: python main.py init {self.project_name} <ruta>"
            )

        with open(self.config_path, "r", encoding="utf-8") as f:
            self.config = json.load(f)

        required_keys = ["project_name", "source_path"]
        for key in required_keys:
            if key not in self.config:
                raise ValueError(f"Config incompleta: falta '{key}' en {self.config_path}")

        source_path = Path(self.config["source_path"])
        if not source_path.exists():
            raise FileNotFoundError(
                f"Directorio de código no encontrado: {source_path}"
            )

        logger.info(f"Config cargada: proyecto='{self.config['project_name']}'")
        logger.info(f"  Código fuente: {self.config['source_path']}")
        logger.info(f"  Extensiones: {self.config.get('extensions', ['*'])}")
        logger.info(f"  Exclusiones: {self.config.get('exclude_dirs', [])}")

        return self.config

    def _scan_files(self) -> list[Path]:
        """Escanea el directorio de código respetando exclusiones."""
        source_path = Path(self.config["source_path"])
        extensions = set(self.config.get("extensions", [
            ".py", ".js", ".ts", ".jsx", ".tsx", ".java", ".cpp", ".c",
            ".h", ".hpp", ".cs", ".go", ".rs", ".rb", ".php", ".swift",
            ".kt", ".scala", ".lua", ".sh", ".bash", ".sql", ".r",
            ".html", ".css", ".scss", ".vue", ".svelte",
            ".json", ".yaml", ".yml", ".toml", ".xml",
            ".md", ".txt", ".rst",
        ]))
        exclude_dirs = set(self.config.get("exclude_dirs", [
            "node_modules", ".git", "__pycache__", "venv", ".venv",
            ".idea", ".vscode", "dist", "build", ".next", "target",
            "vendor", ".tox", ".mypy_cache", ".pytest_cache",
        ]))
        exclude_files = set(self.config.get("exclude_files", [
            "*.pyc", "*.pyo", "*.lock", "*.log", "*.min.js", "*.min.css",
            "*.map", "*.wasm", "*.so", "*.dll", "*.exe",
        ]))
        # Tamaño máximo de archivo (500KB por defecto)
        max_file_size = self.config.get("max_file_size_kb", 500) * 1024

        files = []
        for root, dirs, filenames in os.walk(source_path):
            # Filtrar directorios excluidos (modifica in-place para os.walk)
            dirs[:] = [d for d in dirs if d not in exclude_dirs]

            for fname in filenames:
                fpath = Path(root) / fname

                # Verificar extensión
                if extensions and fpath.suffix.lower() not in extensions:
                    continue

                # Verificar patrones de exclusión de archivos
                if any(fpath.match(pat) for pat in exclude_files):
                    continue

                # Verificar tamaño
                try:
                    if fpath.stat().st_size > max_file_size:
                        logger.debug(f"  Archivo demasiado grande, omitido: {fpath}")
                        continue
                    if fpath.stat().st_size == 0:
                        continue
                except OSError:
                    continue

                files.append(fpath)

        files.sort()
        self.stats["files_scanned"] = len(files)
        logger.info(f"Archivos encontrados: {len(files)}")
        return files

    @staticmethod
    def _chunk_text(
        text: str,
        chunk_size: int = 1500,
        overlap: int = 200,
    ) -> list[str]:
        """Divide texto en chunks con overlap.
        
        Args:
            text: Texto a dividir
            chunk_size: Tamaño máximo de cada chunk (caracteres)
            overlap: Solapamiento entre chunks consecutivos
            
        Returns:
            Lista de chunks de texto
        """
        if len(text) <= chunk_size:
            return [text]

        chunks = []
        start = 0
        while start < len(text):
            end = start + chunk_size

            # Intentar cortar en un salto de línea para no partir funciones
            if end < len(text):
                # Buscar el último salto de línea dentro del chunk
                newline_pos = text.rfind("\n", start + chunk_size // 2, end)
                if newline_pos > start:
                    end = newline_pos + 1

            chunk = text[start:end].strip()
            if chunk:
                chunks.append(chunk)

            start = end - overlap
            if start >= len(text):
                break

        return chunks

    def _load_embedding_model(self):
        """Carga el modelo de embeddings (sentence-transformers)."""
        if self._embedding_model is not None:
            return self._embedding_model

        from sentence_transformers import SentenceTransformer

        logger.info(f"Cargando modelo de embeddings: {self.embedding_model_name}")
        start = time.time()

        try:
            self._embedding_model = SentenceTransformer(self.embedding_model_name)
            logger.info(
                f"✓ Modelo embeddings cargado: {self.embedding_model_name} "
                f"({time.time()-start:.1f}s)"
            )
        except Exception as e:
            logger.warning(
                f"Error cargando {self.embedding_model_name}: {e}. "
                f"Usando fallback: {self.embedding_fallback_name}"
            )
            self._embedding_model = SentenceTransformer(self.embedding_fallback_name)
            logger.info(f"✓ Modelo fallback cargado: {self.embedding_fallback_name}")

        return self._embedding_model

    def _generate_summary(self, llm, code: str, filename: str) -> str:
        """Genera un resumen técnico del código usando CDLM.
        
        Args:
            llm: Instancia de Llama cargada
            code: Fragmento de código
            filename: Nombre del archivo fuente
            
        Returns:
            Resumen generado por el LLM
        """
        language = self.config.get("language", "unknown")
        # Truncar código si es demasiado largo para el contexto
        max_code_chars = 3000
        truncated = code[:max_code_chars] if len(code) > max_code_chars else code

        prompt = self.ANALYSIS_PROMPT_TEMPLATE.format(
            filename=filename,
            language=language,
            code=truncated,
        )

        try:
            response = llm(
                prompt,
                max_tokens=256,
                temperature=0.1,
                top_p=0.9,
                stop=["\n\n\n", "```"],
                echo=False,
            )
            summary = response["choices"][0]["text"].strip()
            if not summary:
                summary = f"Código de {filename}: {code[:200]}"
            return summary
        except Exception as e:
            logger.warning(f"Error generando resumen para {filename}: {e}")
            return f"Archivo: {filename}\n{code[:300]}"

    def _save_vector_db(
        self,
        embeddings: np.ndarray,
        metadata: list[dict],
    ) -> None:
        """Guarda el índice FAISS y los metadatos en disco.
        
        Args:
            embeddings: Matriz de embeddings (n_chunks x dim)
            metadata: Lista de metadatos por chunk
        """
        self.vector_db_dir.mkdir(parents=True, exist_ok=True)

        # Guardar índice FAISS
        index = faiss.IndexFlatIP(embeddings.shape[1])  # Inner Product (cosine con normalización)
        # Normalizar para cosine similarity
        faiss.normalize_L2(embeddings)
        index.add(embeddings)

        index_path = self.vector_db_dir / "index.faiss"
        faiss.write_index(index, str(index_path))
        logger.info(f"Índice FAISS guardado: {index_path} ({index.ntotal} vectores)")

        # Guardar metadatos
        meta_path = self.vector_db_dir / "metadata.json"
        with open(meta_path, "w", encoding="utf-8") as f:
            json.dump(metadata, f, ensure_ascii=False, indent=2)
        logger.info(f"Metadatos guardados: {meta_path}")

        # Guardar info del índice
        info = {
            "project_name": self.project_name,
            "num_vectors": index.ntotal,
            "embedding_dim": embeddings.shape[1],
            "embedding_model": self.embedding_model_name,
            "analyst_model": self.analyst_model_name,
            "indexed_at": time.strftime("%Y-%m-%d %H:%M:%S"),
            "stats": self.stats,
        }
        info_path = self.vector_db_dir / "index_info.json"
        with open(info_path, "w", encoding="utf-8") as f:
            json.dump(info, f, ensure_ascii=False, indent=2)

    def run(self, skip_summaries: bool = False) -> dict:
        """Ejecuta el pipeline completo de indexación.
        
        Args:
            skip_summaries: Si True, omite la generación de resúmenes con LLM
                          (útil para pruebas rápidas sin GPU)
                          
        Returns:
            Dict con estadísticas de la indexación
        """
        from rich.console import Console
        from rich.progress import Progress, SpinnerColumn, TextColumn, BarColumn

        console = Console()
        total_start = time.time()

        console.print(f"\n[bold cyan]═══ Indexando proyecto: {self.project_name} ═══[/bold cyan]\n")

        # 1. Cargar configuración
        self._load_config()

        # 2. Escanear archivos
        console.print("[bold]Fase 1/4:[/bold] Escaneando archivos...")
        files = self._scan_files()
        if not files:
            console.print("[yellow]⚠ No se encontraron archivos para indexar[/yellow]")
            return self.stats

        # 3. Crear chunks
        console.print("[bold]Fase 2/4:[/bold] Dividiendo en fragmentos...")
        chunk_size = self.config.get("chunk_size", 1500)
        chunk_overlap = self.config.get("chunk_overlap", 200)

        all_chunks: list[dict] = []
        source_path = Path(self.config["source_path"])

        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            BarColumn(),
            TextColumn("[progress.percentage]{task.percentage:>3.0f}%"),
            console=console,
        ) as progress:
            task = progress.add_task("Procesando archivos", total=len(files))

            for fpath in files:
                try:
                    content = fpath.read_text(encoding="utf-8", errors="replace")
                except Exception as e:
                    logger.warning(f"Error leyendo {fpath}: {e}")
                    self.stats["errors"] += 1
                    progress.advance(task)
                    continue

                # Ruta relativa al proyecto
                try:
                    rel_path = str(fpath.relative_to(source_path))
                except ValueError:
                    rel_path = str(fpath)

                chunks = self._chunk_text(content, chunk_size, chunk_overlap)

                for i, chunk in enumerate(chunks):
                    all_chunks.append({
                        "file": rel_path,
                        "chunk_index": i,
                        "total_chunks": len(chunks),
                        "content": chunk,
                        "summary": "",  # Se llenará en Fase 3
                        "language": fpath.suffix.lstrip("."),
                    })

                self.stats["files_indexed"] += 1
                progress.advance(task)

        self.stats["chunks_created"] = len(all_chunks)
        console.print(f"  Chunks creados: {len(all_chunks)} de {self.stats['files_indexed']} archivos")

        if not all_chunks:
            console.print("[yellow]⚠ No se generaron chunks[/yellow]")
            return self.stats

        # 4. Generar resúmenes con CDLM (si no se omite)
        if not skip_summaries:
            console.print("[bold]Fase 3/4:[/bold] Generando resúmenes con CDLM...")
            analyst_path = self.models_dir / self.analyst_model_name

            try:
                llm = self._memory_manager.load_model(
                    str(analyst_path),
                    n_ctx=2048,
                    n_gpu_layers=-1,
                )

                summary_start = time.time()
                with Progress(
                    SpinnerColumn(),
                    TextColumn("[progress.description]{task.description}"),
                    BarColumn(),
                    TextColumn("[progress.percentage]{task.percentage:>3.0f}%"),
                    console=console,
                ) as progress:
                    task = progress.add_task("Analizando código", total=len(all_chunks))

                    for chunk_data in all_chunks:
                        chunk_data["summary"] = self._generate_summary(
                            llm,
                            chunk_data["content"],
                            chunk_data["file"],
                        )
                        self.stats["chunks_summarized"] += 1
                        progress.advance(task)

                self.stats["indexing_time_s"] = time.time() - summary_start
                console.print(
                    f"  Resúmenes generados: {self.stats['chunks_summarized']} "
                    f"({self.stats['indexing_time_s']:.1f}s)"
                )

            except (FileNotFoundError, RuntimeError, MemoryError) as e:
                console.print(f"[yellow]⚠ Modelo CDLM no disponible: {e}[/yellow]")
                console.print("[yellow]  Continuando sin resúmenes LLM...[/yellow]")
            finally:
                # SIEMPRE descargar el modelo al terminar
                self._memory_manager.unload_model()
        else:
            console.print("[dim]Fase 3/4: Resúmenes omitidos (skip_summaries=True)[/dim]")

        # 5. Vectorizar con sentence-transformers
        console.print("[bold]Fase 4/4:[/bold] Vectorizando con embeddings...")
        embed_start = time.time()

        self._load_embedding_model()

        # Combinar resumen + código para el embedding
        texts_to_embed = []
        for chunk_data in all_chunks:
            if chunk_data["summary"]:
                combined = f"{chunk_data['summary']}\n\n{chunk_data['content'][:500]}"
            else:
                combined = chunk_data["content"]
            texts_to_embed.append(combined)

        console.print(f"  Generando {len(texts_to_embed)} embeddings...")
        embeddings = self._embedding_model.encode(
            texts_to_embed,
            show_progress_bar=True,
            batch_size=32,
            normalize_embeddings=False,  # Lo haremos con FAISS
        )
        embeddings = np.array(embeddings, dtype=np.float32)
        self.stats["embedding_time_s"] = time.time() - embed_start

        console.print(
            f"  Embeddings generados: {embeddings.shape} "
            f"({self.stats['embedding_time_s']:.1f}s)"
        )

        # 6. Guardar en disco
        metadata = [
            {
                "file": c["file"],
                "chunk_index": c["chunk_index"],
                "total_chunks": c["total_chunks"],
                "language": c["language"],
                "summary": c["summary"],
                "content": c["content"],
            }
            for c in all_chunks
        ]

        self._save_vector_db(embeddings, metadata)

        # Estadísticas finales
        self.stats["total_time_s"] = time.time() - total_start
        console.print(f"\n[bold green]✓ Indexación completada en {self.stats['total_time_s']:.1f}s[/bold green]")
        console.print(f"  Archivos: {self.stats['files_indexed']}/{self.stats['files_scanned']}")
        console.print(f"  Chunks: {self.stats['chunks_created']}")
        console.print(f"  Errores: {self.stats['errors']}")
        console.print(f"  Vector DB: {self.vector_db_dir}")

        return self.stats
