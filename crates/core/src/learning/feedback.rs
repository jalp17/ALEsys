use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackEntry {
    pub id: String,
    pub suggestion_id: String,
    pub rating: FeedbackRating,
    pub comment: Option<String>,
    pub timestamp: i64,
    pub context: FeedbackContext,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FeedbackRating {
    Helpful,
    Neutral,
    Unhelpful,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackContext {
    pub file_path: Option<String>,
    pub language: Option<String>,
    pub suggestion_type: String,
    pub project_id: Option<String>,
}

pub struct FeedbackCollector {
    entries: Vec<FeedbackEntry>,
}

impl FeedbackCollector {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn collect(
        &mut self,
        suggestion_id: String,
        rating: FeedbackRating,
        comment: Option<String>,
        context: FeedbackContext,
    ) -> FeedbackEntry {
        let entry = FeedbackEntry {
            id: uuid::Uuid::new_v4().to_string(),
            suggestion_id,
            rating,
            comment,
            timestamp: chrono::Utc::now().timestamp(),
            context,
        };
        self.entries.push(entry.clone());
        entry
    }

    pub fn get_feedback_for_suggestion(&self, suggestion_id: &str) -> Vec<&FeedbackEntry> {
        self.entries
            .iter()
            .filter(|e| e.suggestion_id == suggestion_id)
            .collect()
    }

    pub fn get_feedback_by_type(&self, suggestion_type: &str) -> Vec<&FeedbackEntry> {
        self.entries
            .iter()
            .filter(|e| e.context.suggestion_type == suggestion_type)
            .collect()
    }

    pub fn get_all(&self) -> &[FeedbackEntry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn helpfulness_ratio(&self, suggestion_type: &str) -> f64 {
        let relevant = self.get_feedback_by_type(suggestion_type);
        if relevant.is_empty() {
            return 0.5;
        }
        let helpful = relevant
            .iter()
            .filter(|e| e.rating == FeedbackRating::Helpful)
            .count() as f64;
        helpful / relevant.len() as f64
    }
}

impl Default for FeedbackCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_feedback() {
        let mut collector = FeedbackCollector::new();
        let entry = collector.collect(
            "sug-1".to_string(),
            FeedbackRating::Helpful,
            Some("Great suggestion".to_string()),
            FeedbackContext {
                file_path: Some("main.rs".to_string()),
                language: Some("rust".to_string()),
                suggestion_type: "CodeSmell".to_string(),
                project_id: None,
            },
        );
        assert_eq!(collector.len(), 1);
        assert_eq!(entry.rating, FeedbackRating::Helpful);
    }

    #[test]
    fn test_helpfulness_ratio() {
        let mut collector = FeedbackCollector::new();
        for i in 0..5 {
            collector.collect(
                format!("sug-{}", i),
                FeedbackRating::Helpful,
                None,
                FeedbackContext {
                    file_path: None,
                    language: None,
                    suggestion_type: "TODO".to_string(),
                    project_id: None,
                },
            );
        }
        for i in 5..8 {
            collector.collect(
                format!("sug-{}", i),
                FeedbackRating::Unhelpful,
                None,
                FeedbackContext {
                    file_path: None,
                    language: None,
                    suggestion_type: "TODO".to_string(),
                    project_id: None,
                },
            );
        }
        let ratio = collector.helpfulness_ratio("TODO");
        assert!((ratio - 5.0 / 8.0).abs() < 0.01);
    }

    #[test]
    fn test_filter_by_type() {
        let mut collector = FeedbackCollector::new();
        collector.collect(
            "s1".to_string(),
            FeedbackRating::Helpful,
            None,
            FeedbackContext {
                file_path: None,
                language: None,
                suggestion_type: "TODO".to_string(),
                project_id: None,
            },
        );
        collector.collect(
            "s2".to_string(),
            FeedbackRating::Unhelpful,
            None,
            FeedbackContext {
                file_path: None,
                language: None,
                suggestion_type: "unwrap".to_string(),
                project_id: None,
            },
        );
        assert_eq!(collector.get_feedback_by_type("TODO").len(), 1);
        assert_eq!(collector.get_feedback_by_type("unwrap").len(), 1);
        assert_eq!(collector.get_feedback_by_type("missing").len(), 0);
    }
}
