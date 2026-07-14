import os
from dataclasses import dataclass, field

# Cargar .env si existe
try:
    from dotenv import load_dotenv
    load_dotenv()
except ImportError:
    pass


def _env_int(key: str, default: int) -> int:
    val = os.getenv(key)
    if val is None:
        return default
    try:
        return int(val)
    except ValueError:
        return default


@dataclass(frozen=True)
class DBConfig:
    host: str = field(default_factory=lambda: os.getenv("PGHOST", "localhost"))
    port: int = field(default_factory=lambda: _env_int("PGPORT", 5433))
    dbname: str = field(default_factory=lambda: os.getenv("PGDATABASE", "alesys"))
    user: str = field(default_factory=lambda: os.getenv("PGUSER", "alesys"))
    password: str = field(default_factory=lambda: os.getenv("PGPASSWORD", "alesys"))


@dataclass(frozen=True)
class OpenRouterConfig:
    api_key: str = field(default_factory=lambda: os.getenv("OPENROUTER_API_KEY", ""))
    base_url: str = "https://openrouter.ai/api/v1"
    model: str = field(default_factory=lambda: os.getenv("OPENROUTER_MODEL", "google/gemma-4-31b-it:free"))
    max_retries: int = 3
    timeout: int = 60


@dataclass(frozen=True)
class EmbeddingConfig:
    backend: str = field(default_factory=lambda: os.getenv("EMBEDDING_BACKEND", "sentence-transformers"))
    model_name: str = field(default_factory=lambda: os.getenv("EMBEDDING_MODEL", "sentence-transformers/all-MiniLM-L6-v2"))
    dimension: int = field(default_factory=lambda: _env_int("EMBEDDING_DIM", 384))
    device: str = field(default_factory=lambda: os.getenv("EMBEDDING_DEVICE", "cpu"))
    
    # llama.cpp / GGUF options
    gguf_model_path: str = field(default_factory=lambda: os.getenv("EMBEDDING_GGUF_PATH", ""))
    n_gpu_layers: int = field(default_factory=lambda: _env_int("EMBEDDING_N_GPU_LAYERS", -1))
    n_ctx: int = field(default_factory=lambda: _env_int("EMBEDDING_N_CTX", 8192))
    n_batch: int = field(default_factory=lambda: _env_int("EMBEDDING_N_BATCH", 512))
    use_vulkan: bool = field(default_factory=lambda: os.getenv("EMBEDDING_USE_VULKAN", "false").lower() == "true")
    llama_cpp_lib_path: str = field(default_factory=lambda: os.getenv("EMBEDDING_LLAMA_CPP_LIB_PATH", ""))


@dataclass(frozen=True)
class PathConfig:
    books_dir: str = field(
        default_factory=lambda: os.getenv(
            "BOOKS_DIR",
            "./books",
        )
    )
    sandbox_dir: str = field(
        default_factory=lambda: os.getenv(
            "SANDBOX_DIR",
            "./sandbox",
        )
    )


@dataclass(frozen=True)
class ChunkingConfig:
    size: int = 1000
    overlap: int = 200


DB = DBConfig()
OPENROUTER = OpenRouterConfig()
EMBEDDING = EmbeddingConfig()
PATHS = PathConfig()
CHUNKING = ChunkingConfig()
