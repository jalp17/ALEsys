"""Configuración y fixtures compartidas para las pruebas."""

import json
import os
import sys
import types
from pathlib import Path
from unittest.mock import MagicMock

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

tests_dir = Path(__file__).resolve().parent

fake_config = types.SimpleNamespace(
    DB=types.SimpleNamespace(
        host="127.0.0.1",
        port=5433,
        dbname="alesys_test",
        user="alesys",
        password="alesys",
    ),
    OPENROUTER=types.SimpleNamespace(
        api_key="test-openrouter-key",
        base_url="https://openrouter.ai/api/v1",
        model="test-model",
        max_retries=1,
        timeout=5,
    ),
    EMBEDDING=types.SimpleNamespace(
        backend="sentence-transformers",
        model_name="sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2",
        dimension=384,
        device="cpu",
        gguf_model_path="",
        n_gpu_layers=-1,
        n_ctx=8192,
        n_batch=512,
        use_vulkan=False,
        llama_cpp_lib_path="",
    ),
    PATHS=types.SimpleNamespace(
        books_dir=str(tests_dir / "fixtures" / "books"),
        sandbox_dir=str(tests_dir / "fixtures" / "sandbox"),
    ),
    CHUNKING=types.SimpleNamespace(size=1000, overlap=200),
)
config_mod = types.ModuleType("config")
config_mod.DB = fake_config.DB
config_mod.OPENROUTER = fake_config.OPENROUTER
config_mod.EMBEDDING = fake_config.EMBEDDING
config_mod.PATHS = fake_config.PATHS
config_mod.CHUNKING = fake_config.CHUNKING
sys.modules.setdefault("config", config_mod)


@pytest.fixture
def mock_embedder():
    embedder = MagicMock()
    embedder.encode.return_value = [0.0] * fake_config.EMBEDDING.dimension
    embedder.encode_batch.return_value = [[0.0] * fake_config.EMBEDDING.dimension, [0.0] * fake_config.EMBEDDING.dimension]
    embedder.unload.return_value = None
    return embedder


@pytest.fixture
def mock_extractor():
    extractor = MagicMock()
    extractor.extract.return_value = {
        "entidades": [{"nombre": "Faraday", "tipo": "científico"}],
        "relaciones": [
            {"origen": "Faraday", "destino": "inducción electromagnética", "tipo": "descubrió"}
        ],
    }
    extractor.answer.return_value = "Respuesta de prueba"
    extractor.close.return_value = None
    return extractor


@pytest.fixture
def tmp_env(tmp_path, monkeypatch):
    books = tmp_path / "books"
    books.mkdir()
    sample = books / "sample.md"
    sample.write_text("# Título\n\nContenido de prueba con varios conceptos científicos.\n\n## Sección\n\nTexto adicional para chunks.\n" * 5, encoding="utf-8")
    fake_config.PATHS = types.SimpleNamespace(books_dir=str(books), sandbox_dir=str(tmp_path / "sandbox"))
    return tmp_path
