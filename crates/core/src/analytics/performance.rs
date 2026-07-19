use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: String,
}

pub struct PerformanceMonitor {
    metrics: Vec<PerformanceMetric>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self { metrics: vec![] }
    }

    pub fn record(&mut self, name: &str, value: f64, unit: &str) -> PerformanceMetric {
        let metric = PerformanceMetric {
            name: name.to_string(),
            value,
            unit: unit.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        self.metrics.push(metric.clone());
        metric
    }

    pub fn get_metrics(&self) -> &[PerformanceMetric] {
        &self.metrics
    }

    pub fn get_latest(&self, name: &str) -> Option<&PerformanceMetric> {
        self.metrics.iter().rev().find(|m| m.name == name)
    }

    pub fn get_average(&self, name: &str) -> f64 {
        let values: Vec<f64> = self.metrics.iter().filter(|m| m.name == name).map(|m| m.value).collect();
        if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<f64>() / values.len() as f64
        }
    }

    pub fn get_max(&self, name: &str) -> f64 {
        self.metrics.iter().filter(|m| m.name == name).map(|m| m.value).fold(0.0_f64, f64::max)
    }

    pub fn get_min(&self, name: &str) -> f64 {
        self.metrics.iter().filter(|m| m.name == name).map(|m| m.value).fold(f64::INFINITY, f64::min)
    }

    pub fn generate_report(&self) -> PerformanceReport {
        let mut metric_names: Vec<String> = self.metrics.iter().map(|m| m.name.clone()).collect();
        metric_names.sort();
        metric_names.dedup();

        let summaries: Vec<MetricSummary> = metric_names.iter().map(|name| {
            MetricSummary {
                name: name.clone(),
                avg: self.get_average(name),
                min: self.get_min(name),
                max: self.get_max(name),
                count: self.metrics.iter().filter(|m| m.name == *name).count(),
            }
        }).collect();

        PerformanceReport {
            total_metrics: self.metrics.len(),
            summaries,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSummary {
    pub name: String,
    pub avg: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub total_metrics: usize,
    pub summaries: Vec<MetricSummary>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_metric() {
        let mut monitor = PerformanceMonitor::new();
        let metric = monitor.record("response_time", 150.0, "ms");
        assert_eq!(metric.name, "response_time");
        assert_eq!(metric.value, 150.0);
    }

    #[test]
    fn test_get_average() {
        let mut monitor = PerformanceMonitor::new();
        monitor.record("latency", 100.0, "ms");
        monitor.record("latency", 200.0, "ms");
        monitor.record("latency", 300.0, "ms");
        assert_eq!(monitor.get_average("latency"), 200.0);
    }

    #[test]
    fn test_get_max_min() {
        let mut monitor = PerformanceMonitor::new();
        monitor.record("cpu", 10.0, "%");
        monitor.record("cpu", 50.0, "%");
        monitor.record("cpu", 30.0, "%");
        assert_eq!(monitor.get_max("cpu"), 50.0);
        assert_eq!(monitor.get_min("cpu"), 10.0);
    }

    #[test]
    fn test_report() {
        let mut monitor = PerformanceMonitor::new();
        monitor.record("rt", 100.0, "ms");
        monitor.record("rt", 150.0, "ms");
        let report = monitor.generate_report();
        assert_eq!(report.total_metrics, 2);
        assert_eq!(report.summaries.len(), 1);
    }

    #[test]
    fn test_empty_monitor() {
        let monitor = PerformanceMonitor::new();
        assert_eq!(monitor.get_average("anything"), 0.0);
    }
}