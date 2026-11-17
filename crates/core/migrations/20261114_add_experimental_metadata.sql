-- Add experimental metadata columns to documentos table (Fase I+D)
-- Created: 2026-11-14

ALTER TABLE documentos ADD COLUMN IF NOT EXISTS experiment_id TEXT;
ALTER TABLE documentos ADD COLUMN IF NOT EXISTS trial_id TEXT;
ALTER TABLE documentos ADD COLUMN IF NOT EXISTS area TEXT;
ALTER TABLE documentos ADD COLUMN IF NOT EXISTS section TEXT;
ALTER TABLE documentos ADD COLUMN IF NOT EXISTS source_type TEXT;
ALTER TABLE documentos ADD COLUMN IF NOT EXISTS data_type TEXT;
ALTER TABLE documentos ADD COLUMN IF NOT EXISTS topic TEXT;
ALTER TABLE documentos ADD COLUMN IF NOT EXISTS session_id UUID;
ALTER TABLE documentos ADD COLUMN IF NOT EXISTS source_pdf TEXT;
ALTER TABLE documentos ADD COLUMN IF NOT EXISTS chapter_title TEXT;

CREATE INDEX IF NOT EXISTS idx_documentos_experiment_id ON documentos(experiment_id);
CREATE INDEX IF NOT EXISTS idx_documentos_source_type ON documentos(source_type);
CREATE INDEX IF NOT EXISTS idx_documentos_data_type ON documentos(data_type);
CREATE INDEX IF NOT EXISTS idx_documentos_topic ON documentos(topic);
