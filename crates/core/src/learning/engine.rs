use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::feedback::{FeedbackCollector, FeedbackRating, FeedbackContext};
use super::memory::{ContextualMemory, ContextType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningInsight {
    pub insight_type: InsightType,
    pub description: String,
    pub confidence: f64,
    pub based_on_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InsightType {
    LanguagePreference,
    SuggestionPreference,
    RefactorPattern,
    CommonIssue,
}

pub struct LearningEngine {
    pub feedback: FeedbackCollector,
    pub memory: ContextualMemory,
    scores: HashMap<String, f64>,
}

impl LearningEngine {
    pub fn new() -> Self {
        Self {
            feedback: FeedbackCollector::new(),
            memory: ContextualMemory::new(),
            scores: HashMap::new(),
        }
    }

    pub fn record_feedback(
        &mut self,
        suggestion_id: &str,
        rating: FeedbackRating,
        suggestion_type: &str,
        file_path: Option<&str>,
        language: Option<&str>,
    ) {
        let context = FeedbackContext {
            file_path: file_path.map(|s| s.to_string()),
            language: language.map(|s| s.to_string()),
            suggestion_type: suggestion_type.to_string(),
            project_id: None,
        };
        let adjustment = match &rating {
            FeedbackRating::Helpful => 0.1,
            FeedbackRating::Unhelpful => -0.1,
            FeedbackRating::Neutral => 0.0,
        };
        self.feedback.collect(
            suggestion_id.to_string(),
            rating,
            None,
            context,
        );

        let key = format!("type:{}", suggestion_type);
        let score = self.scores.get(&key).copied().unwrap_or(0.5);
        self.scores.insert(key, (score + adjustment).clamp(0.0, 1.0));
    }

    pub fn get_type_score(&self, suggestion_type: &str) -> f64 {
        let key = format!("type:{}", suggestion_type);
        self.scores.get(&key).copied().unwrap_or(0.5)
    }

    pub fn generate_insights(&self) -> Vec<LearningInsight> {
        let mut insights = Vec::new();

        for (key, score) in &self.scores {
            if key.starts_with("type:") {
                let suggestion_type = &key[5..];
                let ratio = self.feedback.helpfulness_ratio(suggestion_type);
                let count = self.feedback.get_feedback_by_type(suggestion_type).len();
                if count >= 3 {
                    insights.push(LearningInsight {
                        insight_type: InsightType::SuggestionPreference,
                        description: format!(
                            "Suggestion type '{}' has {:.0}% helpfulness based on {} ratings",
                            suggestion_type,
                            ratio * 100.0,
                            count
                        ),
                        confidence: *score,
                        based_on_count: count,
                    });
                }
            }
        }

        let language_entries = self.memory.query_by_type(&ContextType::Language);
        if language_entries.len() >= 3 {
            let mut lang_counts: HashMap<&str, usize> = HashMap::new();
            for entry in &language_entries {
                *lang_counts.entry(&entry.value).or_insert(0) += 1;
            }
            if let Some((lang, count)) = lang_counts.iter().max_by_key(|(_, c)| **c) {
                insights.push(LearningInsight {
                    insight_type: InsightType::LanguagePreference,
                    description: format!(
                        "Most used language: {} ({} occurrences)",
                        lang, count
                    ),
                    confidence: *count as f64 / language_entries.len() as f64,
                    based_on_count: *count,
                });
            }
        }

        insights
    }

    pub fn store_context(&mut self, key: &str, value: &str, context_type: ContextType) {
        self.memory.store(
            key.to_string(),
            value.to_string(),
            context_type,
        );
    }
}

impl Default for LearningEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learning_engine_feedback() {
        let mut engine = LearningEngine::new();
        engine.record_feedback(
            "s1",
            FeedbackRating::Helpful,
            "TODO",
            Some("main.rs"),
            Some("rust"),
        );
        assert!((engine.get_type_score("TODO") - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_learning_engine_insights() {
        let mut engine = LearningEngine::new();
        for i in 0..5 {
            engine.record_feedback(
                &format!("s{}", i),
                FeedbackRating::Helpful,
                "TODO",
                None,
                None,
            );
        }
        let insights = engine.generate_insights();
        assert!(!insights.is_empty());
        assert_eq!(insights[0].insight_type, InsightType::SuggestionPreference);
    }

    #[test]
    fn test_store_and_query_context() {
        let mut engine = LearningEngine::new();
        engine.store_context("language", "rust", ContextType::Language);
        engine.store_context("language", "typescript", ContextType::Language);
        assert_eq!(engine.memory.len(), 2);
        let lang_entries = engine.memory.query_by_type(&ContextType::Language);
        assert_eq!(lang_entries.len(), 2);
    }
}
