//! Bibliography storage - TICKET-30.4
//! PostgreSQL persistence for extracted citations

use crate::bibliography::{Citation, Result, CitationError};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct CitationStorage {
    pool: PgPool,
}

impl CitationStorage {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn store(&self, citation: &Citation) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO bibliography_citations (
                id, title, authors, journal, year, doi, isbn, url,
                pages, volume, issue, publisher, raw_text,
                cited_in_chapter, cited_page, confidence
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8,
                $9, $10, $11, $12, $13,
                $14, $15, $16
            )
            ON CONFLICT (id) DO UPDATE SET
                title = EXCLUDED.title,
                authors = EXCLUDED.authors,
                journal = EXCLUDED.journal,
                year = EXCLUDED.year,
                doi = EXCLUDED.doi,
                isbn = EXCLUDED.isbn,
                url = EXCLUDED.url,
                pages = EXCLUDED.pages,
                volume = EXCLUDED.volume,
                issue = EXCLUDED.issue,
                publisher = EXCLUDED.publisher,
                raw_text = EXCLUDED.raw_text,
                cited_in_chapter = EXCLUDED.cited_in_chapter,
                cited_page = EXCLUDED.cited_page,
                confidence = EXCLUDED.confidence
            "#
        )
        .bind(citation.id)
        .bind(&citation.title)
        .bind(&citation.authors)
        .bind(&citation.journal)
        .bind(citation.year.map(|y| y as i32))
        .bind(&citation.doi)
        .bind(&citation.isbn)
        .bind(&citation.url)
        .bind(&citation.pages)
        .bind(&citation.volume)
        .bind(&citation.issue)
        .bind(&citation.publisher)
        .bind(&citation.raw_text)
        .bind(citation.cited_in_chapter)
        .bind(citation.cited_page as i32)
        .bind(citation.confidence)
        .execute(&self.pool)
        .await
        .map_err(|e| CitationError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e))))?;

        Ok(())
    }

    pub async fn list_by_chapter(&self, chapter_id: Uuid, limit: i64, offset: i64) -> Result<Vec<Citation>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, title, authors, journal, year, doi, isbn, url,
                pages, volume, issue, publisher, raw_text,
                cited_in_chapter, cited_page, confidence
            FROM bibliography_citations
            WHERE cited_in_chapter = $1
            ORDER BY cited_page ASC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(chapter_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CitationError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e))))?;

        let mut citations = Vec::with_capacity(rows.len());
        for row in rows {
            let id: Uuid = row.try_get("id").map_err(io_err)?;
            let title: Option<String> = row.try_get("title").map_err(io_err)?;
            let authors: Vec<String> = row.try_get("authors").map_err(io_err)?;
            let journal: Option<String> = row.try_get("journal").map_err(io_err)?;
            let year: Option<i32> = row.try_get("year").map_err(io_err)?;
            let doi: Option<String> = row.try_get("doi").map_err(io_err)?;
            let isbn: Option<String> = row.try_get("isbn").map_err(io_err)?;
            let url: Option<String> = row.try_get("url").map_err(io_err)?;
            let pages: Option<String> = row.try_get("pages").map_err(io_err)?;
            let volume: Option<String> = row.try_get("volume").map_err(io_err)?;
            let issue: Option<String> = row.try_get("issue").map_err(io_err)?;
            let publisher: Option<String> = row.try_get("publisher").map_err(io_err)?;
            let raw_text: String = row.try_get("raw_text").map_err(io_err)?;
            let cited_in_chapter: Option<Uuid> = row.try_get("cited_in_chapter").map_err(io_err)?;
            let cited_page: i32 = row.try_get("cited_page").map_err(io_err)?;
            let confidence: f32 = row.try_get("confidence").map_err(io_err)?;

            citations.push(Citation {
                id,
                title,
                authors,
                journal,
                year: year.map(|y| y as u32),
                doi,
                isbn,
                url,
                pages,
                volume,
                issue,
                publisher,
                raw_text,
                cited_in_chapter,
                cited_page: cited_page as u32,
                confidence,
            });
        }

        Ok(citations)
    }
}

fn io_err(e: sqlx::Error) -> CitationError {
    CitationError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_creation() {
        let _ = CitationStorage::new;
    }
}
