-- Ingestion jobs tracking table
-- Created: 2026-07-26

CREATE TABLE IF NOT EXISTS ingestion_jobs (
    id UUID PRIMARY KEY,
    pdf_path TEXT NOT NULL,
    topic TEXT NOT NULL DEFAULT 'uncategorized',
    status TEXT NOT NULL DEFAULT 'pending',
    progress REAL NOT NULL DEFAULT 0.0,
    message TEXT,
    output_dir TEXT,
    markdown_path TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ingestion_jobs_status ON ingestion_jobs(status);
CREATE INDEX IF NOT EXISTS idx_ingestion_jobs_created_at ON ingestion_jobs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_ingestion_jobs_topic ON ingestion_jobs(topic);
