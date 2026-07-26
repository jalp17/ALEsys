-- TICKET-30.4: Bibliography citations table
CREATE TABLE IF NOT EXISTS bibliography_citations (
    id UUID PRIMARY KEY,
    title TEXT,
    authors TEXT[] NOT NULL DEFAULT '{}',
    journal TEXT,
    year INTEGER,
    doi TEXT,
    isbn TEXT,
    url TEXT,
    pages TEXT,
    volume TEXT,
    issue TEXT,
    publisher TEXT,
    raw_text TEXT NOT NULL,
    cited_in_chapter UUID,
    cited_page INTEGER NOT NULL DEFAULT 1,
    confidence REAL NOT NULL DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_bibliography_citations_chapter
    ON bibliography_citations (cited_in_chapter);

CREATE INDEX IF NOT EXISTS idx_bibliography_citations_doi
    ON bibliography_citations (doi);
