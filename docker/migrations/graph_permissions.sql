-- Migration: graph_permissions
-- Fase 5: Control de acceso jerárquico para el grafo
-- Admin (user_id=0) tiene acceso a todo. Otros usuarios solo a documentos asignados.

CREATE TABLE IF NOT EXISTS graph_permissions (
    user_id INTEGER NOT NULL DEFAULT 0,
    doc_id INTEGER NOT NULL REFERENCES documentos(id) ON DELETE CASCADE,
    permission VARCHAR(20) NOT NULL DEFAULT 'read' CHECK (permission IN ('read', 'write', 'admin')),
    granted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    granted_by INTEGER DEFAULT 0,
    PRIMARY KEY (user_id, doc_id)
);

CREATE INDEX IF NOT EXISTS idx_graph_permissions_user ON graph_permissions(user_id);
CREATE INDEX IF NOT EXISTS idx_graph_permissions_doc ON graph_permissions(doc_id);

-- Grant permissions
GRANT ALL PRIVILEGES ON graph_permissions TO alesys;
