"""
chat_agent.py — Fase 2: Consulta RAG + Generación con LLM conversacional.

Pipeline:
1. Recibe nombre del proyecto + pregunta del usuario
2. Carga índice FAISS + metadatos del proyecto
3. Busca los chunks más relevantes (cosine similarity)
4. Opcionalmente busca en internet para complementar
5. Carga modelo ruvltra vía MemoryManager
6. Inyecta contexto en el prompt y genera respuesta
7. Streaming token-a-token en consola

Uso:
    from core.chat_agent import ChatAgent
    agent = ChatAgent("mi_proyecto")
    agent.start_chat()
"""

import json
import logging
import time
from pathlib import Path
from typing import Optional

import faiss
import numpy as np

from core.memory_manager import MemoryManager
from core.web_search import WebSearcher

logger = logging.getLogger("IA-Dev-System.ChatAgent")

BASE_DIR = Path(__file__).resolve().parent.parent
PROJECTS_DIR = BASE_DIR / "projects"
MODELS_DIR = Path.home() / "llama.cpp" / "build-vulkan" / "bin" / "models"


class ChatAgent:
    """Agente de chat RAG para consultar proyectos indexados."""

    # Modelos conversacionales disponibles (en orden de preferencia)
    CONVERSATIONAL_MODELS = [
        "ruvltra-1.1b-q4_k_m.gguf",
        "ruvltra-claude-code-0.5b-q4_k_m.gguf",
    ]

    SYSTEM_PROMPT_TEMPLATE = """You are an expert code assistant analyzing the project "{project_name}".
You have access to the project's source code context below. Answer questions accurately based on this context.
If the context doesn't contain enough information, say so clearly.
Always format code in markdown code blocks with the appropriate language.
Respond in the same language as the user's question.

{web_context}

--- Project Source Code Context ---
{context}
--- End of Context ---"""

    def __init__(
        self,
        project_name: str,
        models_dir: Optional[str] = None,
        model_name: Optional[str] = None,
        embedding_model: str = "imocha-ai-org/ssf-skill-extractor",
        embedding_fallback: str = "sentence-transformers/all-MiniLM-L6-v2",
        top_k: int = 5,
        enable_web_search: bool = True,
    ):
        """
        Args:
            project_name: Nombre del proyecto a consultar
            models_dir: Directorio de modelos GGUF (override)
            model_name: Modelo conversacional específico (override auto-detect)
            embedding_model: Modelo de embeddings (debe coincidir con el indexador)
            embedding_fallback: Fallback de embeddings
            top_k: Número de chunks a recuperar por consulta
            enable_web_search: Habilitar búsqueda web complementaria
        """
        self.project_name = project_name
        self.project_dir = PROJECTS_DIR / project_name
        self.vector_db_dir = self.project_dir / "vector_db"
        self.models_dir = Path(models_dir) if models_dir else MODELS_DIR
        self.model_name = model_name
        self.embedding_model_name = embedding_model
        self.embedding_fallback_name = embedding_fallback
        self.top_k = top_k
        self.enable_web_search = enable_web_search

        self._memory_manager = MemoryManager()
        self._embedding_model = None
        self._faiss_index = None
        self._metadata: list[dict] = []
        self._web_searcher = WebSearcher() if enable_web_search else None

        # Historial de chat para contexto conversacional
        self._chat_history: list[dict] = []
        self._max_history = 6  # últimos 3 pares user/assistant

    def _load_vector_db(self) -> None:
        """Carga el índice FAISS y los metadatos del proyecto."""
        index_path = self.vector_db_dir / "index.faiss"
        meta_path = self.vector_db_dir / "metadata.json"

        if not index_path.exists():
            raise FileNotFoundError(
                f"Índice no encontrado: {index_path}\n"
                f"Ejecuta primero: python main.py index {self.project_name}"
            )

        logger.info(f"Cargando índice FAISS: {index_path}")
        self._faiss_index = faiss.read_index(str(index_path))
        logger.info(f"  Vectores en índice: {self._faiss_index.ntotal}")

        with open(meta_path, "r", encoding="utf-8") as f:
            self._metadata = json.load(f)
        logger.info(f"  Metadatos cargados: {len(self._metadata)} entradas")

    def _load_embedding_model(self):
        """Carga el modelo de embeddings para las consultas."""
        if self._embedding_model is not None:
            return self._embedding_model

        from sentence_transformers import SentenceTransformer

        logger.info(f"Cargando modelo de embeddings: {self.embedding_model_name}")
        try:
            self._embedding_model = SentenceTransformer(self.embedding_model_name)
        except Exception as e:
            logger.warning(f"Error: {e}. Usando fallback: {self.embedding_fallback_name}")
            self._embedding_model = SentenceTransformer(self.embedding_fallback_name)

        return self._embedding_model

    def _find_conversational_model(self) -> Path:
        """Busca el mejor modelo conversacional disponible."""
        if self.model_name:
            path = self.models_dir / self.model_name
            if path.exists():
                return path
            raise FileNotFoundError(f"Modelo especificado no encontrado: {path}")

        for name in self.CONVERSATIONAL_MODELS:
            path = self.models_dir / name
            if path.exists():
                logger.info(f"Modelo conversacional encontrado: {name}")
                return path

        raise FileNotFoundError(
            f"No se encontró modelo conversacional en {self.models_dir}.\n"
            f"Modelos buscados: {', '.join(self.CONVERSATIONAL_MODELS)}"
        )

    def retrieve(self, query: str, top_k: Optional[int] = None) -> list[dict]:
        """Busca los chunks más relevantes para una consulta.
        
        Args:
            query: Pregunta del usuario
            top_k: Override del número de resultados
            
        Returns:
            Lista de chunks con score de relevancia
        """
        k = top_k or self.top_k

        if self._faiss_index is None:
            self._load_vector_db()

        if self._faiss_index.ntotal == 0:
            logger.warning("Índice vacío, no hay chunks para buscar")
            return []

        # Generar embedding de la consulta
        self._load_embedding_model()
        query_embedding = self._embedding_model.encode(
            [query],
            normalize_embeddings=True,
        )
        query_embedding = np.array(query_embedding, dtype=np.float32)

        # Buscar en FAISS
        actual_k = min(k, self._faiss_index.ntotal)
        scores, indices = self._faiss_index.search(query_embedding, actual_k)

        results = []
        for score, idx in zip(scores[0], indices[0]):
            if idx < 0 or idx >= len(self._metadata):
                continue
            meta = self._metadata[idx].copy()
            meta["relevance_score"] = float(score)
            results.append(meta)

        logger.info(f"Retrieval: {len(results)} chunks recuperados para: '{query[:60]}...'")
        return results

    def _build_context(self, chunks: list[dict], max_chars: int = 6000) -> str:
        """Construye el contexto RAG a partir de los chunks recuperados.
        
        Args:
            chunks: Lista de chunks con metadatos
            max_chars: Límite de caracteres para el contexto
            
        Returns:
            Texto formateado para inyectar en el prompt
        """
        if not chunks:
            return "No se encontró contexto relevante en el código del proyecto."

        parts = []
        current_len = 0

        for i, chunk in enumerate(chunks):
            header = f"[Archivo: {chunk['file']} | Chunk {chunk['chunk_index']+1}/{chunk['total_chunks']} | Score: {chunk.get('relevance_score', 0):.3f}]"
            
            # Incluir resumen si existe
            content = ""
            if chunk.get("summary"):
                content += f"Resumen: {chunk['summary']}\n"
            content += f"Código:\n```{chunk.get('language', '')}\n{chunk['content']}\n```"

            entry = f"\n{header}\n{content}\n"

            if current_len + len(entry) > max_chars:
                break

            parts.append(entry)
            current_len += len(entry)

        return "\n".join(parts)

    def _build_messages(
        self,
        question: str,
        context: str,
        web_context: str = "",
    ) -> list[dict]:
        """Construye los mensajes para el LLM en formato chat.
        
        Args:
            question: Pregunta del usuario
            context: Contexto RAG del código
            web_context: Contexto de búsqueda web (opcional)
            
        Returns:
            Lista de mensajes en formato ChatML
        """
        system_msg = self.SYSTEM_PROMPT_TEMPLATE.format(
            project_name=self.project_name,
            context=context,
            web_context=web_context,
        )

        messages = [{"role": "system", "content": system_msg}]

        # Agregar historial de chat (últimos N mensajes)
        if self._chat_history:
            messages.extend(self._chat_history[-self._max_history:])

        messages.append({"role": "user", "content": question})

        return messages

    def ask(
        self,
        question: str,
        web_search: bool = False,
        web_query: Optional[str] = None,
        stream: bool = True,
    ) -> str:
        """Realiza una consulta al agente.
        
        Args:
            question: Pregunta del usuario
            web_search: Forzar búsqueda web para esta consulta
            web_query: Query personalizado para búsqueda web
            stream: Activar streaming de respuesta
            
        Returns:
            Respuesta generada por el LLM
        """
        from rich.console import Console
        from rich.markdown import Markdown

        console = Console()
        total_start = time.time()

        # 1. Recuperar chunks relevantes
        console.print("[dim]Buscando en el código del proyecto...[/dim]")
        chunks = self.retrieve(question)

        if not chunks:
            console.print("[yellow]⚠ No se encontraron fragmentos relevantes[/yellow]")

        context = self._build_context(chunks)

        # 2. Búsqueda web complementaria (si está habilitada)
        web_context = ""
        should_search = web_search or (
            self.enable_web_search
            and self._should_web_search(question)
        )
        if should_search and self._web_searcher:
            search_query = web_query or question
            console.print(f"[dim]Buscando en internet: '{search_query[:50]}...'[/dim]")
            web_results = self._web_searcher.search(search_query)
            web_context = self._web_searcher.format_results_as_context(web_results)

        # 3. Cargar modelo conversacional
        model_path = self._find_conversational_model()
        llm = self._memory_manager.load_model(
            str(model_path),
            n_ctx=4096,
            n_gpu_layers=-1,
        )

        # 4. Construir prompt y generar
        messages = self._build_messages(question, context, web_context)

        console.print(f"\n[bold cyan]Asistente ({Path(model_path).stem}):[/bold cyan]")

        if stream:
            response_text = self._stream_response(llm, messages, console)
        else:
            response_text = self._generate_response(llm, messages)
            console.print(Markdown(response_text))

        # 5. Guardar en historial
        self._chat_history.append({"role": "user", "content": question})
        self._chat_history.append({"role": "assistant", "content": response_text})

        elapsed = time.time() - total_start
        console.print(f"\n[dim]({elapsed:.1f}s | {len(chunks)} chunks | web: {'sí' if web_context else 'no'})[/dim]")

        return response_text

    @staticmethod
    def _should_web_search(question: str) -> bool:
        """Heurística para decidir si buscar en internet.
        
        Busca en internet cuando la pregunta parece pedir
        documentación, sintaxis, o información externa.
        """
        web_keywords = [
            "documentación", "documentation", "docs",
            "sintaxis", "syntax",
            "cómo se usa", "how to use", "how do i",
            "ejemplo", "example",
            "librería", "library", "package",
            "api", "referencia", "reference",
            "error", "bug", "fix",
            "instalar", "install",
            "versión", "version",
            "web", "internet", "buscar", "search",
        ]
        q_lower = question.lower()
        return any(kw in q_lower for kw in web_keywords)

    def _stream_response(self, llm, messages: list[dict], console) -> str:
        """Genera respuesta en modo streaming token-a-token.
        
        Args:
            llm: Instancia de Llama
            messages: Mensajes en formato ChatML
            console: Instancia de Rich Console
            
        Returns:
            Respuesta completa como string
        """
        response_parts = []

        try:
            stream = llm.create_chat_completion(
                messages=messages,
                max_tokens=1024,
                temperature=0.7,
                top_p=0.9,
                stream=True,
            )

            for chunk in stream:
                delta = chunk.get("choices", [{}])[0].get("delta", {})
                content = delta.get("content", "")
                if content:
                    console.print(content, end="")
                    response_parts.append(content)

            console.print()  # Salto de línea final
        except Exception as e:
            error_msg = f"\n[ERROR] Fallo en la generación: {e}"
            logger.error(error_msg)
            console.print(f"[red]{error_msg}[/red]")
            response_parts.append(error_msg)

        return "".join(response_parts)

    @staticmethod
    def _generate_response(llm, messages: list[dict]) -> str:
        """Genera respuesta completa (sin streaming).
        
        Args:
            llm: Instancia de Llama
            messages: Mensajes en formato ChatML
            
        Returns:
            Respuesta completa como string
        """
        try:
            response = llm.create_chat_completion(
                messages=messages,
                max_tokens=1024,
                temperature=0.7,
                top_p=0.9,
            )
            return response["choices"][0]["message"]["content"]
        except Exception as e:
            logger.error(f"Error generando respuesta: {e}")
            return f"Error: {e}"

    def start_chat(self) -> None:
        """Inicia un chat interactivo con el proyecto.
        
        Loop principal que acepta preguntas y genera respuestas.
        Comandos especiales:
            /salir, /exit, /quit — Termina el chat
            /web <query>         — Fuerza búsqueda web
            /clear               — Limpia historial
            /info                — Muestra info del proyecto
        """
        from rich.console import Console
        from rich.panel import Panel

        console = Console()

        # Verificar que el proyecto está indexado
        if not (self.vector_db_dir / "index.faiss").exists():
            console.print(
                f"[red]Error: El proyecto '{self.project_name}' no está indexado.[/red]\n"
                f"Ejecuta primero: [bold]python main.py index {self.project_name}[/bold]"
            )
            return

        # Cargar índice
        self._load_vector_db()
        self._load_embedding_model()

        # Mostrar info
        info_path = self.vector_db_dir / "index_info.json"
        if info_path.exists():
            with open(info_path, "r") as f:
                info = json.load(f)
            console.print(Panel(
                f"[bold]{self.project_name}[/bold]\n"
                f"Vectores: {info.get('num_vectors', '?')} | "
                f"Modelo: {info.get('embedding_model', '?')}\n"
                f"Indexado: {info.get('indexed_at', '?')}",
                title="Proyecto Cargado",
                border_style="cyan",
            ))

        console.print(
            "\n[dim]Escribe tu pregunta. "
            "Comandos: /salir, /web <query>, /clear, /info[/dim]\n"
        )

        try:
            while True:
                try:
                    question = console.input("[bold green]Tú:[/bold green] ").strip()
                except (EOFError, KeyboardInterrupt):
                    break

                if not question:
                    continue

                # Comandos especiales
                if question.lower() in ("/salir", "/exit", "/quit"):
                    break
                elif question.lower() == "/clear":
                    self._chat_history.clear()
                    console.print("[dim]Historial limpiado[/dim]")
                    continue
                elif question.lower() == "/info":
                    info = self._memory_manager.current_model_info
                    console.print(f"[dim]Modelo: {info}[/dim]")
                    console.print(f"[dim]Chunks en índice: {self._faiss_index.ntotal}[/dim]")
                    console.print(f"[dim]Historial: {len(self._chat_history)} msgs[/dim]")
                    continue
                elif question.lower().startswith("/web "):
                    web_query = question[5:].strip()
                    self.ask(web_query, web_search=True, web_query=web_query)
                    continue

                # Pregunta normal
                self.ask(question)

        finally:
            console.print("\n[dim]Descargando modelo...[/dim]")
            self._memory_manager.unload_model()
            console.print("[bold cyan]Chat finalizado.[/bold cyan]")
