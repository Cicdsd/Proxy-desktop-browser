//! Network Intelligence Module - v30.0 Network Intelligence
//!
//! Part of the V1000 Upgrade Deep Plan - Phase 2 Feature Expansion
//! Provides AI-based traffic optimization, bandwidth management, and QoS.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Network intelligence manager
pub struct NetworkIntelligence {
    traffic_analyzer: Arc<RwLock<TrafficAnalyzer>>,
    bandwidth_manager: Arc<RwLock<BandwidthManager>>,
    qos_manager: Arc<RwLock<QosManager>>,
    connection_pool: Arc<RwLock<ConnectionPool>>,
    config: NetworkIntelligenceConfig,
    start_time: Instant,
}

/// Configuration for network intelligence
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a NetworkIntelligenceConfig.
pub struct NetworkIntelligenceConfig {
    /// Enable traffic analysis
    pub traffic_analysis_enabled: bool,
    /// Enable bandwidth management
    pub bandwidth_management_enabled: bool,
    /// Enable QoS prioritization
    pub qos_enabled: bool,
    /// Maximum bandwidth (bytes/sec, 0 = unlimited)
    pub max_bandwidth_bps: u64,
    /// Enable connection pooling
    pub connection_pooling_enabled: bool,
    /// Maximum connections per host
    pub max_connections_per_host: u32,
    /// Enable DNS prefetching
    pub dns_prefetch_enabled: bool,
    /// Enable connection preconnect
    pub preconnect_enabled: bool,
}

impl Default for NetworkIntelligenceConfig {
    fn default() -> Self {
        Self {
            traffic_analysis_enabled: true,
            bandwidth_management_enabled: true,
            qos_enabled: true,
            max_bandwidth_bps: 0, // Unlimited
            connection_pooling_enabled: true,
            max_connections_per_host: 6,
            dns_prefetch_enabled: true,
            preconnect_enabled: true,
        }
    }
}

/// Traffic analyzer for understanding network patterns
#[derive(Debug)]
/// Represents a TrafficAnalyzer.
#[allow(dead_code)]
#[allow(dead_code)]
pub struct TrafficAnalyzer {
    request_history: VecDeque<RequestRecord>,
    domain_stats: HashMap<String, DomainStats>,
    protocol_stats: HashMap<String, ProtocolStats>,
    hourly_traffic: [u64; 24],
    total_bytes_sent: u64,
    total_bytes_received: u64,
    total_requests: u64,
}

/// Request record for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a RequestRecord.
pub struct RequestRecord {
    pub url: String,
    pub domain: String,
    pub method: String,
    pub status_code: u16,
    pub request_size_bytes: u64,
    pub response_size_bytes: u64,
    pub latency_ms: u64,
    pub timestamp: u128,
    pub content_type: String,
    pub cached: bool,
}

/// Domain-level statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Represents a DomainStats.
pub struct DomainStats {
    pub domain: String,
    pub request_count: u64,
    pub total_bytes: u64,
    pub average_latency_ms: u64,
    pub error_count: u64,
    pub success_rate: f64,
    pub last_request: u128,
}

/// Protocol statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Represents a ProtocolStats.
pub struct ProtocolStats {
    pub protocol: String,
    pub request_count: u64,
    pub total_bytes: u64,
    pub average_latency_ms: u64,
}

impl TrafficAnalyzer {
    /// Create a new traffic analyzer
    pub fn new() -> Self {
        Self {
            request_history: VecDeque::with_capacity(10000),
            domain_stats: HashMap::new(),
            protocol_stats: HashMap::new(),
            hourly_traffic: [0; 24],
            total_bytes_sent: 0,
            total_bytes_received: 0,
            total_requests: 0,
        }
    }

    /// Record a network request
    pub fn record_request(&mut self, record: RequestRecord) {
        self.total_requests += 1;
        self.total_bytes_sent += record.request_size_bytes;
        self.total_bytes_received += record.response_size_bytes;

        // Update domain stats
        let domain_stat = self.domain_stats.entry(record.domain.clone())
            .or_insert_with(|| DomainStats {
                domain: record.domain.clone(),
                ..Default::default()
            });
        
        domain_stat.request_count += 1;
        domain_stat.total_bytes += record.response_size_bytes;
        domain_stat.last_request = record.timestamp;
        
        if record.status_code >= 200 && record.status_code < 400 {
            domain_stat.success_rate = 
                (domain_stat.success_rate * (domain_stat.request_count - 1) as f64 + 1.0) 
                / domain_stat.request_count as f64;
        } else {
            domain_stat.error_count += 1;
            domain_stat.success_rate = 
                (domain_stat.success_rate * (domain_stat.request_count - 1) as f64) 
                / domain_stat.request_count as f64;
        }
        
        // Update running average latency
        domain_stat.average_latency_ms = 
            (domain_stat.average_latency_ms * (domain_stat.request_count - 1) + record.latency_ms)
            / domain_stat.request_count;

        // Update protocol stats
        let protocol = if record.url.starts_with("https") { "https" } else { "http" };
        let protocol_stat = self.protocol_stats.entry(protocol.to_string())
            .or_insert_with(|| ProtocolStats {
                protocol: protocol.to_string(),
                ..Default::default()
            });
        protocol_stat.request_count += 1;
        protocol_stat.total_bytes += record.response_size_bytes;

        // Store in history
        self.request_history.push_back(record);
        if self.request_history.len() > 10000 {
            self.request_history.pop_front();
        }
    }

    /// Predict likely next requests based on patterns
    pub fn predict_next_requests(&self, current_url: &str, limit: usize) -> Vec<String> {
        // Simple prediction based on domain frequency
        let current_domain = extract_domain(current_url);
        
        let mut candidates: Vec<_> = self.domain_stats.iter()
            .filter(|(d, _)| *d != &current_domain)
            .map(|(_, stats)| (stats.domain.clone(), stats.request_count))
            .collect();
        
        candidates.sort_by(|a, b| b.1.cmp(&a.1));
        
        candidates.into_iter()
            .take(limit)
            .map(|(domain, _)| format!("https://{}", domain))
            .collect()
    }

    /// Get traffic analysis report
    pub fn get_report(&self) -> TrafficReport {
        let top_domains: Vec<_> = {
            let mut sorted: Vec<_> = self.domain_stats.values().cloned().collect();
            sorted.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));
            sorted.into_iter().take(10).collect()
        };

        TrafficReport {
            total_requests: self.total_requests,
            total_bytes_sent: self.total_bytes_sent,
            total_bytes_received: self.total_bytes_received,
            unique_domains: self.domain_stats.len(),
            top_domains,
            protocol_breakdown: self.protocol_stats.values().cloned().collect(),
        }
    }
}

impl Default for TrafficAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Traffic analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a TrafficReport.
pub struct TrafficReport {
    pub total_requests: u64,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub unique_domains: usize,
    pub top_domains: Vec<DomainStats>,
    pub protocol_breakdown: Vec<ProtocolStats>,
}

/// Bandwidth manager for traffic shaping
#[derive(Debug)]
/// Represents a BandwidthManager.
#[allow(dead_code)]
#[allow(dead_code)]
pub struct BandwidthManager {
    current_usage_bps: u64,
    max_bandwidth_bps: u64,
    allocations: HashMap<String, BandwidthAllocation>,
    history: VecDeque<BandwidthSample>,
}

/// Bandwidth allocation for a component/domain
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a BandwidthAllocation.
pub struct BandwidthAllocation {
    pub id: String,
    pub allocated_bps: u64,
    pub used_bps: u64,
    pub priority: u8, // 0-10, higher = more priority
}

/// Bandwidth sample for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a BandwidthSample.
pub struct BandwidthSample {
    pub timestamp: u128,
    pub download_bps: u64,
    pub upload_bps: u64,
}

impl BandwidthManager {
    /// Create a new bandwidth manager
    pub fn new(max_bandwidth_bps: u64) -> Self {
        Self {
            current_usage_bps: 0,
            max_bandwidth_bps,
            allocations: HashMap::new(),
            history: VecDeque::with_capacity(1000),
        }
    }

    /// Allocate bandwidth for a component
    pub fn allocate(&mut self, id: &str, requested_bps: u64, priority: u8) -> u64 {
        let available = self.available_bandwidth();
        let allocated = requested_bps.min(available);
        
        self.allocations.insert(id.to_string(), BandwidthAllocation {
            id: id.to_string(),
            allocated_bps: allocated,
            used_bps: 0,
            priority,
        });
        
        allocated
    }

    /// Release bandwidth allocation
    pub fn release(&mut self, id: &str) {
        self.allocations.remove(id);
    }

    /// Record bandwidth usage
    pub fn record_usage(&mut self, id: &str, bytes: u64, duration_ms: u64) {
        if duration_ms == 0 {
            return;
        }
        
        let bps = bytes * 1000 / duration_ms;
        
        if let Some(alloc) = self.allocations.get_mut(id) {
            alloc.used_bps = bps;
        }
        
        self.current_usage_bps = self.allocations.values()
            .map(|a| a.used_bps)
            .sum();
    }

    /// Get available bandwidth
    pub fn available_bandwidth(&self) -> u64 {
        if self.max_bandwidth_bps == 0 {
            return u64::MAX;
        }
        self.max_bandwidth_bps.saturating_sub(self.current_usage_bps)
    }

    /// Check if bandwidth is throttled
    pub fn is_throttled(&self) -> bool {
        self.max_bandwidth_bps > 0 && self.current_usage_bps >= self.max_bandwidth_bps * 90 / 100
    }

    /// Get bandwidth report
    pub fn get_report(&self) -> BandwidthReport {
        BandwidthReport {
            current_usage_bps: self.current_usage_bps,
            max_bandwidth_bps: self.max_bandwidth_bps,
            available_bps: self.available_bandwidth(),
            utilization_percent: if self.max_bandwidth_bps > 0 {
                (self.current_usage_bps as f64 / self.max_bandwidth_bps as f64 * 100.0) as u8
            } else {
                0
            },
            allocations: self.allocations.values().cloned().collect(),
        }
    }
}

/// Bandwidth report
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a BandwidthReport.
pub struct BandwidthReport {
    pub current_usage_bps: u64,
    pub max_bandwidth_bps: u64,
    pub available_bps: u64,
    pub utilization_percent: u8,
    pub allocations: Vec<BandwidthAllocation>,
}

/// Quality of Service manager
#[derive(Debug)]
/// Represents a QosManager.
pub struct QosManager {
    queues: HashMap<QosPriority, VecDeque<QosRequest>>,
    active_requests: HashMap<String, QosPriority>,
    stats: QosStats,
}

/// QoS priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Enumeration of QosPriority variants.
pub enum QosPriority {
    Critical,     // User-initiated, blocking
    Interactive,  // User actions, visible
    High,         // Important resources
    Normal,       // Regular requests
    Low,          // Background, prefetch
    Bulk,         // Large downloads, updates
}

impl Default for QosPriority {
    fn default() -> Self {
        QosPriority::Normal
    }
}

/// QoS request
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a QosRequest.
pub struct QosRequest {
    pub id: String,
    pub url: String,
    pub priority: QosPriority,
    pub queued_at: u128,
    pub deadline_ms: Option<u64>,
}

/// QoS statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Represents a QosStats.
pub struct QosStats {
    pub requests_by_priority: HashMap<QosPriority, u64>,
    pub total_wait_time_ms: u64,
    pub average_wait_time_ms: u64,
    pub deadline_violations: u64,
}

impl QosManager {
    /// Create a new QoS manager
    pub fn new() -> Self {
        let mut queues = HashMap::new();
        for priority in [QosPriority::Critical, QosPriority::Interactive, QosPriority::High, 
                        QosPriority::Normal, QosPriority::Low, QosPriority::Bulk] {
            queues.insert(priority, VecDeque::new());
        }
        
        Self {
            queues,
            active_requests: HashMap::new(),
            stats: QosStats::default(),
        }
    }

    /// Enqueue a request
    pub fn enqueue(&mut self, request: QosRequest) {
        let count = self.stats.requests_by_priority
            .entry(request.priority)
            .or_insert(0);
        *count += 1;
        
        if let Some(queue) = self.queues.get_mut(&request.priority) {
            queue.push_back(request);
        }
    }

    /// Get next request to process
    pub fn dequeue(&mut self, current_time: u128) -> Option<QosRequest> {
        for priority in [QosPriority::Critical, QosPriority::Interactive, QosPriority::High,
                        QosPriority::Normal, QosPriority::Low, QosPriority::Bulk] {
            if let Some(queue) = self.queues.get_mut(&priority) {
                if let Some(request) = queue.pop_front() {
                    let wait_time = current_time - request.queued_at;
                    self.stats.total_wait_time_ms += wait_time as u64;
                    
                    // Check deadline
                    if let Some(deadline) = request.deadline_ms {
                        if wait_time > deadline as u128 {
                            self.stats.deadline_violations += 1;
                            warn!("QoS deadline violated for {}", request.url);
                        }
                    }
                    
                    self.active_requests.insert(request.id.clone(), request.priority);
                    return Some(request);
                }
            }
        }
        None
    }

    /// Mark request as complete
    pub fn complete(&mut self, request_id: &str) {
        self.active_requests.remove(request_id);
    }

    /// Determine priority for a URL
    pub fn determine_priority(&self, url: &str, user_initiated: bool) -> QosPriority {
        if user_initiated {
            return QosPriority::Interactive;
        }
        
        let lower = url.to_lowercase();
        
        if lower.contains("analytics") || lower.contains("tracking") || lower.contains("beacon") {
            QosPriority::Low
        } else if lower.contains("prefetch") || lower.contains("preload") {
            QosPriority::Low
        } else if lower.ends_with(".js") || lower.ends_with(".css") {
            QosPriority::High
        } else if lower.contains("download") || lower.contains("update") {
            QosPriority::Bulk
        } else {
            QosPriority::Normal
        }
    }

    /// Get QoS report
    pub fn get_report(&self) -> QosReport {
        let pending: HashMap<QosPriority, usize> = self.queues.iter()
            .map(|(p, q)| (*p, q.len()))
            .collect();
        
        QosReport {
            pending_by_priority: pending,
            active_requests: self.active_requests.len(),
            stats: self.stats.clone(),
        }
    }
}

impl Default for QosManager {
    fn default() -> Self {
        Self::new()
    }
}

/// QoS report
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a QosReport.
pub struct QosReport {
    pub pending_by_priority: HashMap<QosPriority, usize>,
    pub active_requests: usize,
    pub stats: QosStats,
}

/// Connection pool manager
#[derive(Debug)]
/// Represents a ConnectionPool.
pub struct ConnectionPool {
    connections: HashMap<String, Vec<ConnectionInfo>>,
    max_per_host: u32,
    total_connections: u32,
    max_total: u32,
}

/// Connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a ConnectionInfo.
pub struct ConnectionInfo {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub protocol: String,
    pub created_at: u128,
    pub last_used: u128,
    pub request_count: u32,
    pub is_active: bool,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub fn new(max_per_host: u32, max_total: u32) -> Self {
        Self {
            connections: HashMap::new(),
            max_per_host,
            total_connections: 0,
            max_total,
        }
    }

    /// Get or create connection
    pub fn get_connection(&mut self, host: &str, port: u16, current_time: u128) -> Option<ConnectionInfo> {
        // Determine protocol from port
        let protocol = if port == 443 { "https" } else { "http" }.to_string();
        
        let conns = self.connections.entry(host.to_string()).or_insert_with(Vec::new);
        
        // Find idle connection
        if let Some(conn) = conns.iter_mut().find(|c| !c.is_active) {
            conn.is_active = true;
            conn.last_used = current_time;
            conn.request_count += 1;
            return Some(conn.clone());
        }
        
        // Create new if under limit
        if conns.len() < self.max_per_host as usize && self.total_connections < self.max_total {
            let conn = ConnectionInfo {
                id: uuid::Uuid::new_v4().to_string(),
                host: host.to_string(),
                port,
                protocol: protocol.clone(),
                created_at: current_time,
                last_used: current_time,
                request_count: 1,
                is_active: true,
            };
            conns.push(conn.clone());
            self.total_connections += 1;
            return Some(conn);
        }
        
        None
    }

    /// Release connection
    pub fn release(&mut self, host: &str, conn_id: &str) {
        if let Some(conns) = self.connections.get_mut(host) {
            if let Some(conn) = conns.iter_mut().find(|c| c.id == conn_id) {
                conn.is_active = false;
            }
        }
    }

    /// Close idle connections
    pub fn cleanup_idle(&mut self, max_idle_ms: u128, current_time: u128) {
        for (_, conns) in self.connections.iter_mut() {
            conns.retain(|c| {
                let is_idle = !c.is_active && current_time - c.last_used > max_idle_ms;
                if is_idle {
                    self.total_connections = self.total_connections.saturating_sub(1);
                }
                !is_idle
            });
        }
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> ConnectionPoolStats {
        let mut active = 0;
        let mut idle = 0;
        
        for conns in self.connections.values() {
            for conn in conns {
                if conn.is_active {
                    active += 1;
                } else {
                    idle += 1;
                }
            }
        }
        
        ConnectionPoolStats {
            total_connections: self.total_connections,
            active_connections: active,
            idle_connections: idle,
            unique_hosts: self.connections.len(),
            max_per_host: self.max_per_host,
            max_total: self.max_total,
        }
    }
}

/// Connection pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a ConnectionPoolStats.
pub struct ConnectionPoolStats {
    pub total_connections: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub unique_hosts: usize,
    pub max_per_host: u32,
    pub max_total: u32,
}

impl NetworkIntelligence {
    /// Create a new network intelligence manager
    pub fn new() -> Self {
        Self::with_config(NetworkIntelligenceConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: NetworkIntelligenceConfig) -> Self {
        info!("Initializing NetworkIntelligence: traffic_analysis={}, qos={}",
            config.traffic_analysis_enabled, config.qos_enabled);
        
        Self {
            traffic_analyzer: Arc::new(RwLock::new(TrafficAnalyzer::new())),
            bandwidth_manager: Arc::new(RwLock::new(BandwidthManager::new(config.max_bandwidth_bps))),
            qos_manager: Arc::new(RwLock::new(QosManager::new())),
            connection_pool: Arc::new(RwLock::new(ConnectionPool::new(
                config.max_connections_per_host,
                config.max_connections_per_host * 10,
            ))),
            config,
            start_time: Instant::now(),
        }
    }

    /// Record a network request
    pub async fn record_request(&self, record: RequestRecord) {
        if self.config.traffic_analysis_enabled {
            let mut analyzer = self.traffic_analyzer.write().await;
            analyzer.record_request(record);
        }
    }

    /// Get traffic report
    pub async fn get_traffic_report(&self) -> TrafficReport {
        let analyzer = self.traffic_analyzer.read().await;
        analyzer.get_report()
    }

    /// Allocate bandwidth
    pub async fn allocate_bandwidth(&self, id: &str, requested_bps: u64, priority: u8) -> u64 {
        if !self.config.bandwidth_management_enabled {
            return requested_bps;
        }
        
        let mut manager = self.bandwidth_manager.write().await;
        manager.allocate(id, requested_bps, priority)
    }

    /// Get bandwidth report
    pub async fn get_bandwidth_report(&self) -> BandwidthReport {
        let manager = self.bandwidth_manager.read().await;
        manager.get_report()
    }

    /// Enqueue QoS request
    pub async fn enqueue_qos(&self, request: QosRequest) {
        if !self.config.qos_enabled {
            return;
        }
        
        let mut qos = self.qos_manager.write().await;
        qos.enqueue(request);
    }

    /// Get next QoS request
    pub async fn dequeue_qos(&self) -> Option<QosRequest> {
        if !self.config.qos_enabled {
            return None;
        }
        
        let mut qos = self.qos_manager.write().await;
        qos.dequeue(self.start_time.elapsed().as_millis())
    }

    /// Get QoS report
    pub async fn get_qos_report(&self) -> QosReport {
        let qos = self.qos_manager.read().await;
        qos.get_report()
    }

    /// Get connection from pool
    pub async fn get_connection(&self, host: &str, port: u16) -> Option<ConnectionInfo> {
        if !self.config.connection_pooling_enabled {
            return None;
        }
        
        let mut pool = self.connection_pool.write().await;
        pool.get_connection(host, port, self.start_time.elapsed().as_millis())
    }

    /// Get connection pool stats
    pub async fn get_connection_stats(&self) -> ConnectionPoolStats {
        let pool = self.connection_pool.read().await;
        pool.get_stats()
    }

    /// Get comprehensive network report
    pub async fn get_full_report(&self) -> NetworkIntelligenceReport {
        NetworkIntelligenceReport {
            traffic: self.get_traffic_report().await,
            bandwidth: self.get_bandwidth_report().await,
            qos: self.get_qos_report().await,
            connections: self.get_connection_stats().await,
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }

    /// Get configuration
    pub fn get_config(&self) -> &NetworkIntelligenceConfig {
        &self.config
    }
}

/// Full network intelligence report
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a NetworkIntelligenceReport.
pub struct NetworkIntelligenceReport {
    pub traffic: TrafficReport,
    pub bandwidth: BandwidthReport,
    pub qos: QosReport,
    pub connections: ConnectionPoolStats,
    pub uptime_seconds: u64,
}

impl Default for NetworkIntelligence {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract domain from URL
fn extract_domain(url: &str) -> String {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_intelligence_creation() {
        let ni = NetworkIntelligence::new();
        let report = ni.get_full_report().await;
        assert_eq!(report.traffic.total_requests, 0);
    }

    #[tokio::test]
    async fn test_traffic_recording() {
        let ni = NetworkIntelligence::new();
        
        ni.record_request(RequestRecord {
            url: "https://example.com/test".to_string(),
            domain: "example.com".to_string(),
            method: "GET".to_string(),
            status_code: 200,
            request_size_bytes: 100,
            response_size_bytes: 1000,
            latency_ms: 50,
            timestamp: 0,
            content_type: "text/html".to_string(),
            cached: false,
        }).await;
        
        let report = ni.get_traffic_report().await;
        assert_eq!(report.total_requests, 1);
        assert_eq!(report.unique_domains, 1);
    }

    #[tokio::test]
    async fn test_qos_priority() {
        let ni = NetworkIntelligence::new();
        
        let request = QosRequest {
            id: "test".to_string(),
            url: "https://example.com/script.js".to_string(),
            priority: QosPriority::High,
            queued_at: 0,
            deadline_ms: Some(1000),
        };
        
        ni.enqueue_qos(request).await;
        
        let dequeued = ni.dequeue_qos().await;
        assert!(dequeued.is_some());
    }

    #[tokio::test]
    async fn test_connection_pool() {
        let ni = NetworkIntelligence::new();
        
        let conn = ni.get_connection("example.com", 443).await;
        assert!(conn.is_some());
        
        let stats = ni.get_connection_stats().await;
        assert_eq!(stats.total_connections, 1);
    }

    #[test]
    fn test_domain_extraction() {
        assert_eq!(extract_domain("https://example.com/path"), "example.com");
        assert_eq!(extract_domain("http://test.com:8080/api"), "test.com");
    }
}
