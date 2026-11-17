-- Fix fragmentos table (was missing / wrong embedding dimension)
-- Embedder: Bge-M3 via llama.cpp HTTP = 1024 dimensions
-- Created: 2026-11-17

CREATE TABLE IF NOT EXISTS fragmentos (
    id SERIAL PRIMARY KEY,
    documento_id INTEGER REFERENCES documentos(id) ON DELETE CASCADE,
    contenido TEXT NOT NULL,
    embedding vector(1024),
    indice_orden INTEGER,
    creado_en TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_fragmentos_documento ON fragmentos(documento_id);
CREATE INDEX IF NOT EXISTS idx_fragmentos_embedding ON fragmentos USING hnsw (embedding vector_cosine_ops);

ALTER TABLE documentos ADD COLUMN IF NOT EXISTS contenido_hash VARCHAR(64);