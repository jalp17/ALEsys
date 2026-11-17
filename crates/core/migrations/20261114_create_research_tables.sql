-- Research module tables (Phase 31 - persistent storage)
-- Created: 2026-11-14

CREATE TABLE IF NOT EXISTS research_projects (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_research_projects_status ON research_projects(status);
CREATE INDEX IF NOT EXISTS idx_research_projects_created_at ON research_projects(created_at DESC);

CREATE TABLE IF NOT EXISTS research_notes (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES research_projects(id) ON DELETE CASCADE,
    chapter_id UUID,
    citation_id TEXT,
    title TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    note_type TEXT NOT NULL DEFAULT 'idea',
    tags TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_research_notes_project_id ON research_notes(project_id);
CREATE INDEX IF NOT EXISTS idx_research_notes_chapter_id ON research_notes(chapter_id);
CREATE INDEX IF NOT EXISTS idx_research_notes_citation_id ON research_notes(citation_id);
CREATE INDEX IF NOT EXISTS idx_research_notes_note_type ON research_notes(note_type);

CREATE TABLE IF NOT EXISTS research_synthesis (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL UNIQUE REFERENCES research_projects(id) ON DELETE CASCADE,
    title TEXT NOT NULL DEFAULT 'Synthesis',
    content TEXT NOT NULL DEFAULT '',
    citation_style TEXT NOT NULL DEFAULT 'apa',
    citations JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_research_synthesis_project_id ON research_synthesis(project_id);

-- Audit columns (added 2026-11-14)
ALTER TABLE research_projects ADD COLUMN IF NOT EXISTS created_by TEXT;
ALTER TABLE research_projects ADD COLUMN IF NOT EXISTS updated_by TEXT;
ALTER TABLE research_notes ADD COLUMN IF NOT EXISTS created_by TEXT;
ALTER TABLE research_notes ADD COLUMN IF NOT EXISTS updated_by TEXT;
ALTER TABLE research_synthesis ADD COLUMN IF NOT EXISTS updated_by TEXT;
