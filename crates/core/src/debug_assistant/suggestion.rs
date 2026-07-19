use serde::{Deserialize, Serialize};
use super::analyzer::{DebugAnalysis, SuggestionAction};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionReport {
    pub analysis_summary: String,
    pub severity: String,
    pub total_errors: usize,
    pub total_warnings: usize,
    pub patterns_found: usize,
    pub suggestions: Vec<FormattedSuggestion>,
    pub root_cause: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattedSuggestion {
    pub title: String,
    pub description: String,
    pub confidence: f64,
    pub action: String,
    pub priority: String,
}

pub struct SuggestionFormatter;

impl SuggestionFormatter {
    pub fn new() -> Self {
        Self
    }

    pub fn format(&self, analysis: &DebugAnalysis) -> SuggestionReport {
        let severity_str = match analysis.severity {
            super::analyzer::AnalysisSeverity::Critical => "CRITICAL".to_string(),
            super::analyzer::AnalysisSeverity::High => "HIGH".to_string(),
            super::analyzer::AnalysisSeverity::Medium => "MEDIUM".to_string(),
            super::analyzer::AnalysisSeverity::Low => "LOW".to_string(),
            super::analyzer::AnalysisSeverity::Informational => "INFO".to_string(),
        };

        let suggestions: Vec<FormattedSuggestion> = analysis.suggestions.iter().map(|s| {
            let priority = if s.confidence >= 0.8 {
                "High"
            } else if s.confidence >= 0.5 {
                "Medium"
            } else {
                "Low"
            };

            FormattedSuggestion {
                title: s.title.clone(),
                description: s.description.clone(),
                confidence: s.confidence,
                action: Self::action_to_string(&s.action_type),
                priority: priority.to_string(),
            }
        }).collect();

        let summary = format!(
            "Found {} errors, {} warnings, {} patterns with {} actionable suggestions. Severity: {}",
            analysis.errors.len(),
            analysis.warnings.len(),
            analysis.patterns.len(),
            suggestions.len(),
            severity_str,
        );

        SuggestionReport {
            analysis_summary: summary,
            severity: severity_str,
            total_errors: analysis.errors.len(),
            total_warnings: analysis.warnings.len(),
            patterns_found: analysis.patterns.len(),
            suggestions,
            root_cause: analysis.root_cause.clone(),
        }
    }

    fn action_to_string(action: &SuggestionAction) -> String {
        match action {
            SuggestionAction::CheckConfig => "Check configuration".to_string(),
            SuggestionAction::AddRetry => "Add retry logic".to_string(),
            SuggestionAction::IncreaseTimeout => "Increase timeout".to_string(),
            SuggestionAction::FixConnection => "Fix connection".to_string(),
            SuggestionAction::AddLogging => "Add logging".to_string(),
            SuggestionAction::FixNullRef => "Fix null reference".to_string(),
            SuggestionAction::FixPermission => "Fix permissions".to_string(),
            SuggestionAction::GeneralFix => "General fix needed".to_string(),
        }
    }
}

impl Default for SuggestionFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::analyzer::{DebugAnalyzer, AnalysisSeverity};
    use super::super::log_parser::LogParser;

    #[test]
    fn test_format_analysis() {
        let parser = LogParser::new();
        let logs = parser.parse_logs("[ERROR] db: connection refused\n[ERROR] db: connection refused\n[ERROR] net: timeout");
        let analyzer = DebugAnalyzer::new();
        let analysis = analyzer.analyze(&logs);
        let formatter = SuggestionFormatter::new();
        let report = formatter.format(&analysis);
        assert_eq!(report.total_errors, 3);
        assert!(!report.suggestions.is_empty());
        assert!(report.root_cause.is_some());
    }

    #[test]
    fn test_severity_formatting() {
        let formatter = SuggestionFormatter::new();
        let analysis = DebugAnalysis {
            errors: vec![],
            warnings: vec![],
            patterns: vec![],
            suggestions: vec![],
            root_cause: None,
            severity: AnalysisSeverity::Critical,
        };
        let report = formatter.format(&analysis);
        assert_eq!(report.severity, "CRITICAL");
    }
}
