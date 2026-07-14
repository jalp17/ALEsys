"""Pruebas unitarias para la segmentación de texto (chunking)."""

from pipeline import chunk_text


def test_chunk_text_empty():
    assert chunk_text("", 1000, 200) == []


def test_chunk_text_single_chunk():
    text = "Hola mundo"
    chunks = chunk_text(text, 1000, 200)
    assert len(chunks) == 1
    assert chunks[0][0] == text


def test_chunk_text_multiple_chunks():
    text = "A" * 2500
    chunks = chunk_text(text, 1000, 200)
    assert len(chunks) > 1


def test_chunk_text_overlap_capped():
    text = "A" * 2000
    chunks = chunk_text(text, 500, 600)
    assert all(isinstance(chunk, tuple) and len(chunk) == 2 for chunk in chunks)


def test_chunk_text_respects_newline():
    text = "Linea uno\nLinea dos\nLinea tres\n" + "A" * 5000
    chunks = chunk_text(text, 1000, 200)
    assert all(chunk[0].strip() for chunk in chunks)
