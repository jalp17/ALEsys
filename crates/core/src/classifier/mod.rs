//! Document classifier for scientific I+D (Fase I+D)
//!
//! Uses the loaded LLM with structured JSON output to classify
//! documents and extract experimental metadata.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    ResearchPaper,
    TechnicalReport,
    Thesis,
    Dataset,
    CodeRepository,
    Protocol,
    Patent,
    Presentation,
    LabNotebook,
    RegulatoryDoc,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    Table,
    Figure,
    Numeric,
    Text,
    Code,
    Formula,
    Reference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionMeta {
    pub name: String,
    pub area: Option<String>,
    pub experiment_id: Option<String>,
    pub trial_id: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub doc_type: DocumentType,
    pub confidence: f32,
    pub language: String,
    pub sections: Vec<SectionMeta>,
    pub data_types: Vec<DataType>,
    pub keywords: Vec<String>,
    pub summary: String,
}

impl Default for ClassificationResult {
    fn default() -> Self {
        Self {
            doc_type: DocumentType::Other,
            confidence: 0.0,
            language: "unknown".to_string(),
            sections: vec![],
            data_types: vec![],
            keywords: vec![],
            summary: String::new(),
        }
    }
}

pub struct DocumentClassifier;

impl DocumentClassifier {
    pub fn build_prompt(markdown: &str, max_chars: usize) -> String {
        let truncated = if markdown.len() > max_chars {
            &markdown[..max_chars]
        } else {
            markdown
        };
        format!(
            r#"You are a scientific document classifier. Analyze the following document and return a JSON object with this exact structure:
{{
  "doc_type": "research_paper|technical_report|thesis|dataset|code_repository|protocol|patent|presentation|lab_notebook|regulatory_doc|other",
  "confidence": 0.0-1.0,
  "language": "iso language code",
  "sections": [{{"name": "...", "area": "...", "experiment_id": "...", "trial_id": "...", "timestamp": "..."}}],
  "data_types": ["table","figure","numeric","text","code","formula","reference"],
  "keywords": ["..."],
  "summary": "one sentence summary"
}}
Only output valid JSON, no markdown fences.

Document:
{}"#,
            truncated
        )
    }

    pub fn parse_response(text: &str) -> ClassificationResult {
        // Strip markdown fences if present
        let cleaned = text.trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        serde_json::from_str(cleaned).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_json() {
        let json = r#"{"doc_type":"research_paper","confidence":0.9,"language":"en","sections":[{"name":"Intro","area":"ML","experiment_id":"exp1","trial_id":null,"timestamp":null}],"data_types":["table","figure"],"keywords":["rag","llm"],"summary":"A paper about RAG."}"#;
        let r = DocumentClassifier::parse_response(json);
        assert_eq!(r.doc_type, DocumentType::ResearchPaper);
        assert!((r.confidence - 0.9).abs() < 1e-6);
        assert_eq!(r.data_types.len(), 2);
        assert_eq!(r.keywords, vec!["rag", "llm"]);
    }

    #[test]
    fn test_parse_with_fences() {
        let json = "```json\n{\"doc_type\":\"dataset\",\"confidence\":0.8,\"language\":\"es\",\"sections\":[],\"data_types\":[\"table\"],\"keywords\":[],\"summary\":\"Datos.\"}\n```";
        let r = DocumentClassifier::parse_response(json);
        assert_eq!(r.doc_type, DocumentType::Dataset);
    }

    #[test]
    fn test_parse_invalid_returns_default() {
        let r = DocumentClassifier::parse_response("not json");
        assert_eq!(r.doc_type, DocumentType::Other);
        assert_eq!(r.confidence, 0.0);
    }

    #[test]
    fn test_build_prompt_truncates() {
        let long = "a".repeat(10000);
        let p = DocumentClassifier::build_prompt(&long, 100);
        assert!(p.len() < 10000);
        assert!(p.contains("Document:"));
    }
}
