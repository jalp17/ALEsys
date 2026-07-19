use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportType {
    Usage,
    Performance,
    Behavior,
    Summary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: String,
    pub title: String,
    pub report_type: ReportType,
    pub data: serde_json::Value,
    pub generated_at: String,
}

pub struct ReportGenerator {
    reports: Vec<Report>,
}

impl ReportGenerator {
    pub fn new() -> Self {
        Self { reports: vec![] }
    }

    pub fn generate_usage_report(&mut self, stats: &super::usage_tracker::UsageStats) -> Report {
        let report = Report {
            id: format!("report-{}", self.reports.len()),
            title: "Usage Report".to_string(),
            report_type: ReportType::Usage,
            data: serde_json::json!({
                "total_events": stats.total_events,
                "unique_users": stats.unique_users,
                "events_by_type": stats.events_by_type,
            }),
            generated_at: chrono::Utc::now().to_rfc3339(),
        };
        self.reports.push(report.clone());
        report
    }

    pub fn generate_performance_report(&mut self, perf_report: &super::performance::PerformanceReport) -> Report {
        let report = Report {
            id: format!("report-{}", self.reports.len()),
            title: "Performance Report".to_string(),
            report_type: ReportType::Performance,
            data: serde_json::json!({
                "total_metrics": perf_report.total_metrics,
                "summaries": perf_report.summaries.iter().map(|s| serde_json::json!({
                    "name": s.name,
                    "avg": s.avg,
                    "min": s.min,
                    "max": s.max,
                })).collect::<Vec<_>>(),
            }),
            generated_at: chrono::Utc::now().to_rfc3339(),
        };
        self.reports.push(report.clone());
        report
    }

    pub fn generate_summary(&mut self, title: &str, data: serde_json::Value) -> Report {
        let report = Report {
            id: format!("report-{}", self.reports.len()),
            title: title.to_string(),
            report_type: ReportType::Summary,
            data,
            generated_at: chrono::Utc::now().to_rfc3339(),
        };
        self.reports.push(report.clone());
        report
    }

    pub fn get_reports(&self) -> &[Report] {
        &self.reports
    }

    pub fn get_report(&self, id: &str) -> Option<&Report> {
        self.reports.iter().find(|r| r.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::usage_tracker::UsageStats;

    #[test]
    fn test_generate_usage_report() {
        let mut gen = ReportGenerator::new();
        let stats = UsageStats {
            total_events: 100,
            unique_users: 10,
            events_by_type: std::collections::HashMap::new(),
            events_by_user: std::collections::HashMap::new(),
        };
        let report = gen.generate_usage_report(&stats);
        assert_eq!(report.title, "Usage Report");
        assert_eq!(report.data["total_events"], 100);
    }

    #[test]
    fn test_generate_summary() {
        let mut gen = ReportGenerator::new();
        let report = gen.generate_summary("Custom Report", serde_json::json!({"key": "value"}));
        assert_eq!(report.title, "Custom Report");
    }

    #[test]
    fn test_get_report() {
        let mut gen = ReportGenerator::new();
        gen.generate_summary("Test", serde_json::json!({}));
        let reports = gen.get_reports();
        let first = gen.get_report(&reports[0].id);
        assert!(first.is_some());
    }

    #[test]
    fn test_empty_generator() {
        let gen = ReportGenerator::new();
        assert!(gen.get_reports().is_empty());
    }

    #[test]
    fn test_multiple_reports() {
        let mut gen = ReportGenerator::new();
        gen.generate_summary("Report 1", serde_json::json!({}));
        gen.generate_summary("Report 2", serde_json::json!({}));
        gen.generate_summary("Report 3", serde_json::json!({}));
        assert_eq!(gen.get_reports().len(), 3);
    }
}