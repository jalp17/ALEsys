use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_connections: usize,
    pub min_connections: usize,
    pub idle_timeout_secs: u64,
    pub max_lifetime_secs: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 25,
            min_connections: 5,
            idle_timeout_secs: 300,
            max_lifetime_secs: 1800,
        }
    }
}

pub struct ConnectionPool {
    active: AtomicUsize,
    idle: AtomicUsize,
    total: AtomicUsize,
    config: PoolConfig,
    _connections: Mutex<Vec<()>>,
}

impl ConnectionPool {
    pub fn new(config: PoolConfig) -> Self {
        let min = config.min_connections;
        Self {
            active: AtomicUsize::new(0),
            idle: AtomicUsize::new(min),
            total: AtomicUsize::new(min),
            config,
            _connections: Mutex::new((0..min).map(|_| ()).collect()),
        }
    }

    pub fn acquire(&self) -> bool {
        let current_active = self.active.load(Ordering::Relaxed);
        if current_active >= self.config.max_connections {
            return false;
        }

        let idle = self.idle.load(Ordering::Relaxed);
        if idle > 0 {
            self.idle.fetch_sub(1, Ordering::Relaxed);
        } else {
            let total = self.total.load(Ordering::Relaxed);
            if total < self.config.max_connections {
                self.total.fetch_add(1, Ordering::Relaxed);
            } else {
                return false;
            }
        }

        self.active.fetch_add(1, Ordering::Relaxed);
        true
    }

    pub fn release(&self) {
        let active = self.active.load(Ordering::Relaxed);
        if active > 0 {
            self.active.fetch_sub(1, Ordering::Relaxed);
            self.idle.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn stats(&self) -> PoolStats {
        PoolStats {
            active: self.active.load(Ordering::Relaxed),
            idle: self.idle.load(Ordering::Relaxed),
            total: self.total.load(Ordering::Relaxed),
            max: self.config.max_connections,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub active: usize,
    pub idle: usize,
    pub total: usize,
    pub max: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_acquire_release() {
        let pool = ConnectionPool::new(PoolConfig::default());
        assert!(pool.acquire());
        let stats = pool.stats();
        assert_eq!(stats.active, 1);
        pool.release();
        let stats = pool.stats();
        assert_eq!(stats.active, 0);
    }

    #[test]
    fn test_pool_max_connections() {
        let config = PoolConfig { max_connections: 2, ..Default::default() };
        let pool = ConnectionPool::new(config);
        assert!(pool.acquire());
        assert!(pool.acquire());
        assert!(!pool.acquire());
    }

    #[test]
    fn test_pool_stats() {
        let pool = ConnectionPool::new(PoolConfig::default());
        let stats = pool.stats();
        assert_eq!(stats.total, 5);
        assert_eq!(stats.max, 25);
    }
}