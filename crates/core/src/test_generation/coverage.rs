use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub file_path: String,
    pub lines_total: usize,
    pub lines_covered: usize,
    pub branches_total: usize,
    pub branches_covered: usize,
    pub functions_total: usize,
    pub functions_covered: usize,
    pub coverage_percentage: f64,
    pub uncovered_lines: Vec<usize>,
    pub uncovered_functions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCoverage {
    pub reports: Vec<CoverageReport>,
    pub overall_percentage: f64,
    pub total_lines: usize,
    pub covered_lines: usize,
    pub files_with_low_coverage: Vec<String>,
}

pub struct CoverageTracker {
    reports: HashMap<String, CoverageReport>,
}

impl CoverageTracker {
    pub fn new() -> Self {
        Self {
            reports: HashMap::new(),
        }
    }

    pub fn track_file(&mut self, file_path: &str, lines_total: usize, lines_covered: usize) {
        let uncovered_lines: Vec<usize> = (1..=lines_total)
            .filter(|&line| line > lines_covered)
            .collect();

        let coverage_percentage = if lines_total > 0 {
            (lines_covered as f64 / lines_total as f64) * 100.0
        } else {
            0.0
        };

        self.reports.insert(
            file_path.to_string(),
            CoverageReport {
                file_path: file_path.to_string(),
                lines_total,
                lines_covered,
                branches_total: 0,
                branches_covered: 0,
                functions_total: 0,
                functions_covered: 0,
                coverage_percentage,
                uncovered_lines,
                uncovered_functions: vec![],
            },
        );
    }

    pub fn track_file_with_branches(
        &mut self,
        file_path: &str,
        lines_total: usize,
        lines_covered: usize,
        branches_total: usize,
        branches_covered: usize,
    ) {
        let uncovered_lines: Vec<usize> = (1..=lines_total)
            .filter(|&line| line > lines_covered)
            .collect();

        let line_coverage = if lines_total > 0 {
            (lines_covered as f64 / lines_total as f64) * 100.0
        } else {
            0.0
        };

        let branch_coverage = if branches_total > 0 {
            (branches_covered as f64 / branches_total as f64) * 100.0
        } else {
            0.0
        };

        let coverage_percentage = (line_coverage + branch_coverage) / 2.0;

        self.reports.insert(
            file_path.to_string(),
            CoverageReport {
                file_path: file_path.to_string(),
                lines_total,
                lines_covered,
                branches_total,
                branches_covered,
                functions_total: 0,
                functions_covered: 0,
                coverage_percentage,
                uncovered_lines,
                uncovered_functions: vec![],
            },
        );
    }

    pub fn get_report(&self, file_path: &str) -> Option<&CoverageReport> {
        self.reports.get(file_path)
    }

    pub fn get_all_reports(&self) -> Vec<&CoverageReport> {
        self.reports.values().collect()
    }

    pub fn calculate_overall(&self) -> ProjectCoverage {
        let reports: Vec<CoverageReport> = self.reports.values().cloned().collect();
        let total_lines: usize = reports.iter().map(|r| r.lines_total).sum();
        let covered_lines: usize = reports.iter().map(|r| r.lines_covered).sum();

        let overall_percentage = if total_lines > 0 {
            (covered_lines as f64 / total_lines as f64) * 100.0
        } else {
            0.0
        };

        let files_with_low_coverage: Vec<String> = reports
            .iter()
            .filter(|r| r.coverage_percentage < 50.0)
            .map(|r| r.file_path.clone())
            .collect();

        ProjectCoverage {
            reports,
            overall_percentage,
            total_lines,
            covered_lines,
            files_with_low_coverage,
        }
    }

    pub fn get_uncovered_lines(&self, file_path: &str) -> Vec<usize> {
        self.reports
            .get(file_path)
            .map(|r| r.uncovered_lines.clone())
            .unwrap_or_default()
    }

    pub fn get_files_below_threshold(&self, threshold: f64) -> Vec<String> {
        self.reports
            .values()
            .filter(|r| r.coverage_percentage < threshold)
            .map(|r| r.file_path.clone())
            .collect()
    }

    pub fn generate_coverage_badge(&self, file_path: &str) -> String {
        match self.reports.get(file_path) {
            Some(report) => {
                let color = if report.coverage_percentage >= 80.0 {
                    "brightgreen"
                } else if report.coverage_percentage >= 60.0 {
                    "yellow"
                } else if report.coverage_percentage >= 40.0 {
                    "orange"
                } else {
                    "red"
                };

                format!(
                    "{{\"schemaVersion\":1,\"label\":\"coverage\",\"message\":\"{:.1}%\",\"color\":\"{}\"}}",
                    report.coverage_percentage, color
                )
            }
            None => "{{\"schemaVersion\":1,\"label\":\"coverage\",\"message\":\"N/A\",\"color\":\"lightgrey\"}}".to_string(),
        }
    }

    pub fn export_summary(&self) -> String {
        let overall = self.calculate_overall();
        let mut summary = String::new();

        summary.push_str(&format!("=== Coverage Summary ===\n"));
        summary.push_str(&format!("Overall: {:.1}%\n", overall.overall_percentage));
        summary.push_str(&format!("Total Lines: {} | Covered: {}\n", overall.total_lines, overall.covered_lines));
        summary.push_str(&format!("Files with low coverage: {}\n\n", overall.files_with_low_coverage.len()));

        for report in &overall.reports {
            summary.push_str(&format!(
                "{}: {:.1}% ({}/{} lines)\n",
                report.file_path,
                report.coverage_percentage,
                report.lines_covered,
                report.lines_total
            ));
        }

        summary
    }
}

impl Default for CoverageTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_file() {
        let mut tracker = CoverageTracker::new();
        tracker.track_file("main.rs", 100, 80);
        let report = tracker.get_report("main.rs").unwrap();
        assert_eq!(report.lines_total, 100);
        assert_eq!(report.lines_covered, 80);
        assert!((report.coverage_percentage - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_track_with_branches() {
        let mut tracker = CoverageTracker::new();
        tracker.track_file_with_branches("lib.rs", 200, 150, 50, 40);
        let report = tracker.get_report("lib.rs").unwrap();
        assert_eq!(report.branches_total, 50);
        assert_eq!(report.branches_covered, 40);
    }

    #[test]
    fn test_calculate_overall() {
        let mut tracker = CoverageTracker::new();
        tracker.track_file("a.rs", 100, 80);
        tracker.track_file("b.rs", 200, 100);
        let overall = tracker.calculate_overall();
        assert_eq!(overall.total_lines, 300);
        assert_eq!(overall.covered_lines, 180);
        assert!((overall.overall_percentage - 60.0).abs() < 0.01);
    }

    #[test]
    fn test_low_coverage_detection() {
        let mut tracker = CoverageTracker::new();
        tracker.track_file("good.rs", 100, 90);
        tracker.track_file("bad.rs", 100, 30);
        let low = tracker.get_files_below_threshold(50.0);
        assert_eq!(low.len(), 1);
        assert_eq!(low[0], "bad.rs");
    }

    #[test]
    fn test_export_summary() {
        let mut tracker = CoverageTracker::new();
        tracker.track_file("test.rs", 100, 80);
        let summary = tracker.export_summary();
        assert!(summary.contains("Coverage Summary"));
        assert!(summary.contains("80.0%"));
    }

    #[test]
    fn test_coverage_badge() {
        let mut tracker = CoverageTracker::new();
        tracker.track_file("main.rs", 100, 85);
        let badge = tracker.generate_coverage_badge("main.rs");
        assert!(badge.contains("85.0%"));
        assert!(badge.contains("brightgreen"));
    }
}
