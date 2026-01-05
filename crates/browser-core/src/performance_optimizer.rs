//! Performance Optimizer Module - v5.0 Performance Optimization
//!
//! Part of the V1000 Upgrade Deep Plan - Phase 1 Foundation
//! Provides lazy loading, predictive caching, and performance monitoring.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Performance optimizer for browser operations
pub struct PerformanceOptimizer {
    cache: Arc<RwLock<PredictiveCache>>,
    metrics: Arc<RwLock<PerformanceMetrics>>,
    config: PerformanceConfig,
    start_time: Instant,
    resource_queue: Arc<RwLock<ResourcePriorityQueue>>,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable lazy loading
    pub lazy_loading_enabled: bool,
    /// Enable predictive caching
    pub predictive_caching_enabled: bool,
    /// Maximum cache size in MB
    pub max_cache_size_mb: u64,
    /// Enable resource prefetching
    pub prefetch_enabled: bool,
    /// Number of resources to prefetch
    pub prefetch_limit: usize,
    /// Enable compression
    pub compression_enabled: bool,
    /// Target startup time in ms
    pub target_startup_ms: u64,
    /// Enable parallel loading
    pub parallel_loading_enabled: bool,
    /// Maximum parallel connections
    pub max_parallel_connections: u32,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            lazy_loading_enabled: true,
            predictive_caching_enabled: true,
            max_cache_size_mb: 256,
            prefetch_enabled: true,
            prefetch_limit: 10,
            compression_enabled: true,
            target_startup_ms: 1000,
            parallel_loading_enabled: true,
            max_parallel_connections: 6,
        }
    }
}

/// Predictive cache for resources
#[derive(Debug)]
pub struct PredictiveCache {
    entries: HashMap<String, CacheEntry>,
    access_history: VecDeque<String>,
    prediction_model: AccessPredictionModel,
    total_size_bytes: u64,
    max_size_bytes: u64,
    hits: u64,
    misses: u64,
}

/// Cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub url: String,
    pub content_type: String,
    pub size_bytes: u64,
    pub last_accessed: u128,
    pub access_count: u32,
    pub priority: CachePriority,
    pub expires: Option<u128>,
    pub compressed: bool,
}

/// Cache priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CachePriority {
    Critical,   // Core resources (HTML, critical CSS)
    High,       // Important resources (JS, fonts)
    Medium,     // Regular resources (images)
    Low,        // Optional resources (analytics, ads)
    Prefetch,   // Prefetched resources
}

impl Default for CachePriority {
    fn default() -> Self {
        CachePriority::Medium
    }
}

/// Access prediction model using simple frequency analysis
#[derive(Debug, Default)]
pub struct AccessPredictionModel {
    frequency_map: HashMap<String, u32>,
    transition_map: HashMap<String, HashMap<String, u32>>,
    last_url: Option<String>,
}

impl AccessPredictionModel {
    /// Record a URL access
    pub fn record_access(&mut self, url: &str) {
        *self.frequency_map.entry(url.to_string()).or_insert(0) += 1;
        
        // Record transition
        if let Some(ref last) = self.last_url {
            let transitions = self.transition_map.entry(last.clone()).or_insert_with(HashMap::new);
            *transitions.entry(url.to_string()).or_insert(0) += 1;
        }
        
        self.last_url = Some(url.to_string());
    }

    /// Predict next URLs based on current URL
    pub fn predict_next(&self, current_url: &str, limit: usize) -> Vec<String> {
        let mut predictions = Vec::new();
        
        // Get transition-based predictions
        if let Some(transitions) = self.transition_map.get(current_url) {
            let mut sorted: Vec<_> = transitions.iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(a.1));
            
            for (url, _) in sorted.iter().take(limit) {
                predictions.push(url.to_string());
            }
        }
        
        // Fill with frequency-based predictions
        if predictions.len() < limit {
            let mut sorted: Vec<_> = self.frequency_map.iter()
                .filter(|(url, _)| !predictions.contains(url))
                .collect();
            sorted.sort_by(|a, b| b.1.cmp(a.1));
            
            for (url, _) in sorted.iter().take(limit - predictions.len()) {
                predictions.push(url.to_string());
            }
        }
        
        predictions
    }
}

impl PredictiveCache {
    /// Create a new predictive cache
    pub fn new(max_size_mb: u64) -> Self {
        Self {
            entries: HashMap::new(),
            access_history: VecDeque::with_capacity(1000),
            prediction_model: AccessPredictionModel::default(),
            total_size_bytes: 0,
            max_size_bytes: max_size_mb * 1024 * 1024,
            hits: 0,
            misses: 0,
        }
    }

    /// Get item from cache
    pub fn get(&mut self, url: &str, timestamp: u128) -> Option<&CacheEntry> {
        if let Some(entry) = self.entries.get_mut(url) {
            // Check if expired
            if let Some(expires) = entry.expires {
                if timestamp > expires {
                    self.misses += 1;
                    return None;
                }
            }
            
            entry.last_accessed = timestamp;
            entry.access_count += 1;
            self.hits += 1;
            self.prediction_model.record_access(url);
            self.access_history.push_back(url.to_string());
            
            if self.access_history.len() > 1000 {
                self.access_history.pop_front();
            }
            
            Some(entry)
        } else {
            self.misses += 1;
            self.prediction_model.record_access(url);
            None
        }
    }

    /// Put item in cache
    pub fn put(&mut self, entry: CacheEntry) {
        // Evict if necessary
        while self.total_size_bytes + entry.size_bytes > self.max_size_bytes && !self.entries.is_empty() {
            self.evict_lru();
        }
        
        self.total_size_bytes += entry.size_bytes;
        self.entries.insert(entry.url.clone(), entry);
    }

    /// Evict least recently used entry
    fn evict_lru(&mut self) {
        if let Some(lru_url) = self.entries.iter()
            .filter(|(_, e)| e.priority != CachePriority::Critical)
            .min_by_key(|(_, e)| e.last_accessed)
            .map(|(url, _)| url.clone())
        {
            if let Some(entry) = self.entries.remove(&lru_url) {
                self.total_size_bytes -= entry.size_bytes;
                debug!("Evicted from cache: {}", lru_url);
            }
        }
    }

    /// Get prefetch candidates
    pub fn get_prefetch_candidates(&self, current_url: &str, limit: usize) -> Vec<String> {
        self.prediction_model.predict_next(current_url, limit)
            .into_iter()
            .filter(|url| !self.entries.contains_key(url))
            .collect()
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        CacheStats {
            total_entries: self.entries.len(),
            total_size_mb: self.total_size_bytes / (1024 * 1024),
            max_size_mb: self.max_size_bytes / (1024 * 1024),
            hits: self.hits,
            misses: self.misses,
            hit_rate: if self.hits + self.misses > 0 {
                self.hits as f64 / (self.hits + self.misses) as f64
            } else {
                0.0
            },
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_mb: u64,
    pub max_size_mb: u64,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

/// Performance metrics tracking
#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    page_load_times: VecDeque<LoadTimeRecord>,
    resource_timings: HashMap<String, ResourceTiming>,
    core_web_vitals: CoreWebVitals,
    startup_time_ms: u64,
}

/// Load time record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadTimeRecord {
    pub url: String,
    pub load_time_ms: u64,
    pub dom_content_loaded_ms: u64,
    pub first_paint_ms: u64,
    pub first_contentful_paint_ms: u64,
    pub timestamp: u128,
}

/// Resource timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTiming {
    pub url: String,
    pub resource_type: String,
    pub dns_time_ms: u64,
    pub connect_time_ms: u64,
    pub ttfb_ms: u64,
    pub download_time_ms: u64,
    pub total_time_ms: u64,
    pub size_bytes: u64,
}

/// Core Web Vitals metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoreWebVitals {
    /// Largest Contentful Paint (ms)
    pub lcp_ms: u64,
    /// First Input Delay (ms)
    pub fid_ms: u64,
    /// Cumulative Layout Shift (score)
    pub cls: f64,
    /// First Contentful Paint (ms)
    pub fcp_ms: u64,
    /// Time to First Byte (ms)
    pub ttfb_ms: u64,
    /// Interaction to Next Paint (ms)
    pub inp_ms: u64,
}

impl CoreWebVitals {
    /// Check if vitals are good (green)
    pub fn is_good(&self) -> bool {
        self.lcp_ms <= 2500 && self.fid_ms <= 100 && self.cls <= 0.1
    }

    /// Check if vitals need improvement (yellow)
    pub fn needs_improvement(&self) -> bool {
        !self.is_good() && !(self.lcp_ms > 4000 || self.fid_ms > 300 || self.cls > 0.25)
    }

    /// Get overall score (0-100)
    pub fn score(&self) -> u32 {
        let lcp_score = if self.lcp_ms <= 2500 { 33 } else if self.lcp_ms <= 4000 { 20 } else { 10 };
        let fid_score = if self.fid_ms <= 100 { 33 } else if self.fid_ms <= 300 { 20 } else { 10 };
        let cls_score = if self.cls <= 0.1 { 34 } else if self.cls <= 0.25 { 20 } else { 10 };
        
        lcp_score + fid_score + cls_score
    }
}

/// Resource priority queue for loading
#[derive(Debug, Default)]
pub struct ResourcePriorityQueue {
    queues: HashMap<CachePriority, VecDeque<ResourceRequest>>,
}

/// Resource request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequest {
    pub url: String,
    pub resource_type: String,
    pub priority: CachePriority,
    pub requested_at: u128,
}

impl ResourcePriorityQueue {
    /// Create a new priority queue
    pub fn new() -> Self {
        let mut queues = HashMap::new();
        queues.insert(CachePriority::Critical, VecDeque::new());
        queues.insert(CachePriority::High, VecDeque::new());
        queues.insert(CachePriority::Medium, VecDeque::new());
        queues.insert(CachePriority::Low, VecDeque::new());
        queues.insert(CachePriority::Prefetch, VecDeque::new());
        Self { queues }
    }

    /// Add a resource request
    pub fn enqueue(&mut self, request: ResourceRequest) {
        if let Some(queue) = self.queues.get_mut(&request.priority) {
            queue.push_back(request);
        }
    }

    /// Get next resource to load
    pub fn dequeue(&mut self) -> Option<ResourceRequest> {
        for priority in &[CachePriority::Critical, CachePriority::High, CachePriority::Medium, CachePriority::Low, CachePriority::Prefetch] {
            if let Some(queue) = self.queues.get_mut(priority) {
                if let Some(request) = queue.pop_front() {
                    return Some(request);
                }
            }
        }
        None
    }

    /// Get pending count
    pub fn pending_count(&self) -> usize {
        self.queues.values().map(|q| q.len()).sum()
    }
}

impl PerformanceOptimizer {
    /// Create a new performance optimizer
    pub fn new() -> Self {
        Self::with_config(PerformanceConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: PerformanceConfig) -> Self {
        info!(
            "Initializing PerformanceOptimizer: lazy_loading={}, predictive_caching={}",
            config.lazy_loading_enabled, config.predictive_caching_enabled
        );
        
        Self {
            cache: Arc::new(RwLock::new(PredictiveCache::new(config.max_cache_size_mb))),
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            config,
            start_time: Instant::now(),
            resource_queue: Arc::new(RwLock::new(ResourcePriorityQueue::new())),
        }
    }

    /// Record page load time
    pub async fn record_page_load(&self, record: LoadTimeRecord) {
        let mut metrics = self.metrics.write().await;
        metrics.page_load_times.push_back(record);
        
        if metrics.page_load_times.len() > 100 {
            metrics.page_load_times.pop_front();
        }
    }

    /// Record resource timing
    pub async fn record_resource_timing(&self, timing: ResourceTiming) {
        let mut metrics = self.metrics.write().await;
        metrics.resource_timings.insert(timing.url.clone(), timing);
    }

    /// Update Core Web Vitals
    pub async fn update_web_vitals(&self, vitals: CoreWebVitals) {
        let mut metrics = self.metrics.write().await;
        metrics.core_web_vitals = vitals;
    }

    /// Get cache entry
    pub async fn get_cached(&self, url: &str) -> Option<CacheEntry> {
        let mut cache = self.cache.write().await;
        let timestamp = self.start_time.elapsed().as_millis();
        cache.get(url, timestamp).cloned()
    }

    /// Cache a resource
    pub async fn cache_resource(&self, entry: CacheEntry) {
        let mut cache = self.cache.write().await;
        cache.put(entry);
    }

    /// Get prefetch candidates
    pub async fn get_prefetch_candidates(&self, current_url: &str) -> Vec<String> {
        if !self.config.prefetch_enabled {
            return Vec::new();
        }
        
        let cache = self.cache.read().await;
        cache.get_prefetch_candidates(current_url, self.config.prefetch_limit)
    }

    /// Enqueue resource for loading
    pub async fn enqueue_resource(&self, url: &str, resource_type: &str, priority: CachePriority) {
        let request = ResourceRequest {
            url: url.to_string(),
            resource_type: resource_type.to_string(),
            priority,
            requested_at: self.start_time.elapsed().as_millis(),
        };
        
        let mut queue = self.resource_queue.write().await;
        queue.enqueue(request);
    }

    /// Get next resource to load
    pub async fn get_next_resource(&self) -> Option<ResourceRequest> {
        let mut queue = self.resource_queue.write().await;
        queue.dequeue()
    }

    /// Get performance report
    pub async fn get_report(&self) -> PerformanceReport {
        let metrics = self.metrics.read().await;
        let cache = self.cache.read().await;
        let queue = self.resource_queue.read().await;
        
        let avg_load_time = if metrics.page_load_times.is_empty() {
            0
        } else {
            metrics.page_load_times.iter().map(|r| r.load_time_ms).sum::<u64>()
                / metrics.page_load_times.len() as u64
        };
        
        PerformanceReport {
            average_load_time_ms: avg_load_time,
            cache_stats: cache.get_stats(),
            web_vitals: metrics.core_web_vitals.clone(),
            startup_time_ms: metrics.startup_time_ms,
            pending_resources: queue.pending_count(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }

    /// Optimize resource loading priority
    pub fn determine_priority(&self, url: &str, resource_type: &str) -> CachePriority {
        let lower_type = resource_type.to_lowercase();
        
        if lower_type.contains("html") || lower_type.contains("critical") {
            CachePriority::Critical
        } else if lower_type.contains("script") || lower_type.contains("font") || lower_type.contains("css") {
            CachePriority::High
        } else if lower_type.contains("image") || lower_type.contains("video") {
            CachePriority::Medium
        } else if lower_type.contains("analytics") || lower_type.contains("tracking") {
            CachePriority::Low
        } else {
            CachePriority::Medium
        }
    }

    /// Clear all caches
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.entries.clear();
        cache.total_size_bytes = 0;
        info!("Performance cache cleared");
    }

    /// Get configuration
    pub fn get_config(&self) -> &PerformanceConfig {
        &self.config
    }
}

/// Performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub average_load_time_ms: u64,
    pub cache_stats: CacheStats,
    pub web_vitals: CoreWebVitals,
    pub startup_time_ms: u64,
    pub pending_resources: usize,
    pub uptime_seconds: u64,
}

impl Default for PerformanceOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_optimizer_creation() {
        let optimizer = PerformanceOptimizer::new();
        let report = optimizer.get_report().await;
        assert_eq!(report.pending_resources, 0);
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let optimizer = PerformanceOptimizer::new();
        
        let entry = CacheEntry {
            url: "https://example.com/script.js".to_string(),
            content_type: "application/javascript".to_string(),
            size_bytes: 1024,
            last_accessed: 0,
            access_count: 0,
            priority: CachePriority::High,
            expires: None,
            compressed: false,
        };
        
        optimizer.cache_resource(entry.clone()).await;
        
        let cached = optimizer.get_cached("https://example.com/script.js").await;
        assert!(cached.is_some());
    }

    #[tokio::test]
    async fn test_resource_queue() {
        let optimizer = PerformanceOptimizer::new();
        
        optimizer.enqueue_resource("https://example.com/low.js", "script", CachePriority::Low).await;
        optimizer.enqueue_resource("https://example.com/critical.html", "html", CachePriority::Critical).await;
        
        let next = optimizer.get_next_resource().await;
        assert!(next.is_some());
        assert_eq!(next.expect("Operation should succeed in test").priority, CachePriority::Critical);
    }

    #[test]
    fn test_web_vitals_scoring() {
        let good_vitals = CoreWebVitals {
            lcp_ms: 2000,
            fid_ms: 50,
            cls: 0.05,
            fcp_ms: 1000,
            ttfb_ms: 200,
            inp_ms: 100,
        };
        
        assert!(good_vitals.is_good());
        assert_eq!(good_vitals.score(), 100);
        
        let poor_vitals = CoreWebVitals {
            lcp_ms: 5000,
            fid_ms: 400,
            cls: 0.5,
            ..Default::default()
        };
        
        assert!(!poor_vitals.is_good());
        assert!(poor_vitals.score() < 50);
    }

    #[test]
    fn test_priority_determination() {
        let optimizer = PerformanceOptimizer::new();
        
        assert_eq!(optimizer.determine_priority("test.html", "text/html"), CachePriority::Critical);
        assert_eq!(optimizer.determine_priority("app.js", "script"), CachePriority::High);
        assert_eq!(optimizer.determine_priority("photo.jpg", "image/jpeg"), CachePriority::Medium);
        assert_eq!(optimizer.determine_priority("ga.js", "analytics"), CachePriority::Low);
    }
}
