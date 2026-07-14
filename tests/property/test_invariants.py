"""Property tests para invariantes clave del proyecto."""

from __future__ import annotations

from pathlib import Path

import pytest

from pipeline import chunk_text
from test_queries import graph_search, hybrid_search, vector_search


def test_chunk_text_never_returns_empty_strings() -> None:
    text = "texto" * 10000
    chunks = chunk_text(text, 1000, 200)
    assert all(chunk[0].strip() for chunk in chunks)


def test_chunk_text_indices_are_sequential() -> None:
    text = "texto" * 10000
    chunks = chunk_text(text, 1000, 200)
    assert [c[1] for c in chunks] == list(range(len(chunks)))


def test_graph_search_result_shape() -> None:
    result = graph_search("nonexistent-entity-12345", limit=1)
    assert "entities" in result
    assert "relations" in result
    assert isinstance(result["entities"], list)
    assert isinstance(result["relations"], list)


def test_hybrid_search_returns_list() -> None:
    result = hybrid_search("query without meaning", top_k=2)
    assert isinstance(result, list)


def test_vector_search_returns_expected_keys() -> None:
    result = vector_search("campo eléctrico", top_k=2)
    if result:
        assert "filename" in result[0]
        assert "similarity" in result[0]
        assert "content" in result[0]
