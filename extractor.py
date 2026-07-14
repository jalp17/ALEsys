import json
import logging
import time
from typing import Any, Optional

import httpx

from config import OPENROUTER

logger = logging.getLogger(__name__)

EXTRACTION_PROMPT = """Eres un extractor de conocimiento científico. Analiza el texto y extrae:
1. **Entidades científicas**: conceptos, términos, teorías, autores, ecuaciones o cualquier entidad relevante.
2. **Relaciones lógicas**: conexiones semánticas entre entidades (es_un, contiene, implica, contradice, define, ejemplifica, etc.).

Responde ÚNICAMENTE con un objeto JSON sin markdown ni texto adicional, siguiendo este schema:
{
  "entidades": [
    {"nombre": "nombre de la entidad", "tipo": "categoria_cientifica"}
  ],
  "relaciones": [
    {"origen": "nombre entidad origen", "destino": "nombre entidad destino", "tipo": "tipo_relacion"}
  ]
}

Si no hay entidades o relaciones, devuelve {"entidades": [], "relaciones": []}."""


class Extractor:
    def __init__(self) -> None:
        self._client: Optional[httpx.Client] = None

    @property
    def client(self) -> httpx.Client:
        if self._client is None:
            self._client = httpx.Client(
                base_url=OPENROUTER.base_url,
                timeout=OPENROUTER.timeout,
                headers={
                    "Authorization": f"Bearer {OPENROUTER.api_key}",
                    "Content-Type": "application/json",
                },
            )
        return self._client

    def _call(self, text: str, system_prompt: str) -> str:
        payload = {
            "model": OPENROUTER.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": text},
            ],
            "temperature": 0.1,
            "max_tokens": 2048,
        }
        response = self.client.post("/chat/completions", json=payload)
        if response.status_code != 200:
            logger.error("OpenRouter error %s: %s", response.status_code, response.text)
        response.raise_for_status()
        data = response.json()
        return data["choices"][0]["message"]["content"]

    def extract(self, text: str) -> dict[str, list[dict[str, str]]]:
        if not OPENROUTER.api_key:
            logger.warning("OPENROUTER_API_KEY no configurada, saltando extracción")
            return {"entidades": [], "relaciones": []}

        for attempt in range(OPENROUTER.max_retries):
            try:
                content = self._call(text, EXTRACTION_PROMPT)
                content = content.strip()
                if content.startswith("```"):
                    content = content.split("\n", 1)[-1]
                    content = content.rsplit("```", 1)[0]
                content = content.strip()
                result = json.loads(content)
                if not isinstance(result, dict):
                    logger.warning("La IA devolvió JSON no válido (no es dict): %s", type(result).__name__)
                    return {"entidades": [], "relaciones": []}
                result.setdefault("entidades", [])
                result.setdefault("relaciones", [])
                return result
            except httpx.HTTPStatusError as e:
                if e.response.status_code == 429:
                    wait = 2 ** attempt
                    logger.warning("Rate limit (intento %d), esperando %ds", attempt + 1, wait)
                    time.sleep(wait)
                    continue
                logger.exception("Error HTTP en extracción (intento %d)", attempt + 1)
            except (httpx.RequestError, json.JSONDecodeError, KeyError, AssertionError):
                logger.exception("Error en extracción (intento %d)", attempt + 1)

            if attempt < OPENROUTER.max_retries - 1:
                time.sleep(1)

        logger.error("Extracción fallida después de %d intentos", OPENROUTER.max_retries)
        return {"entidades": [], "relaciones": []}

    def answer(self, question: str, context: str) -> str:
        if not OPENROUTER.api_key:
            return "OPENROUTER_API_KEY no configurada."

        qa_prompt = (
            "Responde la pregunta basándote exclusivamente en el contexto proporcionado. "
            "Si el contexto no contiene suficiente información, indícalo claramente. "
            "Responde en el mismo idioma de la pregunta."
        )
        text = f"Contexto:\n{context}\n\nPregunta: {question}"

        for attempt in range(OPENROUTER.max_retries):
            try:
                content = self._call(text, qa_prompt)
                return content.strip()
            except httpx.HTTPStatusError as e:
                if e.response.status_code == 429:
                    time.sleep(2 ** attempt)
                    continue
                logger.exception("Error HTTP en answer (intento %d)", attempt + 1)
            except (httpx.RequestError, KeyError):
                logger.exception("Error en answer (intento %d)", attempt + 1)
            if attempt < OPENROUTER.max_retries - 1:
                time.sleep(1)

        return "Error al generar respuesta."

    def close(self) -> None:
        if self._client is not None:
            self._client.close()
            self._client = None
