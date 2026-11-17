-- Workspaces (cuadernos de temas) — inspired by AnythingLLM workspaces
-- Created: 2026-11-16

CREATE TABLE IF NOT EXISTS workspaces (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    llm_model TEXT,
    temperature DOUBLE PRECISION NOT NULL DEFAULT 0.7,
    top_p DOUBLE PRECISION NOT NULL DEFAULT 0.9,
    system_prompt TEXT,
    embedder_mode TEXT NOT NULL DEFAULT 'embed',  -- embed | attach | none
    owner_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workspaces_owner ON workspaces(owner_id);
CREATE INDEX IF NOT EXISTS idx_workspaces_created_at ON workspaces(created_at DESC);

CREATE TABLE IF NOT EXISTS workspace_documents (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    document_id INTEGER,
    title TEXT NOT NULL,
    mode TEXT NOT NULL DEFAULT 'embed',  -- embed | attach
    content TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workspace_documents_workspace ON workspace_documents(workspace_id);
CREATE INDEX IF NOT EXISTS idx_workspace_documents_mode ON workspace_documents(mode);

CREATE TABLE IF NOT EXISTS workspace_threads (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL DEFAULT 'New Thread',
    mode TEXT NOT NULL DEFAULT 'conversation',  -- conversation | query
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workspace_threads_workspace ON workspace_threads(workspace_id);
