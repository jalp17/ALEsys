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
    port: int = field(default_factory=lambda: _env_int("PGPORT", 5432))
    dbname: str = field(default_factory=lambda: os.getenv("PGDATABASE", "alesys"))
    user: str = field(default_factory=lambda: os.getenv("PGUSER", "alesys"))
    password: str = field(default_factory=lambda: os.getenv("PGPASSWORD", "alesys"))


@dataclass(frozen=True)
class OpenRouterConfig:
    api_key: str = field(default_factory=lambda: os.getenv("OPENROUTER_API_KEY", ""))
    base_url: str = "https://openrouter.ai/api/v1"
    model: str = field(default_factory=lambda: os.getenv("OPENROUTER_MODEL", "google/gemini-2.5-flash-free"))
    max_retries: int = 3
    timeout: int = 60


@dataclass(frozen=True)
class EmbeddingConfig:
    model_name: str = "sentence-transformers/all-MiniLM-L6-v2"
    dimension: int = 384
    device: str = "cpu"


@dataclass(frozen=True)
class PathConfig:
    books_dir: str = field(
        default_factory=lambda: os.getenv(
            "BOOKS_DIR",
            "/home/jesus/knowledge_database/biblioteca_ia_rag/libros_ext4/books/",
        )
    )
    sandbox_dir: str = field(
        default_factory=lambda: os.getenv(
            "SANDBOX_DIR",
            "/home/jesus/knowledge_database/sandbox/",
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
