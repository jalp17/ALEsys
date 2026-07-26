use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimilarityMethod {
    Exact,
    Fuzzy,
    Semantic,
    TokenOverlap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicatePair {
    pub doc_a_id: String,
    pub doc_b_id: String,
    pub similarity_score: f64,
    pub method: String,
    pub overlapping_sections: Vec<OverlapSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlapSection {
    pub section_a: String,
    pub section_b: String,
    pub overlap_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateReport {
    pub pairs: Vec<DuplicatePair>,
    pub total_checked: usize,
    pub duplicates_found: usize,
    pub method_used: String,
}

pub struct DuplicateDetector {
    threshold: f64,
    method: SimilarityMethod,
}

impl DuplicateDetector {
    pub fn new(threshold: f64, method: SimilarityMethod) -> Self {
        Self { threshold, method }
    }

    pub fn detect(&self, documents: &[super::merger::Document]) -> DuplicateReport {
        let mut pairs = Vec::new();

        for i in 0..documents.len() {
            for j in (i + 1)..documents.len() {
                let similarity = self.calculate_similarity(&documents[i].content, &documents[j].content);

                if similarity >= self.threshold {
                    pairs.push(DuplicatePair {
                        doc_a_id: documents[i].id.clone(),
                        doc_b_id: documents[j].id.clone(),
                        similarity_score: similarity,
                        method: format!("{:?}", self.method),
                        overlapping_sections: self.find_overlapping_sections(
                            &documents[i].content,
                            &documents[j].content,
                        ),
                    });
                }
            }
        }

        DuplicateReport {
            pairs,
            total_checked: documents.len(),
            duplicates_found: documents.len(),
            method_used: format!("{:?}", self.method),
        }
    }

    fn calculate_similarity(&self, a: &str, b: &str) -> f64 {
        match self.method {
            SimilarityMethod::Exact => {
                if a == b { 1.0 } else { 0.0 }
            }
            SimilarityMethod::Fuzzy => self.fuzzy_similarity(a, b),
            SimilarityMethod::TokenOverlap => self.token_overlap(a, b),
            SimilarityMethod::Semantic => self.semantic_similarity(a, b),
        }
    }

    fn fuzzy_similarity(&self, a: &str, b: &str) -> f64 {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();

        if a_lower == b_lower {
            return 1.0;
        }

        let a_chars: Vec<char> = a_lower.chars().collect();
        let b_chars: Vec<char> = b_lower.chars().collect();

        let matches = a_chars.iter().filter(|c| b_chars.contains(c)).count();
        let total = std::cmp::max(a_chars.len(), b_chars.len());

        if total == 0 {
            0.0
        } else {
            matches as f64 / total as f64
        }
    }

    fn token_overlap(&self, a: &str, b: &str) -> f64 {
        let a_tokens: Vec<&str> = a.split_whitespace().collect();
        let b_tokens: Vec<&str> = b.split_whitespace().collect();

        if a_tokens.is_empty() || b_tokens.is_empty() {
            return 0.0;
        }

        let overlap = a_tokens.iter().filter(|t| b_tokens.contains(t)).count();
        let total = a_tokens.len() + b_tokens.len() - overlap;

        if total == 0 {
            0.0
        } else {
            overlap as f64 / total as f64
        }
    }

    fn semantic_similarity(&self, a: &str, b: &str) -> f64 {
        self.fuzzy_similarity(a, b)
    }

    fn find_overlapping_sections(&self, a: &str, b: &str) -> Vec<OverlapSection> {
        let mut sections = Vec::new();
        let a_lines: Vec<&str> = a.lines().collect();
        let b_lines: Vec<&str> = b.lines().collect();

        let mut overlapping_a = Vec::new();
        let mut overlapping_b = Vec::new();

        for (_, line_a) in a_lines.iter().enumerate() {
            for (_, line_b) in b_lines.iter().enumerate() {
                if line_a.trim() == line_b.trim() && !line_a.trim().is_empty() {
                    overlapping_a.push(line_a.to_string());
                    overlapping_b.push(line_b.to_string());
                }
            }
        }

        if !overlapping_a.is_empty() {
            sections.push(OverlapSection {
                section_a: overlapping_a.join("\n"),
                section_b: overlapping_b.join("\n"),
                overlap_ratio: overlapping_a.len() as f64 / a_lines.len() as f64,
            });
        }

        sections
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge_curation::merger::Document;

    fn make_doc(id: &str, content: &str) -> Document {
        Document {
            id: id.to_string(),
            title: format!("Doc {}", id),
            content: content.to_string(),
            tags: vec![],
            metadata: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_exact_duplicate() {
        let detector = DuplicateDetector::new(1.0, SimilarityMethod::Exact);
        let docs = vec![
            make_doc("1", "identical content"),
            make_doc("2", "identical content"),
        ];
        let report = detector.detect(&docs);
        assert_eq!(report.pairs.len(), 1);
        assert_eq!(report.pairs[0].similarity_score, 1.0);
    }

    #[test]
    fn test_no_duplicate() {
        let detector = DuplicateDetector::new(0.9, SimilarityMethod::Exact);
        let docs = vec![
            make_doc("1", "completely different"),
            make_doc("2", "totally unrelated"),
        ];
        let report = detector.detect(&docs);
        assert_eq!(report.pairs.len(), 0);
    }

    #[test]
    fn test_fuzzy_duplicate() {
        let detector = DuplicateDetector::new(0.7, SimilarityMethod::Fuzzy);
        let docs = vec![
            make_doc("1", "Hello World"),
            make_doc("2", "Hello World!"),
        ];
        let report = detector.detect(&docs);
        assert!(report.pairs.len() > 0);
    }

    #[test]
    fn test_token_overlap() {
        let detector = DuplicateDetector::new(0.3, SimilarityMethod::TokenOverlap);
        let docs = vec![
            make_doc("1", "the quick brown fox"),
            make_doc("2", "the quick brown dog"),
        ];
        let report = detector.detect(&docs);
        assert!(report.pairs.len() > 0);
    }

    #[test]
    fn test_empty_documents() {
        let detector = DuplicateDetector::new(0.5, SimilarityMethod::Fuzzy);
        let docs = vec![];
        let report = detector.detect(&docs);
        assert_eq!(report.pairs.len(), 0);
    }
}