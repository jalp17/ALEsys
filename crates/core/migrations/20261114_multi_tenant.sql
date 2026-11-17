-- Multi-tenant isolation columns (Fase I+D)
-- Created: 2026-11-14

ALTER TABLE research_projects ADD COLUMN IF NOT EXISTS owner_id TEXT;
ALTER TABLE research_notes ADD COLUMN IF NOT EXISTS owner_id TEXT;
ALTER TABLE research_synthesis ADD COLUMN IF NOT EXISTS owner_id TEXT;
ALTER TABLE experiment_runs ADD COLUMN IF NOT EXISTS owner_id TEXT;
ALTER TABLE ingestion_jobs ADD COLUMN IF NOT EXISTS owner_id TEXT;

CREATE INDEX IF NOT EXISTS idx_research_projects_owner ON research_projects(owner_id);
CREATE INDEX IF NOT EXISTS idx_research_notes_owner ON research_notes(owner_id);
CREATE INDEX IF NOT EXISTS idx_experiment_runs_owner ON experiment_runs(owner_id);
CREATE INDEX IF NOT EXISTS idx_ingestion_jobs_owner ON ingestion_jobs(owner_id);

-- RLS policies (enable per-tenant isolation when using a shared DB role)
-- Note: RLS requires SET app.current_user = <user> per session; application filters by owner_id.
ALTER TABLE research_projects ENABLE ROW LEVEL SECURITY;
ALTER TABLE research_notes ENABLE ROW LEVEL SECURITY;
ALTER TABLE research_synthesis ENABLE ROW LEVEL SECURITY;
ALTER TABLE experiment_runs ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS research_projects_owner ON research_projects;
CREATE POLICY research_projects_owner ON research_projects
    USING (owner_id = current_setting('app.current_user', true) OR current_setting('app.current_user', true) IS NULL);

DROP POLICY IF EXISTS research_notes_owner ON research_notes;
CREATE POLICY research_notes_owner ON research_notes
    USING (owner_id = current_setting('app.current_user', true) OR current_setting('app.current_user', true) IS NULL);

DROP POLICY IF EXISTS research_synthesis_owner ON research_synthesis;
CREATE POLICY research_synthesis_owner ON research_synthesis
    USING (owner_id = current_setting('app.current_user', true) OR current_setting('app.current_user', true) IS NULL);

DROP POLICY IF EXISTS experiment_runs_owner ON experiment_runs;
CREATE POLICY experiment_runs_owner ON experiment_runs
    USING (owner_id = current_setting('app.current_user', true) OR current_setting('app.current_user', true) IS NULL);
