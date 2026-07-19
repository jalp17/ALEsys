use serde::{Deserialize, Serialize};
use std::time::Instant;

pub struct Profiler {
    start: Instant,
    checkpoints: Vec<(String, u64)>,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            checkpoints: vec![],
        }
    }

    pub fn checkpoint(&mut self, name: &str) {
        let elapsed = self.start.elapsed().as_millis() as u64;
        self.checkpoints.push((name.to_string(), elapsed));
    }

    pub fn report(&self) -> ProfileReport {
        let mut sections = Vec::new();
        let mut prev_time = 0;

        for (name, time) in &self.checkpoints {
            sections.push(ProfileSection {
                name: name.clone(),
                duration_ms: time - prev_time,
                cumulative_ms: *time,
            });
            prev_time = *time;
        }

        let total = self.start.elapsed().as_millis() as u64;

        ProfileReport {
            total_duration_ms: total,
            sections,
            memory_estimated_bytes: 0,
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileReport {
    pub total_duration_ms: u64,
    pub sections: Vec<ProfileSection>,
    pub memory_estimated_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSection {
    pub name: String,
    pub duration_ms: u64,
    pub cumulative_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_profiler_basic() {
        let mut profiler = Profiler::new();
        thread::sleep(Duration::from_millis(10));
        profiler.checkpoint("step1");
        thread::sleep(Duration::from_millis(10));
        profiler.checkpoint("step2");
        let report = profiler.report();
        assert_eq!(report.sections.len(), 2);
        assert!(report.total_duration_ms >= 10);
    }

    #[test]
    fn test_profiler_elapsed() {
        let profiler = Profiler::new();
        thread::sleep(Duration::from_millis(5));
        assert!(profiler.elapsed_ms() >= 5);
    }

    #[test]
    fn test_empty_profiler() {
        let profiler = Profiler::new();
        let report = profiler.report();
        assert!(report.sections.is_empty());
    }
}