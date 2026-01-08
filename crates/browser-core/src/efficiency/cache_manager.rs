//! # Optimized Cache Manager

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use super::EfficiencyMetrics;

#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    created_at: Instant,
    last_accessed: Instant,
}

impl<V> CacheEntry<V> {
    fn new(value: V) -> Self {
        let now = Instant::now();
        Self { value, created_at: now, last_accessed: now }
    }
    fn is_expired(&self, ttl: Duration) -> bool { self.created_at.elapsed() > ttl }
    fn touch(&mut self) { self.last_accessed = Instant::now(); }
}

#[derive(Debug)]
pub struct OptimizedCacheManager {
    inner: Arc<RwLock<HashMap<String, CacheEntry<Vec<u8>>>>>,
    ttl: Duration,
    metrics: Arc<EfficiencyMetrics>,
}

impl OptimizedCacheManager {
    pub fn new(ttl_seconds: u64, metrics: Arc<EfficiencyMetrics>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_seconds),
            metrics,
        }
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let mut inner = self.inner.write().ok()?;
        if let Some(entry) = inner.get_mut(key) {
            if entry.is_expired(self.ttl) {
                inner.remove(key);
                self.metrics.record_cache_miss();
                return None;
            }
            entry.touch();
            self.metrics.record_cache_hit();
            return Some(entry.value.clone());
        }
        self.metrics.record_cache_miss();
        None
    }

    pub fn insert(&self, key: String, value: Vec<u8>) {
        if let Ok(mut inner) = self.inner.write() {
            inner.insert(key, CacheEntry::new(value));
        }
    }

    pub fn remove(&self, key: &str) -> Option<Vec<u8>> {
        self.inner.write().ok()?.remove(key).map(|e| e.value)
    }

    pub fn clear(&self) { if let Ok(mut inner) = self.inner.write() { inner.clear(); } }
    pub fn len(&self) -> usize { self.inner.read().map(|i| i.len()).unwrap_or(0) }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
}

impl Clone for OptimizedCacheManager {
    fn clone(&self) -> Self {
        Self { inner: Arc::clone(&self.inner), ttl: self.ttl, metrics: Arc::clone(&self.metrics) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache() {
        let metrics = Arc::new(EfficiencyMetrics::default());
        let cache = OptimizedCacheManager::new(300, metrics);
        cache.insert("key1".to_string(), b"value1".to_vec());
        assert_eq!(cache.get("key1"), Some(b"value1".to_vec()));
    }
}
