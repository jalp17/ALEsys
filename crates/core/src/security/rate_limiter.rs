use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests: usize,
    pub window_secs: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_secs: 60,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RateLimitResult {
    Allowed,
    Limited { retry_after_secs: u64 },
}

pub struct RateLimiter {
    config: RateLimitConfig,
    requests: HashMap<String, Vec<Instant>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            requests: HashMap::new(),
        }
    }

    pub fn check(&mut self, key: &str) -> RateLimitResult {
        let now = Instant::now();
        let window = Duration::from_secs(self.config.window_secs);

        let timestamps = self.requests.entry(key.to_string()).or_insert_with(Vec::new);
        timestamps.retain(|t| now.duration_since(*t) < window);

        if timestamps.len() >= self.config.max_requests {
            let oldest = timestamps[0];
            let retry_after = window.as_secs() - oldest.elapsed().as_secs();
            RateLimitResult::Limited { retry_after_secs: retry_after }
        } else {
            timestamps.push(now);
            RateLimitResult::Allowed
        }
    }

    pub fn reset(&mut self, key: &str) {
        self.requests.remove(key);
    }

    pub fn clear(&mut self) {
        self.requests.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_allowed() {
        let mut limiter = RateLimiter::new(RateLimitConfig { max_requests: 5, window_secs: 60 });
        assert_eq!(limiter.check("user1"), RateLimitResult::Allowed);
    }

    #[test]
    fn test_rate_limit_exceeded() {
        let mut limiter = RateLimiter::new(RateLimitConfig { max_requests: 2, window_secs: 60 });
        limiter.check("user1");
        limiter.check("user1");
        assert!(matches!(limiter.check("user1"), RateLimitResult::Limited { .. }));
    }

    #[test]
    fn test_rate_limit_separate_keys() {
        let mut limiter = RateLimiter::new(RateLimitConfig { max_requests: 1, window_secs: 60 });
        limiter.check("user1");
        assert_eq!(limiter.check("user2"), RateLimitResult::Allowed);
    }

    #[test]
    fn test_rate_limit_reset() {
        let mut limiter = RateLimiter::new(RateLimitConfig { max_requests: 1, window_secs: 60 });
        limiter.check("user1");
        limiter.reset("user1");
        assert_eq!(limiter.check("user1"), RateLimitResult::Allowed);
    }
}