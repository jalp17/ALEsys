import hashlib
import json
import logging
import os
import tempfile
from pathlib import Path
from typing import Optional, List, Protocol

from config import EMBEDDING

logger = logging.getLogger(__name__)


CACHE_DIR = Path(tempfile.gettempdir()) / "alesys_embedding_cache"


def _cache_key(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def _cache_get(key: str, model_name: str) -> Optional[List[float]]:
    cache_path = CACHE_DIR / model_name.replace("/", "_") / f"{key}.json"
    if cache_path.exists():
        try:
            data = json.loads(cache_path.read_text())
            if len(data) == EMBEDDING.dimension:
                logger.debug("Cache hit: %s", key[:16])
                return data
        except (json.JSONDecodeError, OSError):
            pass
    return None


def _cache_set(key: str, model_name: str, vector: List[float]) -> None:
    cache_dir = CACHE_DIR / model_name.replace("/", "_")
    cache_dir.mkdir(parents=True, exist_ok=True)
    cache_path = cache_dir / f"{key}.json"
    try:
        cache_path.write_text(json.dumps(vector))
        logger.debug("Cache set: %s", key[:16])
    except OSError:
        pass


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
        key = _cache_key(text)
        cached = _cache_get(key, EMBEDDING.model_name)
        if cached is not None:
            return cached
        model = self._load()
        vector = model.encode(text, normalize_embeddings=True).tolist()
        assert len(vector) == EMBEDDING.dimension, (
            f"Dimensión del embedding {len(vector)} != {EMBEDDING.dimension}"
        )
        _cache_set(key, EMBEDDING.model_name, vector)
        return vector

    def encode_batch(self, texts: List[str]) -> List[List[float]]:
        keys = [_cache_key(t) for t in texts]
        cached: list[Optional[List[float]]] = [_cache_get(k, EMBEDDING.model_name) for k in keys]
        uncached_indices = [i for i, v in enumerate(cached) if v is None]
        if uncached_indices:
            uncached_texts = [texts[i] for i in uncached_indices]
            model = self._load()
            vectors = model.encode(uncached_texts, normalize_embeddings=True).tolist()
            for i, v in zip(uncached_indices, vectors):
                assert len(v) == EMBEDDING.dimension, (
                    f"Dimensión del embedding {len(v)} != {EMBEDDING.dimension} (índice {i})"
                )
                _cache_set(keys[i], EMBEDDING.model_name, v)
                cached[i] = v
        return [v for v in cached if v is not None]

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

            llama_kwargs = dict(
                model_path=self._model_path,
                n_ctx=EMBEDDING.n_ctx,
                n_batch=EMBEDDING.n_batch,
                n_gpu_layers=EMBEDDING.n_gpu_layers,
                embedding=True,
                verbose=False,
                use_vulkan=EMBEDDING.use_vulkan,
            )
            if EMBEDDING.llama_cpp_lib_path:
                llama_kwargs["llama_cpp_lib_path"] = EMBEDDING.llama_cpp_lib_path
                logger.info("Usando librería llama.cpp personalizada: %s", EMBEDDING.llama_cpp_lib_path)

            self._model = Llama(**llama_kwargs)
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


def clear_cache() -> None:
    if CACHE_DIR.exists():
        import shutil
        shutil.rmtree(CACHE_DIR)
        logger.info("Caché de embeddings limpiada")


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