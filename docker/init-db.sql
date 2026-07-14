# Database initialization for ALEsys Core (PostgreSQL + pgvector)

-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Create tables for GraphRAG
CREATE TABLE IF NOT EXISTS documentos (
    id SERIAL PRIMARY KEY,
    ruta_relativa VARCHAR(500) UNIQUE NOT NULL,
    tipo VARCHAR(50) NOT NULL,
    area_id INTEGER,
    subarea_id INTEGER,
    frontmatter JSONB NOT NULL,
    contenido_hash VARCHAR(64),
    creado_en TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS fragmentos (
    id SERIAL PRIMARY KEY,
    documento_id INTEGER REFERENCES documentos(id) ON DELETE CASCADE,
    contenido TEXT NOT NULL,
    embedding vector(384),
    indice_orden INTEGER,
    creado_en TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS entidades (
    id SERIAL PRIMARY KEY,
    documento_id INTEGER REFERENCES documentos(id) ON DELETE CASCADE,
    nombre VARCHAR(200) NOT NULL,
    tipo VARCHAR(50) NOT NULL,
    metadata JSONB,
    creado_en TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS relaciones (
    id SERIAL PRIMARY KEY,
    origen_id INTEGER REFERENCES entidades(id) ON DELETE CASCADE,
    destino_id INTEGER REFERENCES entidades(id) ON DELETE CASCADE,
    tipo VARCHAR(50) NOT NULL,
    metadata JSONB,
    creado_en TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS enlaces (
    origen_id INTEGER REFERENCES documentos(id) ON DELETE CASCADE,
    destino_id INTEGER REFERENCES documentos(id) ON DELETE CASCADE,
    tipo_enlace VARCHAR(100),
    contexto TEXT,
    PRIMARY KEY (origen_id, destino_id, tipo_enlace)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_fragmentos_documento ON fragmentos(documento_id);
CREATE INDEX IF NOT EXISTS idx_fragmentos_embedding ON fragmentos USING hnsw (embedding vector_cosine_ops);
CREATE INDEX IF NOT EXISTS idx_entidades_documento ON entidades(documento_id);
CREATE INDEX IF NOT EXISTS idx_enlaces_origen ON enlaces(origen_id);
CREATE INDEX IF NOT EXISTS idx_enlaces_destino ON enlaces(destino_id);

-- Grant permissions
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO alesys;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO alesys;