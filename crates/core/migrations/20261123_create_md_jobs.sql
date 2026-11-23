-- Markdown ingestion job persistence for crash recovery
CREATE TABLE IF NOT EXISTS markdown_ingestion_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dir TEXT NOT NULL,
    topic_override TEXT,
    status TEXT NOT NULL DEFAULT 'queued',
    files_done INTEGER NOT NULL DEFAULT 0,
    files_total INTEGER NOT NULL DEFAULT 0,
    chunks_indexed INTEGER NOT NULL DEFAULT 0,
    files_indexed INTEGER NOT NULL DEFAULT 0,
    skipped_empty INTEGER NOT NULL DEFAULT 0,
    topics JSONB DEFAULT '[]'::jsonb,
    current_file TEXT,
    current_topic TEXT,
    message TEXT,
    error TEXT,
    result JSONB,
    owner_id TEXT,
    cancelled BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_md_jobs_status ON markdown_ingestion_jobs(status);
CREATE INDEX IF NOT EXISTS idx_md_jobs_created ON markdown_ingestion_jobs(created_at DESC);
