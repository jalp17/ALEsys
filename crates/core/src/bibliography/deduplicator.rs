//! Citation deduplicator - removes duplicate bibliography entries
//! TICKET-30.5

use crate::bibliography::Citation;

pub struct CitationDeduplicator {
    threshold: f32,
}

impl Default for CitationDeduplicator {
    fn default() -> Self {
        Self::new()
    }
}

impl CitationDeduplicator {
    pub fn new() -> Self {
        Self { threshold: 0.8 }
    }

    pub fn with_threshold(threshold: f32) -> Self {
        Self { threshold }
    }

    /// Deduplicate citations based on similarity
    pub fn deduplicate(&self, citations: Vec<Citation>) -> Vec<Citation> {
        let mut result = Vec::new();
        for citation in citations {
            let mut is_dup = false;
            for existing in &result {
                if self.similarity(&citation, existing) > self.threshold {
                    is_dup = true;
                    break;
                }
            }
            if !is_dup {
                result.push(citation);
            }
        }
        result
    }

    /// Calculate similarity between two citations based on title, year, and DOI
    pub fn similarity(&self, a: &Citation, b: &Citation) -> f32 {
        let mut score = 0.0f32;

        if let (Some(t1), Some(t2)) = (&a.title, &b.title) {
            let t1_lower = t1.to_lowercase();
            let t2_lower = t2.to_lowercase();
            if t1_lower == t2_lower {
                score += 0.5;
            } else if t1_lower.contains(&t2_lower) || t2_lower.contains(&t1_lower) {
                score += 0.3;
            }
        }

        if let (Some(y1), Some(y2)) = (a.year, b.year) {
            if y1 == y2 {
                score += 0.3;
            }
        }

        if let (Some(d1), Some(d2)) = (&a.doi, &b.doi) {
            if d1 == d2 {
                score += 0.2;
            }
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use crate::bibliography::deduplicator::CitationDeduplicator;
    use crate::bibliography::Citation;
    use uuid::Uuid;

    fn make_citation(title: &str, year: u32, doi: Option<&str>) -> Citation {
        let mut c = Citation::new(format!("{} ({})", title, year), 1);
        c.title = Some(title.to_string());
        c.year = Some(year);
        if let Some(d) = doi {
            c.doi = Some(d.to_string());
        }
        c
    }

    #[test]
    fn test_identical_doi() {
        let d = CitationDeduplicator::default();
        let c1 = make_citation("Same Paper", 2023, Some("10.1234/test"));
        let c2 = make_citation("Same Paper", 2023, Some("10.1234/test"));

        assert!(d.similarity(&c1, &c2) > 0.9);
    }

    #[test]
    fn test_different_doi() {
        let d = CitationDeduplicator::default();
        let c1 = make_citation("Paper A", 2023, Some("10.1234/paper-a"));
        let c2 = make_citation("Paper B", 2023, Some("10.1234/paper-b"));

        assert!(d.similarity(&c1, &c2) < 0.5);
    }

    #[test]
    fn test_no_doi_similarity() {
        let d = CitationDeduplicator::default();
        let c1 = make_citation("Introduction to Physics", 2020, None);
        let c2 = make_citation("Introduction to Physics", 2020, None);

        assert!(d.similarity(&c1, &c2) > 0.7);
    }
}