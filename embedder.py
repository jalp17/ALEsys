import logging
from typing import Optional, List, Protocol

from config import EMBEDDING

logger = logging.getLogger(__name__)


class EmbedderProtocol(Protocol):
    def encode(self, text: str) -> List[float]: ...
    def encode_batch(self, texts: List[str]) -> List[List[float]]: ...
    def unload(self) -> None: ...


class SentenceTransformersEmbedder:
    def __init__(self) -> None:
        self._model: Optional["SentenceTransformer"] = None

    def _load(self) -> "SentenceTransformer":
        if self._model is None:
            from sentence_transformers import SentenceTransformer

            logger.info("Cargando modelo %s en %s", EMBEDDING.model_name, EMBEDDING.device)
            self._model = SentenceTransformer(EMBEDDING.model_name, device=EMBEDDING.device)
        return self._model

    def encode(self, text: str) -> List[float]:
        model = self._load()
        vector = model.encode(text, normalize_embeddings=True).tolist()
        assert len(vector) == EMBEDDING.dimension, (
            f"Dimensión del embedding {len(vector)} != {EMBEDDING.dimension}"
        )
        return vector

    def encode_batch(self, texts: List[str]) -> List[List[float]]:
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
            logger.info("Modelo sentence-transformers descargado")


class LlamaCppEmbedder:
    def __init__(self) -> None:
        self._model: Optional["Llama"] = None
        self._model_path = EMBEDDING.gguf_model_path

    def _load(self) -> "Llama":
        if self._model is None:
            from llama_cpp import Llama

            if not self._model_path:
                raise ValueError(
                    "EMBEDDING_GGUF_PATH no configurado. "
                    "Establece la variable de entorno EMBEDDING_GGUF_PATH "
                    "o configúralo en .env con la ruta al modelo .gguf"
                )

            logger.info(
                "Cargando modelo GGUF %s (device=%s, vulkan=%s, n_gpu_layers=%d)",
                self._model_path,
                EMBEDDING.device,
                EMBEDDING.use_vulkan,
                EMBEDDING.n_gpu_layers,
            )

            self._model = Llama(
                model_path=self._model_path,
                n_ctx=EMBEDDING.n_ctx,
                n_batch=EMBEDDING.n_batch,
                n_gpu_layers=EMBEDDING.n_gpu_layers,
                embedding=True,
                verbose=False,
                use_vulkan=EMBEDDING.use_vulkan,
            )
        return self._model

    def encode(self, text: str) -> List[float]:
        model = self._load()
        embedding = model.create_embedding(text)
        return embedding["data"][0]["embedding"]

    def encode_batch(self, texts: List[str]) -> List[List[float]]:
        model = self._load()
        embeddings = model.create_embedding(texts)
        return [e["embedding"] for e in embeddings["data"]]

    def unload(self) -> None:
        if self._model is not None:
            import gc
            del self._model
            self._model = None
            gc.collect()
            logger.info("Modelo GGUF descargado")


def create_embedder() -> EmbedderProtocol:
    backend = EMBEDDING.backend.lower()
    if backend == "llama.cpp":
        return LlamaCppEmbedder()
    elif backend == "sentence-transformers":
        return SentenceTransformersEmbedder()
    else:
        raise ValueError(f"Backend de embedding desconocido: {backend}. Usa 'sentence-transformers' o 'llama.cpp'")


Embedder = create_embedder()


def get_embedder() -> EmbedderProtocol:
    return Embedder