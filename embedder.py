import logging
from typing import Optional

from config import EMBEDDING

logger = logging.getLogger(__name__)


class Embedder:
    def __init__(self) -> None:
        self._model: Optional["SentenceTransformer"] = None

    def _load(self) -> "SentenceTransformer":
        if self._model is None:
            from sentence_transformers import SentenceTransformer

            logger.info("Cargando modelo %s en %s", EMBEDDING.model_name, EMBEDDING.device)
            self._model = SentenceTransformer(EMBEDDING.model_name, device=EMBEDDING.device)
        return self._model

    def encode(self, text: str) -> list[float]:
        model = self._load()
        vector = model.encode(text, normalize_embeddings=True).tolist()
        assert len(vector) == EMBEDDING.dimension, (
            f"Dimensión del embedding {len(vector)} != {EMBEDDING.dimension}"
        )
        return vector

    def encode_batch(self, texts: list[str]) -> list[list[float]]:
        model = self._load()
        vectors = model.encode(texts, normalize_embeddings=True).tolist()
        for i, v in enumerate(vectors):
            assert len(v) == EMBEDDING.dimension, (
                f"Dimensión del embedding {len(v)} != {EMBEDDING.dimension} (índice {i})"
            )
        return vectors

    def unload(self) -> None:
        if self._model is not None:
            import gc
            del self._model
            self._model = None
            gc.collect()
            logger.info("Modelo de embeddings descargado")
