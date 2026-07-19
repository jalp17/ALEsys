use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CacheEntry<V> {
    pub value: V,
    pub inserted_at: Instant,
    pub ttl: Duration,
    pub access_count: usize,
}

impl<V> CacheEntry<V> {
    pub fn is_expired(&self) -> bool {
        self.inserted_at.elapsed() > self.ttl
    }
}

pub struct Cache<K, V> {
    entries: HashMap<K, CacheEntry<V>>,
    default_ttl: Duration,
    max_size: usize,
    hits: usize,
    misses: usize,
}

impl<K: Eq + std::hash::Hash + Clone, V: Clone> Cache<K, V> {
    pub fn new(default_ttl: Duration, max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            default_ttl,
            max_size,
            hits: 0,
            misses: 0,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.entries.get(key) {
            if entry.is_expired() {
                self.entries.remove(key);
                self.misses += 1;
                return None;
            }
            self.hits += 1;
            let mut entry = self.entries.get_mut(key).unwrap();
            entry.access_count += 1;
            Some(entry.value.clone())
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        if self.entries.len() >= self.max_size {
            self.evict_lru();
        }

        self.entries.insert(key, CacheEntry {
            value,
            inserted_at: Instant::now(),
            ttl: self.default_ttl,
            access_count: 0,
        });
    }

    pub fn insert_with_ttl(&mut self, key: K, value: V, ttl: Duration) {
        if self.entries.len() >= self.max_size {
            self.evict_lru();
        }

        self.entries.insert(key, CacheEntry {
            value,
            inserted_at: Instant::now(),
            ttl,
            access_count: 0,
        });
    }

    pub fn remove(&mut self, key: &K) -> bool {
        self.entries.remove(key).is_some()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.hits = 0;
        self.misses = 0;
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn evict_lru(&mut self) {
        if let Some(lru_key) = self.entries.iter()
            .min_by_key(|(_, e)| e.access_count)
            .map(|(k, _)| k.clone())
        {
            self.entries.remove(&lru_key);
        }
    }

    pub fn cleanup_expired(&mut self) {
        self.entries.retain(|_, entry| !entry.is_expired());
    }

    pub fn stats(&self) -> CacheStats {
        let total = self.hits + self.misses;
        let hit_rate = if total == 0 { 0.0 } else { self.hits as f64 / total as f64 };

        CacheStats {
            size: self.entries.len(),
            max_size: self.max_size,
            hits: self.hits,
            misses: self.misses,
            hit_rate,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub max_size: usize,
    pub hits: usize,
    pub misses: usize,
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut cache = Cache::new(Duration::from_secs(60), 100);
        cache.insert("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = Cache::<String, String>::new(Duration::from_secs(60), 100);
        assert_eq!(cache.get(&"missing".to_string()), None);
    }

    #[test]
    fn test_eviction() {
        let mut cache = Cache::new(Duration::from_secs(60), 2);
        cache.insert("a".to_string(), 1);
        cache.insert("b".to_string(), 2);
        cache.insert("c".to_string(), 3);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_stats() {
        let mut cache = Cache::<String, i32>::new(Duration::from_secs(60), 100);
        cache.get(&"a".to_string());
        cache.insert("b".to_string(), 1);
        cache.get(&"b".to_string());
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_clear() {
        let mut cache = Cache::new(Duration::from_secs(60), 100);
        cache.insert("a".to_string(), 1);
        cache.clear();
        assert!(cache.is_empty());
    }
}