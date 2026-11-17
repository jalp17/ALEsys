-- Experiment runs tracking (ELN - Fase I+D)
-- Created: 2026-11-14

CREATE TABLE IF NOT EXISTS experiment_runs (
    id UUID PRIMARY KEY,
    project_id UUID REFERENCES research_projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    protocol TEXT,
    params JSONB NOT NULL DEFAULT '{}'::jsonb,
    status TEXT NOT NULL DEFAULT 'pending',
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    result_summary TEXT,
    created_by TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_experiment_runs_project_id ON experiment_runs(project_id);
CREATE INDEX IF NOT EXISTS idx_experiment_runs_status ON experiment_runs(status);
CREATE INDEX IF NOT EXISTS idx_experiment_runs_created_at ON experiment_runs(created_at DESC);
