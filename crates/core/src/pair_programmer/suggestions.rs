//! Suggestion Engine - Identifies improvements proactively

use serde::{Deserialize, Serialize};

/// Type of suggestion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SuggestionType {
    CodeSmell,
    MissingTest,
    DuplicateCode,
    RefactorOpportunity,
    PerformanceImprovement,
    SecurityIssue,
}

/// A code suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub id: String,
    pub suggestion_type: SuggestionType,
    pub file_path: String,
    pub line: usize,
    pub description: String,
    pub severity: Severity,
    pub auto_fixable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Generates suggestions for code improvement
pub struct SuggestionEngine;

impl SuggestionEngine {
    pub fn new() -> Self {
        Self
    }

    /// Analyze code and generate suggestions
    pub fn analyze(&self, code: &str, file_path: &str) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        for (line_num, line) in code.lines().enumerate() {
            // Check for TODO/FIXME
            if line.contains("TODO") || line.contains("FIXME") {
                suggestions.push(Suggestion {
                    id: format!("todo-{}", line_num),
                    suggestion_type: SuggestionType::CodeSmell,
                    file_path: file_path.to_string(),
                    line: line_num + 1,
                    description: "Contains TODO/FIXME marker".to_string(),
                    severity: Severity::Low,
                    auto_fixable: false,
                });
            }

            // Check for unwrap() in production code
            if line.contains(".unwrap()") && !line.contains("#[cfg(test)]") {
                suggestions.push(Suggestion {
                    id: format!("unwrap-{}", line_num),
                    suggestion_type: SuggestionType::CodeSmell,
                    file_path: file_path.to_string(),
                    line: line_num + 1,
                    description: "Uses unwrap() which can panic".to_string(),
                    severity: Severity::Medium,
                    auto_fixable: true,
                });
            }

            // Check for long functions (> 50 lines)
            if line.len() > 200 {
                suggestions.push(Suggestion {
                    id: format!("long-line-{}", line_num),
                    suggestion_type: SuggestionType::CodeSmell,
                    file_path: file_path.to_string(),
                    line: line_num + 1,
                    description: "Line exceeds 200 characters".to_string(),
                    severity: Severity::Low,
                    auto_fixable: false,
                });
            }
        }

        suggestions
    }

    /// Get suggestion count by type
    pub fn count_by_type(suggestions: &[Suggestion]) -> std::collections::HashMap<SuggestionType, usize> {
        let mut counts = std::collections::HashMap::new();
        for s in suggestions {
            *counts.entry(s.suggestion_type.clone()).or_insert(0) += 1;
        }
        counts
    }
}

impl Default for SuggestionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_todo() {
        let engine = SuggestionEngine::new();
        let suggestions = engine.analyze("// TODO: implement this", "test.rs");
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].suggestion_type, SuggestionType::CodeSmell);
    }

    #[test]
    fn test_detect_unwrap() {
        let engine = SuggestionEngine::new();
        let suggestions = engine.analyze("let x = foo.unwrap();", "test.rs");
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].auto_fixable);
    }

    #[test]
    fn test_no_suggestions() {
        let engine = SuggestionEngine::new();
        let suggestions = engine.analyze("let x = 42;", "test.rs");
        assert_eq!(suggestions.len(), 0);
    }
}
