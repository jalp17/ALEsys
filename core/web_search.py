"""
web_search.py — Módulo de búsqueda web para complementar el contexto RAG.

Usa DuckDuckGo (ddgs) para buscar documentación, sintaxis,
y datos complementarios sin necesidad de API keys.

Uso:
    from core.web_search import WebSearcher
    ws = WebSearcher()
    results = ws.search("python asyncio gather syntax")
"""

import logging
from typing import Optional

logger = logging.getLogger("IA-Dev-System.WebSearch")


class WebSearcher:
    """Buscador web usando DuckDuckGo para complementar información del proyecto."""

    def __init__(self, region: str = "es-es", max_results: int = 5):
        """
        Args:
            region: Región para los resultados (es-es, en-us, etc.)
            max_results: Número máximo de resultados por búsqueda
        """
        self.region = region
        self.max_results = max_results
        self._ddgs = None
        logger.info(f"WebSearcher inicializado (región={region})")

    def _get_ddgs(self):
        """Inicializa cliente DDGS de forma lazy."""
        if self._ddgs is None:
            try:
                from ddgs import DDGS
                self._ddgs = DDGS()
                logger.debug("Cliente DDGS inicializado")
            except ImportError:
                logger.error(
                    "ddgs no está instalado. "
                    "Instala con: pip install ddgs"
                )
                raise
        return self._ddgs

    def search(
        self,
        query: str,
        max_results: Optional[int] = None,
        timelimit: Optional[str] = None,
    ) -> list[dict]:
        """Realiza una búsqueda web y retorna resultados formateados.
        
        Args:
            query: Consulta de búsqueda
            max_results: Override del máximo de resultados
            timelimit: Filtro de tiempo ('d'=día, 'w'=semana, 'm'=mes, 'y'=año)
            
        Returns:
            Lista de dicts con keys: title, url, body
        """
        n = max_results or self.max_results
        logger.info(f"Buscando: '{query}' (max={n})")

        try:
            ddgs = self._get_ddgs()
            results = list(ddgs.text(
                query,
                region=self.region,
                max_results=n,
                timelimit=timelimit,
            ))
            logger.info(f"  → {len(results)} resultados encontrados")
            return [
                {
                    "title": r.get("title", ""),
                    "url": r.get("href", r.get("link", "")),
                    "body": r.get("body", r.get("snippet", "")),
                }
                for r in results
            ]
        except Exception as e:
            logger.warning(f"Error en búsqueda web: {e}")
            return []

    def search_code_docs(self, language: str, topic: str) -> list[dict]:
        """Búsqueda enfocada en documentación de código.
        
        Args:
            language: Lenguaje de programación (python, javascript, etc.)
            topic: Tema o función a buscar
            
        Returns:
            Lista de resultados relevantes
        """
        query = f"{language} {topic} documentation syntax example"
        return self.search(query)

    def search_error(self, error_message: str, language: str = "") -> list[dict]:
        """Busca soluciones para un mensaje de error.
        
        Args:
            error_message: Mensaje de error
            language: Lenguaje de programación (opcional)
            
        Returns:
            Lista de resultados con posibles soluciones
        """
        # Limpiar el mensaje de error para la búsqueda
        clean_error = error_message.strip()[:200]
        query = f"{language} {clean_error} solution fix" if language else f"{clean_error} solution fix"
        return self.search(query)

    def format_results_as_context(self, results: list[dict], max_chars: int = 2000) -> str:
        """Formatea resultados de búsqueda como contexto para el LLM.
        
        Args:
            results: Lista de resultados de búsqueda
            max_chars: Límite de caracteres en el contexto generado
            
        Returns:
            Texto formateado para inyectar en el prompt
        """
        if not results:
            return ""

        parts = ["--- Información de Internet ---"]
        current_len = len(parts[0])

        for i, r in enumerate(results, 1):
            entry = f"\n[{i}] {r['title']}\n    URL: {r['url']}\n    {r['body']}"
            if current_len + len(entry) > max_chars:
                break
            parts.append(entry)
            current_len += len(entry)

        parts.append("--- Fin información de Internet ---")
        return "\n".join(parts)
