//! Bibliographic extraction module - Fase 30
//! Extracts citations, references, and bibliographic metadata from academic PDFs

pub mod extractor;
pub mod formatter;
pub mod deduplicator;
pub mod storage;
pub mod models;

pub use extractor::BibliographyResult;
pub use extractor::CitationExtractor;
pub use formatter::{CitationFormatter, FormatError};
pub use models::Citation;
pub use models::ExtractionResult;
pub use storage::CitationStorage;
pub use deduplicator::CitationDeduplicator;

/// Configuration for citation extraction
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CitationExtractorConfig {
    pub extract_bibliography: bool,
    pub extract_inline_citations: bool,
    pub citation_style: CitationStyle,
    pub verify_doi: bool,
    pub crossref_api_key: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum CitationStyle {
    APA,
    MLA,
    Chicago,
    IEEE,
    Unknown,
}

pub type Result<T> = std::result::Result<T, CitationError>;

impl From<std::io::Error> for CitationError {
    fn from(e: std::io::Error) -> Self {
        CitationError::Io(e)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CitationError {
    #[error("Failed to parse citation: {0}")]
    ParseFailed(String),
    
    #[error("DOI verification failed: {0}")]
    DoiVerification(String),
    
    #[error("IO error: {0}")]
    Io(std::io::Error),
}