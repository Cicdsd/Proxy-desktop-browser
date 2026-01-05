//! Efficiency Module
//!
//! Provides comprehensive efficiency optimizations including:
//! - Memory pool management for reduced allocations
//! - Buffer pooling for network operations
//! - CPU optimization utilities
//! - Startup time improvements
//! - Performance monitoring and tuning

use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Memory Pool Implementation (#970, #1171, #488, #521)
// ============================================================================

/// A thread-safe memory pool for efficient buffer reuse
/// Addresses issues: #970, #1171, #488, #521
#[derive(Debug)]
pub struct MemoryPool {
    pools: Arc<RwLock<Vec<VecDeque<Vec<u8>>>>>,
    pool_sizes: Vec<usize>,
    stats: PoolStats,
}

#[derive(Debug, Default)]
pub struct PoolStats {
    pub allocations: AtomicUsize,
    pub reuses: AtomicUsize,
    pub deallocations: AtomicUsize,
}

impl MemoryPool {
    /// Create a new memory pool with predefined buffer sizes
    /// Sizes: 1KB, 4KB, 16KB, 64KB, 256KB, 1MB
    pub fn new() -> Self {
        let pool_sizes = vec![1024, 4096, 16384, 65536, 262144, 1048576];
        let pools = pool_sizes.iter().map(|_| VecDeque::new()).collect();
        
        Self {
            pools: Arc::new(RwLock::new(pools)),
            pool_sizes,
            stats: PoolStats::default(),
        }
    }

    /// Acquire a buffer of at least the specified size
    pub async fn acquire(&self, min_size: usize) -> Vec<u8> {
        let pool_idx = self.find_pool_index(min_size);
        
        if let Some(idx) = pool_idx {
            let mut pools = self.pools.write().await;
            if let Some(buffer) = pools[idx].pop_front() {
                self.stats.reuses.fetch_add(1, Ordering::Relaxed);
                return buffer;
            }
        }
        
        self.stats.allocations.fetch_add(1, Ordering::Relaxed);
        let size = pool_idx.map(|i| self.pool_sizes[i]).unwrap_or(min_size);
        vec![0u8; size]
    }

    /// Release a buffer back to the pool
    pub async fn release(&self, buffer: Vec<u8>) {
        let size = buffer.len();
        if let Some(idx) = self.find_pool_index(size) {
            let mut pools = self.pools.write().await;
            // Limit pool size to prevent memory bloat
            if pools[idx].len() < 100 {
                pools[idx].push_back(buffer);
                self.stats.deallocations.fetch_add(1, Ordering::Relaxed);
                return;
            }
        }
        // Buffer dropped if pool is full or size doesn't match
    }

    fn find_pool_index(&self, size: usize) -> Option<usize> {
        self.pool_sizes.iter().position(|&s| s >= size)
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> (usize, usize, usize) {
        (
            self.stats.allocations.load(Ordering::Relaxed),
            self.stats.reuses.load(Ordering::Relaxed),
            self.stats.deallocations.load(Ordering::Relaxed),
        )
    }
}

impl Default for MemoryPool {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Buffer Efficiency (#969, #978, #1170)
// ============================================================================

/// Efficient buffer management for network operations
/// Addresses issues: #969, #978, #1170
#[derive(Debug)]
pub struct BufferManager {
    pool: MemoryPool,
    max_buffer_size: usize,
}

impl BufferManager {
    pub fn new(max_buffer_size: usize) -> Self {
        Self {
            pool: MemoryPool::new(),
            max_buffer_size,
        }
    }

    /// Get a buffer for reading data
    pub async fn get_read_buffer(&self, expected_size: usize) -> Vec<u8> {
        let size = expected_size.min(self.max_buffer_size);
        self.pool.acquire(size).await
    }

    /// Get a buffer for writing data
    pub async fn get_write_buffer(&self, data_size: usize) -> Vec<u8> {
        let size = data_size.min(self.max_buffer_size);
        self.pool.acquire(size).await
    }

    /// Return a buffer to the pool
    pub async fn return_buffer(&self, buffer: Vec<u8>) {
        self.pool.release(buffer).await;
    }
}

impl Default for BufferManager {
    fn default() -> Self {
        Self::new(1048576) // 1MB default max
    }
}

// ============================================================================
// CPU Optimization (#496-#520, #971, #972)
// ============================================================================

/// CPU optimization utilities
/// Addresses issues: #496-#520, #971, #972
pub struct CpuOptimizer {
    worker_count: usize,
}

impl CpuOptimizer {
    pub fn new() -> Self {
        let worker_count = num_cpus::get().max(1);
        Self { worker_count }
    }

    /// Get optimal number of worker threads
    pub fn optimal_workers(&self) -> usize {
        self.worker_count
    }

    /// Get optimal batch size for parallel processing
    pub fn optimal_batch_size(&self, total_items: usize) -> usize {
        let batch = total_items / self.worker_count;
        batch.max(1).min(1000)
    }
}

impl Default for CpuOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Startup Optimization (#964, #966, #967, #973)
// ============================================================================

/// Lazy initialization wrapper for deferred startup
/// Addresses issues: #964, #966, #967, #973
pub struct LazyInit<T> {
    value: Arc<RwLock<Option<T>>>,
    init_fn: Arc<dyn Fn() -> T + Send + Sync>,
}

impl<T: Clone> LazyInit<T> {
    pub fn new<F>(init_fn: F) -> Self 
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            value: Arc::new(RwLock::new(None)),
            init_fn: Arc::new(init_fn),
        }
    }

    pub async fn get(&self) -> T {
        {
            let read_guard = self.value.read().await;
            if let Some(ref val) = *read_guard {
                return val.clone();
            }
        }
        
        let mut write_guard = self.value.write().await;
        if write_guard.is_none() {
            *write_guard = Some((self.init_fn)());
        }
        write_guard.as_ref().unwrap().clone()
    }
}

// ============================================================================
// Performance Monitoring (#522-#600)
// ============================================================================

/// Performance metrics collector
/// Addresses issues: #522-#600
#[derive(Debug, Default)]
pub struct PerformanceMonitor {
    request_count: AtomicUsize,
    total_latency_ms: AtomicUsize,
    error_count: AtomicUsize,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_request(&self, latency_ms: usize, is_error: bool) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
        self.total_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);
        if is_error {
            self.error_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn get_metrics(&self) -> PerformanceMetrics {
        let requests = self.request_count.load(Ordering::Relaxed);
        let total_latency = self.total_latency_ms.load(Ordering::Relaxed);
        let errors = self.error_count.load(Ordering::Relaxed);
        
        PerformanceMetrics {
            request_count: requests,
            avg_latency_ms: if requests > 0 { total_latency / requests } else { 0 },
            error_rate: if requests > 0 { errors as f64 / requests as f64 } else { 0.0 },
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub request_count: usize,
    pub avg_latency_ms: usize,
    pub error_rate: f64,
}

// ============================================================================
// Cache Optimization (#961, #968, #982, #987)
// ============================================================================

/// LRU Cache with efficient memory management
/// Addresses issues: #961, #968, #982, #987
use std::collections::HashMap;
use std::hash::Hash;

pub struct LruCache<K, V> {
    capacity: usize,
    map: HashMap<K, (V, usize)>,
    access_counter: usize,
}

impl<K: Eq + Hash + Clone, V: Clone> LruCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            map: HashMap::with_capacity(capacity),
            access_counter: 0,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some((value, access)) = self.map.get_mut(key) {
            self.access_counter += 1;
            *access = self.access_counter;
            Some(value.clone())
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        if self.map.len() >= self.capacity && !self.map.contains_key(&key) {
            // Remove least recently used
            if let Some(lru_key) = self.find_lru() {
                self.map.remove(&lru_key);
            }
        }
        
        self.access_counter += 1;
        self.map.insert(key, (value, self.access_counter));
    }

    fn find_lru(&self) -> Option<K> {
        self.map
            .iter()
            .min_by_key(|(_, (_, access))| access)
            .map(|(k, _)| k.clone())
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

// ============================================================================
// Connection Pool (#954, #962)
// ============================================================================

/// Connection pool for efficient connection reuse
/// Addresses issues: #954, #962
#[derive(Debug)]
pub struct ConnectionPool<T> {
    connections: Arc<RwLock<VecDeque<T>>>,
    max_size: usize,
    current_size: AtomicUsize,
}

impl<T> ConnectionPool<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            connections: Arc::new(RwLock::new(VecDeque::new())),
            max_size,
            current_size: AtomicUsize::new(0),
        }
    }

    pub async fn acquire(&self) -> Option<T> {
        let mut connections = self.connections.write().await;
        connections.pop_front()
    }

    pub async fn release(&self, conn: T) {
        let mut connections = self.connections.write().await;
        if connections.len() < self.max_size {
            connections.push_back(conn);
        }
    }

    pub fn size(&self) -> usize {
        self.current_size.load(Ordering::Relaxed)
    }
}

// ============================================================================
// Batch Processor (#934, #975)
// ============================================================================

/// Batch processor for efficient bulk operations
/// Addresses issues: #934, #975
pub struct BatchProcessor<T> {
    batch_size: usize,
    items: Vec<T>,
}

impl<T> BatchProcessor<T> {
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch_size,
            items: Vec::with_capacity(batch_size),
        }
    }

    pub fn add(&mut self, item: T) -> Option<Vec<T>> {
        self.items.push(item);
        if self.items.len() >= self.batch_size {
            Some(std::mem::take(&mut self.items))
        } else {
            None
        }
    }

    pub fn flush(&mut self) -> Vec<T> {
        std::mem::take(&mut self.items)
    }
}

// ============================================================================
// Resource Manager (#984)
// ============================================================================

/// Resource manager for configuration optimization
/// Addresses issue: #984
pub struct ResourceManager {
    memory_limit: usize,
    cpu_limit: f64,
}

impl ResourceManager {
    pub fn new(memory_limit: usize, cpu_limit: f64) -> Self {
        Self {
            memory_limit,
            cpu_limit,
        }
    }

    pub fn memory_limit(&self) -> usize {
        self.memory_limit
    }

    pub fn cpu_limit(&self) -> f64 {
        self.cpu_limit
    }

    pub fn should_throttle(&self, current_memory: usize, current_cpu: f64) -> bool {
        current_memory > self.memory_limit || current_cpu > self.cpu_limit
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new(1024 * 1024 * 1024, 0.8) // 1GB memory, 80% CPU
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_pool() {
        let pool = MemoryPool::new();
        
        let buf1 = pool.acquire(1000).await;
        assert!(buf1.len() >= 1000);
        
        pool.release(buf1).await;
        
        let buf2 = pool.acquire(1000).await;
        assert!(buf2.len() >= 1000);
    }

    #[tokio::test]
    async fn test_buffer_manager() {
        let manager = BufferManager::default();
        
        let buf = manager.get_read_buffer(4096).await;
        assert!(buf.len() >= 4096);
        
        manager.return_buffer(buf).await;
    }

    #[test]
    fn test_cpu_optimizer() {
        let optimizer = CpuOptimizer::new();
        assert!(optimizer.optimal_workers() >= 1);
        assert!(optimizer.optimal_batch_size(1000) >= 1);
    }

    #[test]
    fn test_lru_cache() {
        let mut cache = LruCache::new(2);
        
        cache.insert("a", 1);
        cache.insert("b", 2);
        
        assert_eq!(cache.get(&"a"), Some(1));
        
        cache.insert("c", 3);
        
        // "b" should be evicted as LRU
        assert_eq!(cache.get(&"b"), None);
        assert_eq!(cache.get(&"c"), Some(3));
    }

    #[test]
    fn test_batch_processor() {
        let mut processor = BatchProcessor::new(3);
        
        assert!(processor.add(1).is_none());
        assert!(processor.add(2).is_none());
        
        let batch = processor.add(3);
        assert_eq!(batch, Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new();
        
        monitor.record_request(100, false);
        monitor.record_request(200, false);
        monitor.record_request(150, true);
        
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.request_count, 3);
        assert_eq!(metrics.avg_latency_ms, 150);
    }

    #[test]
    fn test_resource_manager() {
        let manager = ResourceManager::default();
        
        assert!(!manager.should_throttle(100, 0.5));
        assert!(manager.should_throttle(2 * 1024 * 1024 * 1024, 0.5));
        assert!(manager.should_throttle(100, 0.9));
    }
}
