use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityMetric {
    Completeness,
    Freshness,
    Readability,
    Uniqueness,
    Relevance,
    Consistency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    pub document_id: String,
    pub overall_score: f64,
    pub metrics: Vec<MetricScore>,
    pub issues: Vec<QualityIssue>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricScore {
    pub metric: String,
    pub score: f64,
    pub weight: f64,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    pub severity: String,
    pub category: String,
    pub message: String,
    pub line: Option<usize>,
}

pub struct QualityScorer {
    weights: std::collections::HashMap<String, f64>,
}

impl QualityScorer {
    pub fn new() -> Self {
        let mut weights = std::collections::HashMap::new();
        weights.insert("completeness".to_string(), 0.25);
        weights.insert("freshness".to_string(), 0.2);
        weights.insert("readability".to_string(), 0.2);
        weights.insert("uniqueness".to_string(), 0.15);
        weights.insert("relevance".to_string(), 0.1);
        weights.insert("consistency".to_string(), 0.1);

        Self { weights }
    }

    pub fn score(&self, document_id: &str, content: &str, metadata: &std::collections::HashMap<String, String>) -> QualityReport {
        let mut metrics = Vec::new();
        let issues = Vec::new();

        let completeness = self.score_completeness(content, metadata);
        metrics.push(completeness.clone());

        let freshness = self.score_freshness(metadata);
        metrics.push(freshness.clone());

        let readability = self.score_readability(content);
        metrics.push(readability.clone());

        let uniqueness = self.score_uniqueness(content);
        metrics.push(uniqueness.clone());

        let relevance = self.score_relevance(content);
        metrics.push(relevance.clone());

        let consistency = self.score_consistency(content);
        metrics.push(consistency.clone());

        let overall_score = metrics.iter().map(|m| m.score * m.weight).sum::<f64>()
            / metrics.iter().map(|m| m.weight).sum::<f64>();

        let recommendations = self.generate_recommendations(&metrics, &issues);

        QualityReport {
            document_id: document_id.to_string(),
            overall_score,
            metrics,
            issues,
            recommendations,
        }
    }

    fn score_completeness(&self, content: &str, metadata: &std::collections::HashMap<String, String>) -> MetricScore {
        let mut score: f64 = 0.0;

        if content.len() > 100 {
            score += 0.3;
        }
        if content.len() > 500 {
            score += 0.2;
        }
        if metadata.contains_key("title") {
            score += 0.1;
        }
        if metadata.contains_key("tags") {
            score += 0.1;
        }
        if content.contains('#') {
            score += 0.1;
        }
        if content.lines().count() > 5 {
            score += 0.1;
        }
        if content.contains("```") {
            score += 0.1;
        }

        MetricScore {
            metric: "Completeness".to_string(),
            score: score.min(1.0),
            weight: *self.weights.get("completeness").unwrap_or(&0.25),
            details: format!("Content length: {} chars, {} lines", content.len(), content.lines().count()),
        }
    }

    fn score_freshness(&self, metadata: &std::collections::HashMap<String, String>) -> MetricScore {
        let score = if let Some(updated) = metadata.get("updated_at") {
            if updated.contains("2026") {
                1.0
            } else if updated.contains("2025") {
                0.8
            } else {
                0.5
            }
        } else {
            0.6
        };

        MetricScore {
            metric: "Freshness".to_string(),
            score,
            weight: *self.weights.get("freshness").unwrap_or(&0.2),
            details: "Based on last update timestamp".to_string(),
        }
    }

    fn score_readability(&self, content: &str) -> MetricScore {
        let lines = content.lines().collect::<Vec<&str>>();
        let avg_line_length = if lines.is_empty() {
            0.0
        } else {
            lines.iter().map(|l| l.len()).sum::<usize>() as f64 / lines.len() as f64
        };

        let paragraphs = content.split("\n\n").count();
        let has_headers = content.lines().any(|l| l.starts_with('#'));
        let has_code = content.contains("```");

        let mut score: f64 = 0.5;
        if avg_line_length < 100.0 {
            score += 0.15;
        }
        if paragraphs > 1 {
            score += 0.1;
        }
        if has_headers {
            score += 0.1;
        }
        if has_code {
            score += 0.1;
        }

        MetricScore {
            metric: "Readability".to_string(),
            score: score.min(1.0),
            weight: *self.weights.get("readability").unwrap_or(&0.2),
            details: format!("Avg line length: {:.1}, Paragraphs: {}", avg_line_length, paragraphs),
        }
    }

    fn score_uniqueness(&self, content: &str) -> MetricScore {
        let words: Vec<&str> = content.split_whitespace().collect();
        let unique_words: std::collections::HashSet<&str> = words.iter().cloned().collect();
        let ratio = if words.is_empty() {
            0.0
        } else {
            unique_words.len() as f64 / words.len() as f64
        };

        MetricScore {
            metric: "Uniqueness".to_string(),
            score: ratio,
            weight: *self.weights.get("uniqueness").unwrap_or(&0.15),
            details: format!("Unique words: {}/{}", unique_words.len(), words.len()),
        }
    }

    fn score_relevance(&self, content: &str) -> MetricScore {
        let keywords = ["implementation", "function", "error", "test", "example", "usage", "config"];
        let found = keywords.iter().filter(|k| content.to_lowercase().contains(*k)).count();
        let score = found as f64 / keywords.len() as f64;

        MetricScore {
            metric: "Relevance".to_string(),
            score,
            weight: *self.weights.get("relevance").unwrap_or(&0.1),
            details: format!("Found {}/{} keywords", found, keywords.len()),
        }
    }

    fn score_consistency(&self, content: &str) -> MetricScore {
        let has_frontmatter = content.starts_with("---");
        let has_headers = content.lines().any(|l| l.starts_with('#'));
        let has_lists = content.lines().any(|l| l.starts_with('-') || l.starts_with('*'));

        let mut score: f64 = 0.5;
        if has_frontmatter {
            score += 0.15;
        }
        if has_headers {
            score += 0.15;
        }
        if has_lists {
            score += 0.1;
        }
        if content.contains("```") {
            score += 0.1;
        }

        MetricScore {
            metric: "Consistency".to_string(),
            score: score.min(1.0),
            weight: *self.weights.get("consistency").unwrap_or(&0.1),
            details: format!("Frontmatter: {}, Headers: {}, Lists: {}", has_frontmatter, has_headers, has_lists),
        }
    }

    fn generate_recommendations(&self, metrics: &[MetricScore], issues: &[QualityIssue]) -> Vec<String> {
        let mut recommendations = Vec::new();

        for metric in metrics {
            if metric.score < 0.5 {
                match metric.metric.as_str() {
                    "Completeness" => recommendations.push("Add more content, headers, and code examples".to_string()),
                    "Freshness" => recommendations.push("Update the document with recent information".to_string()),
                    "Readability" => recommendations.push("Improve structure with shorter lines and paragraphs".to_string()),
                    "Uniqueness" => recommendations.push("Reduce repetition and add unique content".to_string()),
                    "Relevance" => recommendations.push("Add more relevant keywords and examples".to_string()),
                    "Consistency" => recommendations.push("Add frontmatter, headers, and consistent formatting".to_string()),
                    _ => {}
                }
            }
        }

        if issues.iter().any(|i| i.severity == "high") {
            recommendations.push("Address high-severity issues immediately".to_string());
        }

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_score_basic() {
        let scorer = QualityScorer::new();
        let metadata = std::collections::HashMap::new();
        let report = scorer.score("doc1", "This is a test document with some content.", &metadata);
        assert!(report.overall_score >= 0.0);
        assert!(report.overall_score <= 1.0);
    }

    #[test]
    fn test_quality_score_with_metadata() {
        let scorer = QualityScorer::new();
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("title".to_string(), "Test".to_string());
        metadata.insert("tags".to_string(), "test,doc".to_string());
        metadata.insert("updated_at".to_string(), "2026-07-19".to_string());
        let report = scorer.score("doc1", "Content with some structure", &metadata);
        assert!(report.overall_score > 0.0);
    }

    #[test]
    fn test_quality_score_empty() {
        let scorer = QualityScorer::new();
        let metadata = std::collections::HashMap::new();
        let report = scorer.score("doc1", "", &metadata);
        assert!(report.overall_score < 0.5);
    }

    #[test]
    fn test_quality_score_rich_content() {
        let scorer = QualityScorer::new();
        let content = "# Title\n\n## Section\n\nThis is a detailed implementation guide.\n\n```rust\nfn main() {}\n```\n\n- Item 1\n- Item 2";
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("title".to_string(), "Guide".to_string());
        metadata.insert("updated_at".to_string(), "2026-07-19".to_string());
        let report = scorer.score("doc1", content, &metadata);
        assert!(report.overall_score > 0.5);
    }

    #[test]
    fn test_recommendations_generated() {
        let scorer = QualityScorer::new();
        let metadata = std::collections::HashMap::new();
        let report = scorer.score("doc1", "short", &metadata);
        assert!(!report.recommendations.is_empty());
    }
}