-- Bibliography module - create citations table
-- Created: 2024-12-01

CREATE TABLE IF NOT EXISTS bibliography_citations (
    id UUID PRIMARY KEY,
    citation_key TEXT UNIQUE,
    title TEXT NOT NULL,
    authors TEXT[] NOT NULL DEFAULT '{}',
    year INTEGER,
    journal TEXT,
    doi TEXT,
    isbn TEXT,
    url TEXT,
    pages TEXT,
    volume TEXT,
    issue TEXT,
    publisher TEXT,
    source_file TEXT,
    confidence REAL DEFAULT 0.0,
    stored_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_bibliography_year ON bibliography_citations(year DESC);
CREATE INDEX IF NOT EXISTS idx_bibliography_doi ON bibliography_citations(doi);
CREATE INDEX IF NOT EXISTS idx_bibliography_title_gin ON bibliography_citations USING gin(title gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_bibliography_authors_gin ON bibliography_citations USING gin(authors);

-- Trigger to auto-generate citation_key
CREATE OR REPLACE FUNCTION bibliography_set_citation_key() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.citation_key IS NULL OR NEW.citation_key = '' THEN
        NEW.citation_key := LOWER(regexp_replace(
            COALESCE(NEW.authors[1], 'unknown'), '[^a-z0-9]', '', 'g') || '_' ||
            COALESCE(NEW.year::text, 'nodate') || '_' ||
            substring(md5(NEW.title) from 1 for 8)
        );
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS set_citation_key ON bibliography_citations;
CREATE TRIGGER set_citation_key BEFORE INSERT OR UPDATE ON bibliography_citations
    FOR EACH ROW EXECUTE FUNCTION bibliography_set_citation_key();