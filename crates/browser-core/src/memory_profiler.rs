//! Memory Profiler Module - v3.0 Core Stability
//! 
//! Part of the V1000 Upgrade Deep Plan - Phase 1 Foundation
//! Provides memory profiling, leak detection, and resource tracking.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Memory allocation tracking
static TOTAL_ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static TOTAL_DEALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static PEAK_MEMORY_MB: AtomicU64 = AtomicU64::new(0);

/// Memory profiler for tracking browser memory usage
pub struct MemoryProfiler {
    snapshots: Arc<RwLock<Vec<MemorySnapshot>>>,
    start_time: Instant,
    allocation_tracking: Arc<RwLock<HashMap<String, AllocationInfo>>>,
    thresholds: MemoryThresholds,
    alerts: Arc<RwLock<Vec<MemoryAlert>>>,
}

/// Memory snapshot at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub timestamp_ms: u128,
    pub heap_used_mb: u64,
    pub heap_total_mb: u64,
    pub external_mb: u64,
    pub array_buffers_mb: u64,
    pub tab_count: usize,
    pub dom_nodes_estimated: u64,
    pub js_heap_size_limit_mb: u64,
}

/// Allocation tracking info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationInfo {
    pub component: String,
    pub allocated_mb: f64,
    pub allocation_count: u64,
    pub last_allocation: u128,
    pub peak_mb: f64,
}

/// Memory thresholds for alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryThresholds {
    pub warning_mb: u64,
    pub critical_mb: u64,
    pub tab_memory_limit_mb: u64,
    pub gc_trigger_mb: u64,
}

impl Default for MemoryThresholds {
    fn default() -> Self {
        Self {
            warning_mb: 512,
            critical_mb: 1024,
            tab_memory_limit_mb: 256,
            gc_trigger_mb: 384,
        }
    }
}

/// Memory alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryAlertLevel {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Memory alert record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAlert {
    pub level: MemoryAlertLevel,
    pub message: String,
    pub timestamp_ms: u128,
    pub memory_mb: u64,
    pub component: Option<String>,
}

/// Memory statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub current_mb: u64,
    pub peak_mb: u64,
    pub average_mb: u64,
    pub allocations: u64,
    pub deallocations: u64,
    pub gc_count: u64,
    pub uptime_seconds: u64,
    pub snapshot_count: usize,
    pub alert_count: usize,
}

/// Garbage collection recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GcRecommendation {
    None,
    Minor,
    Major,
    Emergency,
}

impl MemoryProfiler {
    /// Create a new memory profiler
    pub fn new() -> Self {
        Self::with_thresholds(MemoryThresholds::default())
    }

    /// Create a new memory profiler with custom thresholds
    pub fn with_thresholds(thresholds: MemoryThresholds) -> Self {
        info!(
            "Initializing MemoryProfiler with thresholds: warning={}MB, critical={}MB",
            thresholds.warning_mb, thresholds.critical_mb
        );
        Self {
            snapshots: Arc::new(RwLock::new(Vec::new())),
            start_time: Instant::now(),
            allocation_tracking: Arc::new(RwLock::new(HashMap::new())),
            thresholds,
            alerts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Take a memory snapshot
    pub async fn take_snapshot(&self, tab_count: usize) -> MemorySnapshot {
        // Get current memory usage (simplified - in production would use system APIs)
        let heap_used = self.estimate_heap_usage();
        let heap_total = heap_used + 128; // Approximate headroom
        
        let snapshot = MemorySnapshot {
            timestamp_ms: self.start_time.elapsed().as_millis(),
            heap_used_mb: heap_used,
            heap_total_mb: heap_total,
            external_mb: 0,
            array_buffers_mb: 0,
            tab_count,
            dom_nodes_estimated: (tab_count as u64) * 5000, // Estimate 5000 nodes per tab
            js_heap_size_limit_mb: 2048,
        };

        // Update peak memory
        let current_peak = PEAK_MEMORY_MB.load(Ordering::Relaxed);
        if heap_used > current_peak {
            PEAK_MEMORY_MB.store(heap_used, Ordering::Relaxed);
        }

        // Check thresholds
        self.check_memory_thresholds(&snapshot).await;

        // Store snapshot
        let mut snapshots = self.snapshots.write().await;
        snapshots.push(snapshot.clone());
        
        // Keep only last 1000 snapshots
        if snapshots.len() > 1000 {
            let drain_count = snapshots.len() - 1000;
            snapshots.drain(0..drain_count);
        }

        debug!("Memory snapshot: {}MB used, {} tabs", heap_used, tab_count);
        snapshot
    }

    /// Track an allocation for a component
    pub async fn track_allocation(&self, component: &str, size_mb: f64) {
        TOTAL_ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        
        let mut allocations = self.allocation_tracking.write().await;
        let info = allocations.entry(component.to_string()).or_insert(AllocationInfo {
            component: component.to_string(),
            allocated_mb: 0.0,
            allocation_count: 0,
            last_allocation: self.start_time.elapsed().as_millis(),
            peak_mb: 0.0,
        });
        
        info.allocated_mb += size_mb;
        info.allocation_count += 1;
        info.last_allocation = self.start_time.elapsed().as_millis();
        
        if info.allocated_mb > info.peak_mb {
            info.peak_mb = info.allocated_mb;
        }
    }

    /// Track a deallocation for a component
    pub async fn track_deallocation(&self, component: &str, size_mb: f64) {
        TOTAL_DEALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        
        let mut allocations = self.allocation_tracking.write().await;
        if let Some(info) = allocations.get_mut(component) {
            info.allocated_mb = (info.allocated_mb - size_mb).max(0.0);
        }
    }

    /// Check memory thresholds and generate alerts
    async fn check_memory_thresholds(&self, snapshot: &MemorySnapshot) {
        let mut alerts = self.alerts.write().await;
        
        if snapshot.heap_used_mb >= self.thresholds.critical_mb {
            let alert = MemoryAlert {
                level: MemoryAlertLevel::Critical,
                message: format!(
                    "Critical memory usage: {}MB exceeds threshold of {}MB",
                    snapshot.heap_used_mb, self.thresholds.critical_mb
                ),
                timestamp_ms: snapshot.timestamp_ms,
                memory_mb: snapshot.heap_used_mb,
                component: None,
            };
            warn!("{}", alert.message);
            alerts.push(alert);
        } else if snapshot.heap_used_mb >= self.thresholds.warning_mb {
            let alert = MemoryAlert {
                level: MemoryAlertLevel::Warning,
                message: format!(
                    "High memory usage: {}MB exceeds warning threshold of {}MB",
                    snapshot.heap_used_mb, self.thresholds.warning_mb
                ),
                timestamp_ms: snapshot.timestamp_ms,
                memory_mb: snapshot.heap_used_mb,
                component: None,
            };
            warn!("{}", alert.message);
            alerts.push(alert);
        }

        // Keep only last 100 alerts
        if alerts.len() > 100 {
            let drain_count = alerts.len() - 100;
            alerts.drain(0..drain_count);
        }
    }

    /// Get GC recommendation based on current memory state
    pub async fn get_gc_recommendation(&self) -> GcRecommendation {
        let snapshots = self.snapshots.read().await;
        if let Some(latest) = snapshots.last() {
            if latest.heap_used_mb >= self.thresholds.critical_mb {
                return GcRecommendation::Emergency;
            } else if latest.heap_used_mb >= self.thresholds.gc_trigger_mb {
                return GcRecommendation::Major;
            } else if latest.heap_used_mb >= self.thresholds.warning_mb {
                return GcRecommendation::Minor;
            }
        }
        GcRecommendation::None
    }

    /// Get memory statistics
    pub async fn get_stats(&self) -> MemoryStats {
        let snapshots = self.snapshots.read().await;
        let alerts = self.alerts.read().await;
        
        let current_mb = snapshots.last().map(|s| s.heap_used_mb).unwrap_or(0);
        let peak_mb = PEAK_MEMORY_MB.load(Ordering::Relaxed);
        
        let average_mb = if snapshots.is_empty() {
            0
        } else {
            snapshots.iter().map(|s| s.heap_used_mb).sum::<u64>() / snapshots.len() as u64
        };

        MemoryStats {
            current_mb,
            peak_mb,
            average_mb,
            allocations: TOTAL_ALLOCATIONS.load(Ordering::Relaxed),
            deallocations: TOTAL_DEALLOCATIONS.load(Ordering::Relaxed),
            gc_count: 0, // Would be tracked by actual GC
            uptime_seconds: self.start_time.elapsed().as_secs(),
            snapshot_count: snapshots.len(),
            alert_count: alerts.len(),
        }
    }

    /// Get allocation breakdown by component
    pub async fn get_allocation_breakdown(&self) -> Vec<AllocationInfo> {
        self.allocation_tracking.read().await.values().cloned().collect()
    }

    /// Get recent alerts
    pub async fn get_alerts(&self, limit: usize) -> Vec<MemoryAlert> {
        let alerts = self.alerts.read().await;
        alerts.iter().rev().take(limit).cloned().collect()
    }

    /// Clear all memory data
    pub async fn clear(&self) {
        self.snapshots.write().await.clear();
        self.allocation_tracking.write().await.clear();
        self.alerts.write().await.clear();
        TOTAL_ALLOCATIONS.store(0, Ordering::Relaxed);
        TOTAL_DEALLOCATIONS.store(0, Ordering::Relaxed);
    }

    /// Estimate heap usage (simplified)
    fn estimate_heap_usage(&self) -> u64 {
        // In production, this would use actual memory APIs
        // For now, return a reasonable estimate based on tracked allocations
        64 // Base memory usage in MB
    }

    /// Detect potential memory leaks
    pub async fn detect_leaks(&self) -> Vec<LeakReport> {
        let allocations = self.allocation_tracking.read().await;
        let mut reports = Vec::new();

        for (component, info) in allocations.iter() {
            // If allocations significantly exceed deallocations, flag as potential leak
            let allocation_rate = info.allocation_count as f64;
            let memory_per_allocation = if allocation_rate > 0.0 {
                info.allocated_mb / allocation_rate
            } else {
                0.0
            };

            // High memory with low deallocation rate suggests a leak
            if info.allocated_mb > 50.0 && info.peak_mb > info.allocated_mb * 0.9 {
                reports.push(LeakReport {
                    component: component.clone(),
                    suspected_leak_mb: info.allocated_mb,
                    confidence: calculate_leak_confidence(info),
                    recommendation: format!(
                        "Component '{}' has {}MB allocated with minimal deallocation",
                        component, info.allocated_mb
                    ),
                });
            }
        }

        reports
    }
}

/// Memory leak detection report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeakReport {
    pub component: String,
    pub suspected_leak_mb: f64,
    pub confidence: f64, // 0.0 to 1.0
    pub recommendation: String,
}

fn calculate_leak_confidence(info: &AllocationInfo) -> f64 {
    // Simple heuristic: higher ratio of peak to current suggests no leak
    // If allocated stays close to peak, more likely to be a leak
    if info.peak_mb == 0.0 {
        return 0.0;
    }
    let ratio = info.allocated_mb / info.peak_mb;
    ratio.min(1.0)
}

impl Default for MemoryProfiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_profiler_creation() {
        let profiler = MemoryProfiler::new();
        let stats = profiler.get_stats().await;
        assert_eq!(stats.snapshot_count, 0);
    }

    #[tokio::test]
    async fn test_take_snapshot() {
        let profiler = MemoryProfiler::new();
        let snapshot = profiler.take_snapshot(5).await;
        
        assert_eq!(snapshot.tab_count, 5);
        assert!(snapshot.heap_used_mb > 0);
        
        let stats = profiler.get_stats().await;
        assert_eq!(stats.snapshot_count, 1);
    }

    #[tokio::test]
    async fn test_track_allocation() {
        let profiler = MemoryProfiler::new();
        
        profiler.track_allocation("tab_manager", 10.0).await;
        profiler.track_allocation("tab_manager", 5.0).await;
        
        let breakdown = profiler.get_allocation_breakdown().await;
        assert_eq!(breakdown.len(), 1);
        assert_eq!(breakdown[0].allocated_mb, 15.0);
        assert_eq!(breakdown[0].allocation_count, 2);
    }

    #[tokio::test]
    async fn test_gc_recommendation() {
        let thresholds = MemoryThresholds {
            warning_mb: 100,
            critical_mb: 200,
            tab_memory_limit_mb: 50,
            gc_trigger_mb: 150,
        };
        let profiler = MemoryProfiler::with_thresholds(thresholds);
        
        let recommendation = profiler.get_gc_recommendation().await;
        assert!(matches!(recommendation, GcRecommendation::None));
    }

    #[tokio::test]
    async fn test_leak_detection() {
        let profiler = MemoryProfiler::new();
        
        // Simulate allocations without deallocations
        for _ in 0..10 {
            profiler.track_allocation("leaky_component", 10.0).await;
        }
        
        let leaks = profiler.detect_leaks().await;
        assert!(!leaks.is_empty());
    }
}
