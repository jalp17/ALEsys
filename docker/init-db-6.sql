-- Fase 6: Advanced Search - Performance indices
-- Ejecutar después de init-db.sql

-- Index for document type filtering
CREATE INDEX IF NOT EXISTS idx_documentos_tipo ON documentos(tipo);

-- Index for area/subarea filtering
CREATE INDEX IF NOT EXISTS idx_documentos_area ON documentos(area_id);
CREATE INDEX IF NOT EXISTS idx_documentos_subarea ON documentos(subarea_id);

-- Composite index for date range queries
CREATE INDEX IF NOT EXISTS idx_documentos_fecha ON documentos(creado_en);

-- Composite index for multi-filter queries
CREATE INDEX IF NOT EXISTS idx_documentos_tipo_area ON documentos(tipo, area_id);
CREATE INDEX IF NOT EXISTS idx_documentos_tipo_fecha ON documentos(tipo, CREATED_EN);

-- Fragment content index for full-text search (GIN index for LIKE queries)
CREATE INDEX IF NOT EXISTS idx_fragmentos_contenido ON fragmentos USING gin(to_tsvector('spanish', contenido));

-- tsvector column for fast full-text search
ALTER TABLE fragmentos ADD COLUMN IF NOT EXISTS search_vector tsvector
    GENERATED ALWAYS AS (to_tsvector('spanish', contenido)) STORED;

-- GIN index on tsvector for fast full-text search
CREATE INDEX IF NOT EXISTS idx_fragmentos_search_vector ON fragmentos USING gin(search_vector);

-- Composite index for search results ordering
CREATE INDEX IF NOT EXISTS idx_fragmentos_documento_orden ON fragmentos(documento_id, indice_orden);

-- Area/Subarea indices for filtering
CREATE INDEX IF NOT EXISTS idx_documentos_area_tipo ON documentos(area_id, tipo);
CREATE INDEX IF NOT EXISTS idx_documentos_subarea_tipo ON documentos(subarea_id, tipo);

-- Permission check optimization
CREATE INDEX IF NOT EXISTS idx_graph_permissions_user_doc ON graph_permissions(user_id, doc_id);
