use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItem<T> {
    pub id: String,
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub processed: usize,
    pub failed: usize,
    pub duration_ms: u64,
    pub errors: Vec<String>,
}

pub struct BatchProcessor {
    batch_size: usize,
}

impl BatchProcessor {
    pub fn new(batch_size: usize) -> Self {
        Self { batch_size }
    }

    pub fn process_batch<F>(&self, items: Vec<BatchItem<String>>, mut processor: F) -> BatchResult
    where
        F: FnMut(&BatchItem<String>) -> Result<(), String>,
    {
        let start = std::time::Instant::now();
        let mut processed = 0;
        let mut failed = 0;
        let mut errors = Vec::new();

        for chunk in items.chunks(self.batch_size) {
            for item in chunk {
                match processor(item) {
                    Ok(()) => processed += 1,
                    Err(e) => {
                        failed += 1;
                        errors.push(format!("{}: {}", item.id, e));
                    }
                }
            }
        }

        BatchResult {
            processed,
            failed,
            duration_ms: start.elapsed().as_millis() as u64,
            errors,
        }
    }

    pub fn calculate_optimal_batch(&self, total_items: usize, target_ms: u64, avg_item_ms: f64) -> usize {
        let max_batch = (target_ms as f64 / avg_item_ms) as usize;
        let optimal = std::cmp::min(max_batch, total_items);
        std::cmp::max(optimal, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_batch() {
        let processor = BatchProcessor::new(5);
        let items: Vec<BatchItem<String>> = (0..10)
            .map(|i| BatchItem { id: format!("item-{}", i), data: format!("data-{}", i) })
            .collect();
        let result = processor.process_batch(items, |item| {
            if item.id.contains("5") {
                Err("test error".to_string())
            } else {
                Ok(())
            }
        });
        assert_eq!(result.processed, 9);
        assert_eq!(result.failed, 1);
    }

    #[test]
    fn test_empty_batch() {
        let processor = BatchProcessor::new(5);
        let result = processor.process_batch(vec![], |_| Ok(()));
        assert_eq!(result.processed, 0);
        assert_eq!(result.failed, 0);
    }

    #[test]
    fn test_optimal_batch() {
        let processor = BatchProcessor::new(100);
        let optimal = processor.calculate_optimal_batch(1000, 100, 10.0);
        assert_eq!(optimal, 10);
    }
}