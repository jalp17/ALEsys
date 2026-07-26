//! Bibliography data models

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub id: Uuid,
    pub title: Option<String>,
    pub authors: Vec<String>,
    pub journal: Option<String>,
    pub year: Option<u32>,
    pub doi: Option<String>,
    pub isbn: Option<String>,
    pub url: Option<String>,
    pub pages: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub publisher: Option<String>,
    pub raw_text: String,
    pub cited_in_chapter: Option<Uuid>,
    pub cited_page: u32,
    pub confidence: f32,
}

impl Citation {
    pub fn new(raw_text: String, cited_page: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: None,
            authors: vec![],
            journal: None,
            year: None,
            doi: None,
            isbn: None,
            url: None,
            pages: None,
            volume: None,
            issue: None,
            publisher: None,
            raw_text,
            cited_in_chapter: None,
            cited_page,
            confidence: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub citations: Vec<Citation>,
    pub total_found: usize,
    pub processing_time_ms: u64,
    pub errors: Vec<String>,
}