use serde::{Deserialize, Serialize};
use super::log_parser::{LogEntry, LogLevel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugAnalysis {
    pub errors: Vec<LogEntry>,
    pub warnings: Vec<LogEntry>,
    pub patterns: Vec<ErrorPattern>,
    pub suggestions: Vec<DebugSuggestion>,
    pub root_cause: Option<String>,
    pub severity: AnalysisSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AnalysisSeverity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    pub pattern_type: String,
    pub description: String,
    pub occurrences: usize,
    pub first_seen: Option<String>,
    pub last_seen: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugSuggestion {
    pub id: String,
    pub title: String,
    pub description: String,
    pub confidence: f64,
    pub action_type: SuggestionAction,
    pub related_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SuggestionAction {
    CheckConfig,
    AddRetry,
    IncreaseTimeout,
    FixConnection,
    AddLogging,
    FixNullRef,
    FixPermission,
    GeneralFix,
}

pub struct DebugAnalyzer;

impl DebugAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self, logs: &[LogEntry]) -> DebugAnalysis {
        let errors: Vec<LogEntry> = logs.iter().filter(|e| e.level == LogLevel::Error).cloned().collect();
        let warnings: Vec<LogEntry> = logs.iter().filter(|e| e.level == LogLevel::Warning).cloned().collect();
        let patterns = self.detect_patterns(logs);
        let suggestions = self.generate_suggestions(&errors, &warnings, &patterns);
        let root_cause = self.estimate_root_cause(&errors, &patterns);
        let severity = self.determine_severity(&errors, &warnings);

        DebugAnalysis {
            errors,
            warnings,
            patterns,
            suggestions,
            root_cause,
            severity,
        }
    }

    fn detect_patterns(&self, logs: &[LogEntry]) -> Vec<ErrorPattern> {
        let mut patterns = Vec::new();
        let mut error_msgs: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for log in logs {
            if log.level == LogLevel::Error {
                let simplified = self.simplify_message(&log.message);
                *error_msgs.entry(simplified).or_insert(0) += 1;
            }
        }

        for (msg, count) in error_msgs {
            if count >= 2 {
                patterns.push(ErrorPattern {
                    pattern_type: "repeated_error".to_string(),
                    description: format!("Error '{}' repeated {} times", msg, count),
                    occurrences: count,
                    first_seen: None,
                    last_seen: None,
                });
            }
        }

        let has_connection = logs.iter().any(|l| l.message.to_lowercase().contains("connection"));
        if has_connection {
            patterns.push(ErrorPattern {
                pattern_type: "connection_issue".to_string(),
                description: "Connection-related errors detected".to_string(),
                occurrences: logs.iter().filter(|l| l.message.to_lowercase().contains("connection")).count(),
                first_seen: None,
                last_seen: None,
            });
        }

        let has_timeout = logs.iter().any(|l| l.message.to_lowercase().contains("timeout"));
        if has_timeout {
            patterns.push(ErrorPattern {
                pattern_type: "timeout_issue".to_string(),
                description: "Timeout-related errors detected".to_string(),
                occurrences: logs.iter().filter(|l| l.message.to_lowercase().contains("timeout")).count(),
                first_seen: None,
                last_seen: None,
            });
        }

        patterns
    }

    fn generate_suggestions(
        &self,
        errors: &[LogEntry],
        _warnings: &[LogEntry],
        patterns: &[ErrorPattern],
    ) -> Vec<DebugSuggestion> {
        let mut suggestions = Vec::new();

        for pattern in patterns {
            match pattern.pattern_type.as_str() {
                "connection_issue" => {
                    suggestions.push(DebugSuggestion {
                        id: uuid::Uuid::new_v4().to_string(),
                        title: "Check network configuration".to_string(),
                        description: "Multiple connection errors suggest network or DNS issues. Verify endpoints and retry configuration.".to_string(),
                        confidence: 0.8,
                        action_type: SuggestionAction::CheckConfig,
                        related_errors: errors.iter().map(|e| e.message.clone()).collect(),
                    });
                    suggestions.push(DebugSuggestion {
                        id: uuid::Uuid::new_v4().to_string(),
                        title: "Add retry with backoff".to_string(),
                        description: "Implement exponential backoff retry for transient connection failures.".to_string(),
                        confidence: 0.75,
                        action_type: SuggestionAction::AddRetry,
                        related_errors: vec![],
                    });
                }
                "timeout_issue" => {
                    suggestions.push(DebugSuggestion {
                        id: uuid::Uuid::new_v4().to_string(),
                        title: "Increase timeout values".to_string(),
                        description: "Current timeout may be too short for the workload. Consider increasing the timeout or optimizing the operation.".to_string(),
                        confidence: 0.7,
                        action_type: SuggestionAction::IncreaseTimeout,
                        related_errors: vec![],
                    });
                }
                "repeated_error" => {
                    suggestions.push(DebugSuggestion {
                        id: uuid::Uuid::new_v4().to_string(),
                        title: "Investigate repeated error".to_string(),
                        description: format!("Same error occurring {} times. This indicates a systematic issue that needs to be addressed at the root cause.", pattern.occurrences),
                        confidence: 0.85,
                        action_type: SuggestionAction::GeneralFix,
                        related_errors: vec![pattern.description.clone()],
                    });
                }
                _ => {}
            }
        }

        for error in errors.iter().take(5) {
            let msg_lower = error.message.to_lowercase();
            if msg_lower.contains("null") || msg_lower.contains("none") || msg_lower.contains("undefined") {
                suggestions.push(DebugSuggestion {
                    id: uuid::Uuid::new_v4().to_string(),
                    title: "Null reference detected".to_string(),
                    description: "Error mentions null/none/undefined. Add null checks or validate inputs before use.".to_string(),
                    confidence: 0.6,
                    action_type: SuggestionAction::FixNullRef,
                    related_errors: vec![error.message.clone()],
                });
            }
            if msg_lower.contains("permission") || msg_lower.contains("denied") || msg_lower.contains("forbidden") {
                suggestions.push(DebugSuggestion {
                    id: uuid::Uuid::new_v4().to_string(),
                    title: "Permission issue".to_string(),
                    description: "Access denied. Check file permissions, API keys, or user roles.".to_string(),
                    confidence: 0.9,
                    action_type: SuggestionAction::FixPermission,
                    related_errors: vec![error.message.clone()],
                });
            }
        }

        if suggestions.is_empty() && !errors.is_empty() {
            suggestions.push(DebugSuggestion {
                id: uuid::Uuid::new_v4().to_string(),
                title: "Review error context".to_string(),
                description: "No specific pattern detected. Review the error messages and their context manually.".to_string(),
                confidence: 0.3,
                action_type: SuggestionAction::AddLogging,
                related_errors: errors.iter().take(3).map(|e| e.message.clone()).collect(),
            });
        }

        suggestions
    }

    fn estimate_root_cause(&self, errors: &[LogEntry], patterns: &[ErrorPattern]) -> Option<String> {
        if errors.is_empty() {
            return None;
        }

        if let Some(conn_pattern) = patterns.iter().find(|p| p.pattern_type == "connection_issue") {
            return Some(format!("Root cause likely: {}", conn_pattern.description));
        }

        if let Some(timeout_pattern) = patterns.iter().find(|p| p.pattern_type == "timeout_issue") {
            return Some(format!("Root cause likely: {}", timeout_pattern.description));
        }

        if let Some(repeated) = patterns.iter().find(|p| p.pattern_type == "repeated_error") {
            return Some(format!("Systematic issue: {}", repeated.description));
        }

        Some(format!("First error may indicate root cause: {}", errors[0].message))
    }

    fn determine_severity(&self, errors: &[LogEntry], warnings: &[LogEntry]) -> AnalysisSeverity {
        if errors.len() > 10 {
            AnalysisSeverity::Critical
        } else if errors.len() > 5 {
            AnalysisSeverity::High
        } else if !errors.is_empty() {
            AnalysisSeverity::Medium
        } else if !warnings.is_empty() {
            AnalysisSeverity::Low
        } else {
            AnalysisSeverity::Informational
        }
    }

    fn simplify_message(&self, msg: &str) -> String {
        let mut result = String::new();
        for word in msg.split_whitespace() {
            if !result.is_empty() {
                result.push(' ');
            }
            if word.len() > 20 && word.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
                result.push_str("<ID>");
            } else if word.chars().all(|c| c.is_ascii_digit()) {
                result.push_str("<N>");
            } else {
                result.push_str(word);
            }
            if result.len() > 100 {
                break;
            }
        }
        result
    }
}

impl Default for DebugAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::log_parser::LogParser;

    #[test]
    fn test_analyze_with_errors() {
        let parser = LogParser::new();
        let logs = parser.parse_logs("[ERROR] db: connection refused\n[ERROR] db: connection refused\n[WARN] app: retry");
        let analyzer = DebugAnalyzer::new();
        let analysis = analyzer.analyze(&logs);
        assert_eq!(analysis.errors.len(), 2);
        assert_eq!(analysis.warnings.len(), 1);
        assert!(!analysis.suggestions.is_empty());
        assert!(analysis.root_cause.is_some());
    }

    #[test]
    fn test_severity_levels() {
        let parser = LogParser::new();
        let analyzer = DebugAnalyzer::new();

        let no_errors = parser.parse_logs("[INFO] app: ok");
        assert_eq!(analyzer.analyze(&no_errors).severity, AnalysisSeverity::Informational);

        let one_warn = parser.parse_logs("[WARN] app: something");
        assert_eq!(analyzer.analyze(&one_warn).severity, AnalysisSeverity::Low);

        let one_error = parser.parse_logs("[ERROR] app: failed");
        assert_eq!(analyzer.analyze(&one_error).severity, AnalysisSeverity::Medium);
    }

    #[test]
    fn test_connection_pattern_detection() {
        let parser = LogParser::new();
        let logs = parser.parse_logs("[ERROR] net: connection refused\n[ERROR] net: connection timeout");
        let analyzer = DebugAnalyzer::new();
        let analysis = analyzer.analyze(&logs);
        let has_conn = analysis.patterns.iter().any(|p| p.pattern_type == "connection_issue");
        assert!(has_conn);
    }

    #[test]
    fn test_permission_detection() {
        let parser = LogParser::new();
        let logs = parser.parse_logs("[ERROR] fs: permission denied");
        let analyzer = DebugAnalyzer::new();
        let analysis = analyzer.analyze(&logs);
        let has_perm = analysis.suggestions.iter().any(|s| s.action_type == SuggestionAction::FixPermission);
        assert!(has_perm);
    }
}
