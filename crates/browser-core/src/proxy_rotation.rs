use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use serde::{Deserialize, Serialize};

use crate::proxy::FreeProxy;
use crate::free_ip_providers::FreeIpProviderManager;

/// Manages proxy rotation strategies for browser tabs.
pub struct ProxyRotationManager {
    provider_manager: Arc<RwLock<FreeIpProviderManager>>,
    active_proxies: Arc<RwLock<HashMap<String, ProxySession>>>,
    strategy: ProxyRotationStrategy,
    performance_metrics: Arc<RwLock<HashMap<String, ProxyMetrics>>>,
}

#[derive(Clone)]
pub struct ProxySession {
    pub proxy: FreeProxy,
    pub assigned_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
    pub request_count: usize,
    pub tab_id: String,
    pub domain_proxy_map: HashMap<String, String>, // domain -> proxy_id
}

#[derive(Debug, Clone)]
pub enum ProxyRotationStrategy {
    /// Rotate proxy after N requests
    PerRequest(usize),
    /// Rotate proxy after time duration
    PerDuration(Duration),
    /// Rotate proxy per session (never during session)
    PerSession,
    /// Random rotation (probabilistic)
    Random { probability: f64 },
    /// Sticky proxy (same proxy for same target domain)
    Sticky { duration: Duration },
    /// Geographic distribution (rotate within country/region)
    Geographic { country_codes: Vec<String> },
    /// Performance-based (use fastest proxies)
    PerformanceBased,
    /// Round-robin rotation
    RoundRobin,
    /// Domain-based (different proxy per domain)
    DomainBased,
    /// Manual rotation (user-triggered)
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyMetrics {
    pub response_time_ms: f64,
    pub success_rate: f64,
    pub last_success: Option<DateTime<Utc>>,
    pub consecutive_failures: u32,
    pub total_requests: u32,
    pub failed_requests: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySessionStats {
    pub tab_id: String,
    pub current_proxy_ip: String,
    pub proxy_country: String,
    pub assigned_at: DateTime<Utc>,
    pub request_count: usize,
    pub duration_seconds: i64,
}

impl ProxyRotationManager {
    pub fn new(
        provider_manager: Arc<RwLock<FreeIpProviderManager>>,
        strategy: ProxyRotationStrategy,
    ) -> Self {
        Self {
            provider_manager,
            active_proxies: Arc::new(RwLock::new(HashMap::new())),
            strategy,
            performance_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or rotate proxy for tab
    pub async fn get_proxy_for_tab(&self, tab_id: &str, domain: Option<&str>) -> Result<FreeProxy> {
        let mut sessions = self.active_proxies.write().await;

        if let Some(session) = sessions.get_mut(tab_id) {
            // Check if we need domain-based proxy
            if let (Some(domain), ProxyRotationStrategy::DomainBased) = (domain, &self.strategy) {
                let _proxy_key = format!("{}:{}", tab_id, domain);
                if let Some(proxy_id) = session.domain_proxy_map.get(domain) {
                    // Use the same proxy for this domain
                    if let Some(proxy) = self.get_proxy_by_id(proxy_id).await? {
                        session.last_used = Utc::now();
                        session.request_count += 1;
                        return Ok(proxy);
                    }
                }
                // Get a new proxy for this domain
                let new_proxy = self.get_next_proxy(session).await?;
                session.domain_proxy_map.insert(domain.to_string(), new_proxy.ip.clone());
                session.last_used = Utc::now();
                session.request_count += 1;
                return Ok(new_proxy);
            }

            if self.should_rotate(session).await {
                let new_proxy = self.get_next_proxy(session).await?;
                session.proxy = new_proxy.clone();
                session.assigned_at = Utc::now();
                session.last_used = Utc::now();
                session.request_count = 1;
                
                // Clear domain map when rotating
                if matches!(self.strategy, ProxyRotationStrategy::DomainBased) {
                    session.domain_proxy_map.clear();
                }
                
                debug!("Rotated proxy for tab {}: {} -> {}", tab_id, session.proxy.ip, new_proxy.ip);
                return Ok(new_proxy);
            } else {
                session.last_used = Utc::now();
                session.request_count += 1;
                return Ok(session.proxy.clone());
            }
        }

        // Create new session with new proxy
        let proxy = self.get_initial_proxy().await?;
        let session = ProxySession {
            proxy: proxy.clone(),
            assigned_at: Utc::now(),
            last_used: Utc::now(),
            request_count: 1,
            tab_id: tab_id.to_string(),
            domain_proxy_map: HashMap::new(),
        };

        sessions.insert(tab_id.to_string(), session);
        info!("Assigned initial proxy {} to tab {}", proxy.ip, tab_id);
        Ok(proxy)
    }

    /// Manually rotate proxy for tab
    pub async fn force_rotate(&self, tab_id: &str) -> Result<FreeProxy> {
        let mut sessions = self.active_proxies.write().await;
        if let Some(session) = sessions.get_mut(tab_id) {
            let new_proxy = self.get_next_proxy(session).await?;
            session.proxy = new_proxy.clone();
            session.assigned_at = Utc::now();
            session.last_used = Utc::now();
            session.request_count = 0;
            session.domain_proxy_map.clear();
            
            info!("Force rotated proxy for tab {}: {} -> {}", tab_id, session.proxy.ip, new_proxy.ip);
            Ok(new_proxy)
        } else {
            Err(anyhow!("Tab session not found"))
        }
    }

    /// Record proxy performance metrics
    pub async fn record_performance(&self, proxy_id: &str, success: bool, response_time_ms: Option<f64>) {
        let mut metrics = self.performance_metrics.write().await;
        let metric = metrics.entry(proxy_id.to_string()).or_insert(ProxyMetrics {
            response_time_ms: 0.0,
            success_rate: 0.0,
            last_success: None,
            consecutive_failures: 0,
            total_requests: 0,
            failed_requests: 0,
        });

        metric.total_requests += 1;
        if success {
            metric.failed_requests = 0;
            metric.consecutive_failures = 0;
            metric.last_success = Some(Utc::now());
            if let Some(rt) = response_time_ms {
                metric.response_time_ms = (metric.response_time_ms * 0.9) + (rt * 0.1); // EMA
            }
        } else {
            metric.failed_requests += 1;
            metric.consecutive_failures += 1;
        }

        metric.success_rate = (metric.total_requests - metric.failed_requests) as f64 / metric.total_requests as f64;
    }

    /// Get current proxy for tab
    pub async fn get_current_proxy(&self, tab_id: &str) -> Option<FreeProxy> {
        let sessions = self.active_proxies.read().await;
        sessions.get(tab_id).map(|s| s.proxy.clone())
    }

    /// Get session statistics
    pub async fn get_session_stats(&self, tab_id: &str) -> Option<ProxySessionStats> {
        let sessions = self.active_proxies.read().await;
        sessions.get(tab_id).map(|s| ProxySessionStats {
            tab_id: s.tab_id.clone(),
            current_proxy_ip: s.proxy.ip.clone(),
            proxy_country: s.proxy.country.clone(),
            assigned_at: s.assigned_at,
            request_count: s.request_count,
            duration_seconds: (Utc::now() - s.assigned_at).num_seconds(),
        })
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired(&self, max_age: Duration) {
        let mut sessions = self.active_proxies.write().await;
        let now = Utc::now();
        let initial_count = sessions.len();
        sessions.retain(|_, session| now - session.last_used < max_age);
        let removed = initial_count - sessions.len();
        
        if removed > 0 {
            info!("Cleaned up {} expired proxy sessions", removed);
        }
    }

    /// Update rotation strategy
    pub async fn update_strategy(&mut self, strategy: ProxyRotationStrategy) {
        info!("Updating proxy rotation strategy to {:?}", strategy);
        self.strategy = strategy;
    }

    async fn should_rotate(&self, session: &ProxySession) -> bool {
        match &self.strategy {
            ProxyRotationStrategy::PerRequest(count) => session.request_count >= *count,
            ProxyRotationStrategy::PerDuration(duration) => {
                let elapsed = Utc::now() - session.assigned_at;
                elapsed > *duration
            }
            ProxyRotationStrategy::PerSession => false,
            ProxyRotationStrategy::Random { probability } => rand::thread_rng().gen::<f64>() < *probability,
            ProxyRotationStrategy::Sticky { duration } => {
                let elapsed = Utc::now() - session.last_used;
                elapsed > *duration
            }
            ProxyRotationStrategy::Geographic { .. } => false, // Handled in get_next_proxy
            ProxyRotationStrategy::PerformanceBased => {
                // Check if current proxy is underperforming
                if let Some(metric) = self.performance_metrics.read().await.get(&session.proxy.ip) {
                    metric.success_rate < 0.8 || metric.consecutive_failures > 3
                } else {
                    false
                }
            }
            ProxyRotationStrategy::RoundRobin => session.request_count >= 100, // Rotate every 100 requests
            ProxyRotationStrategy::DomainBased => false, // Handled per domain
            ProxyRotationStrategy::Manual => false,
        }
    }

    async fn get_initial_proxy(&self) -> Result<FreeProxy> {
        let provider = self.provider_manager.read().await;
        
        match &self.strategy {
            ProxyRotationStrategy::Geographic { country_codes } => {
                if country_codes.is_empty() {
                    return provider.get_random_working_proxy()
                        .ok_or_else(|| anyhow!("No working proxies available"))
                        .map(|p| p.clone());
                }
                
                let working_proxies = provider.get_working_proxies();
                let country_proxies: Vec<_> = working_proxies
                    .iter()
                    .filter(|p| country_codes.contains(&p.country) || country_codes.contains(&p.country_code))
                    .collect();
                
                if country_proxies.is_empty() {
                    warn!("No proxies found for specified countries, using random working proxy");
                    provider.get_random_working_proxy()
                        .ok_or_else(|| anyhow!("No working proxies available"))
                        .map(|p| p.clone())
                } else {
                    let mut rng = rand::thread_rng();
                    Ok((*country_proxies[rng.gen_range(0..country_proxies.len())]).clone())
                }
            }
            ProxyRotationStrategy::PerformanceBased => {
                let metrics = self.performance_metrics.read().await;
                let working_proxies = provider.get_working_proxies();
                
                if working_proxies.is_empty() {
                    return Err(anyhow!("No working proxies available"));
                }
                
                // Sort by success rate and response time
                let mut sorted_proxies: Vec<_> = working_proxies.iter().collect();
                sorted_proxies.sort_by(|a, b| {
                    let metric_a = metrics.get(&a.ip);
                    let metric_b = metrics.get(&b.ip);
                    
                    match (metric_a, metric_b) {
                        (Some(ma), Some(mb)) => {
                            mb.success_rate.partial_cmp(&ma.success_rate)
                                .unwrap_or(std::cmp::Ordering::Equal)
                                .then_with(|| ma.response_time_ms.partial_cmp(&mb.response_time_ms).unwrap_or(std::cmp::Ordering::Equal))
                        }
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                });
                
                Ok((*sorted_proxies[0]).clone())
            }
            _ => {
                provider.get_random_working_proxy()
                    .ok_or_else(|| anyhow!("No working proxies available"))
                    .map(|p| p.clone())
            }
        }
    }

    async fn get_next_proxy(&self, _current_session: &ProxySession) -> Result<FreeProxy> {
        self.get_initial_proxy().await
    }

    async fn get_proxy_by_id(&self, proxy_id: &str) -> Result<Option<FreeProxy>> {
        let provider = self.provider_manager.read().await;
        Ok(provider.get_proxy_pool()
            .iter()
            .find(|p| p.ip == proxy_id && p.is_working)
            .cloned())
    }
}

// =============================================================================
// Enhanced Proxy Selection and Health Monitoring
// =============================================================================

/// Smart proxy selector with weighted scoring algorithm
#[derive(Debug, Clone)]
pub struct SmartProxySelector {
    /// Weight for success rate (0.0 - 1.0)
    pub success_rate_weight: f64,
    /// Weight for response time (0.0 - 1.0)
    pub response_time_weight: f64,
    /// Weight for geographic preference (0.0 - 1.0)
    pub geo_weight: f64,
    /// Weight for uptime (0.0 - 1.0)
    pub uptime_weight: f64,
    /// Weight for anonymity level (0.0 - 1.0)
    pub anonymity_weight: f64,
    /// Preferred countries for geographic scoring
    pub preferred_countries: Vec<String>,
    /// Maximum acceptable response time in ms
    pub max_response_time_ms: f64,
    /// Minimum acceptable success rate
    pub min_success_rate: f64,
}

impl Default for SmartProxySelector {
    fn default() -> Self {
        Self {
            success_rate_weight: 0.35,
            response_time_weight: 0.25,
            geo_weight: 0.15,
            uptime_weight: 0.15,
            anonymity_weight: 0.10,
            preferred_countries: vec![],
            max_response_time_ms: 5000.0,
            min_success_rate: 0.7,
        }
    }
}

impl SmartProxySelector {
    /// Create a new smart proxy selector with custom weights
    pub fn new(
        success_rate_weight: f64,
        response_time_weight: f64,
        geo_weight: f64,
        uptime_weight: f64,
        anonymity_weight: f64,
    ) -> Self {
        Self {
            success_rate_weight,
            response_time_weight,
            geo_weight,
            uptime_weight,
            anonymity_weight,
            ..Default::default()
        }
    }

    /// Calculate composite score for a proxy
    pub fn calculate_score(&self, proxy: &FreeProxy, metrics: Option<&ProxyMetrics>) -> f64 {
        let mut score = 0.0;
        
        // Success rate score (0-1)
        let success_score = metrics
            .map(|m| m.success_rate)
            .unwrap_or(0.5); // Default to 0.5 for unknown proxies
        score += success_score * self.success_rate_weight;
        
        // Response time score (inverse, normalized to 0-1)
        let response_score = metrics
            .map(|m| {
                if m.response_time_ms <= 0.0 {
                    0.5
                } else {
                    (1.0 - (m.response_time_ms / self.max_response_time_ms)).max(0.0)
                }
            })
            .unwrap_or(0.5);
        score += response_score * self.response_time_weight;
        
        // Geographic score
        let geo_score = if self.preferred_countries.is_empty() {
            1.0
        } else if self.preferred_countries.contains(&proxy.country) 
            || self.preferred_countries.contains(&proxy.country_code) {
            1.0
        } else {
            0.3
        };
        score += geo_score * self.geo_weight;
        
        // Uptime score (normalized from percentage)
        let uptime_score = (proxy.uptime / 100.0) as f64;
        score += uptime_score * self.uptime_weight;
        
        // Anonymity score
        let anonymity_score = match proxy.anonymity.to_lowercase().as_str() {
            "elite" | "high" => 1.0,
            "anonymous" | "medium" => 0.7,
            "transparent" | "low" => 0.3,
            _ => 0.5,
        };
        score += anonymity_score * self.anonymity_weight;
        
        score
    }

    /// Select the best proxy from a list based on weighted scoring
    pub fn select_best(&self, proxies: &[FreeProxy], metrics: &HashMap<String, ProxyMetrics>) -> Option<FreeProxy> {
        if proxies.is_empty() {
            return None;
        }
        
        let mut best_proxy: Option<&FreeProxy> = None;
        let mut best_score = f64::NEG_INFINITY;
        
        for proxy in proxies {
            if !proxy.is_working {
                continue;
            }
            
            // Skip proxies below minimum success rate
            if let Some(metric) = metrics.get(&proxy.ip) {
                if metric.success_rate < self.min_success_rate && metric.total_requests > 5 {
                    continue;
                }
            }
            
            let score = self.calculate_score(proxy, metrics.get(&proxy.ip));
            if score > best_score {
                best_score = score;
                best_proxy = Some(proxy);
            }
        }
        
        best_proxy.cloned()
    }

    /// Select top N proxies based on scoring
    pub fn select_top_n(&self, proxies: &[FreeProxy], metrics: &HashMap<String, ProxyMetrics>, n: usize) -> Vec<FreeProxy> {
        let mut scored_proxies: Vec<(f64, &FreeProxy)> = proxies
            .iter()
            .filter(|p| p.is_working)
            .map(|p| (self.calculate_score(p, metrics.get(&p.ip)), p))
            .collect();
        
        scored_proxies.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        
        scored_proxies
            .into_iter()
            .take(n)
            .map(|(_, p)| p.clone())
            .collect()
    }
}

/// Proxy health monitor for automatic failover and health tracking
pub struct ProxyHealthMonitor {
    /// Health check interval in seconds
    pub check_interval_secs: u64,
    /// Maximum consecutive failures before marking unhealthy
    pub max_failures: u32,
    /// Recovery check interval for unhealthy proxies
    pub recovery_interval_secs: u64,
    /// Health status for each proxy
    health_status: Arc<RwLock<HashMap<String, ProxyHealthStatus>>>,
    /// Bandwidth tracking
    bandwidth_tracker: Arc<RwLock<HashMap<String, BandwidthStats>>>,
}

/// Health status for a proxy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyHealthStatus {
    pub proxy_id: String,
    pub is_healthy: bool,
    pub last_check: DateTime<Utc>,
    pub consecutive_failures: u32,
    pub last_error: Option<String>,
    pub average_latency_ms: f64,
    pub health_score: f64,
}

/// Bandwidth statistics for a proxy
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BandwidthStats {
    pub proxy_id: String,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub requests_count: u64,
    pub start_time: Option<DateTime<Utc>>,
    pub last_updated: Option<DateTime<Utc>>,
}

impl BandwidthStats {
    /// Calculate average bandwidth in bytes per second
    pub fn average_bandwidth_bps(&self) -> f64 {
        if let (Some(start), Some(end)) = (self.start_time, self.last_updated) {
            let duration_secs = (end - start).num_seconds() as f64;
            if duration_secs > 0.0 {
                return (self.bytes_sent + self.bytes_received) as f64 / duration_secs;
            }
        }
        0.0
    }
    
    /// Get total bandwidth used
    pub fn total_bytes(&self) -> u64 {
        self.bytes_sent + self.bytes_received
    }
}

impl ProxyHealthMonitor {
    pub fn new() -> Self {
        Self {
            check_interval_secs: 60,
            max_failures: 3,
            recovery_interval_secs: 300,
            health_status: Arc::new(RwLock::new(HashMap::new())),
            bandwidth_tracker: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a successful request
    pub async fn record_success(&self, proxy_id: &str, latency_ms: f64, bytes_sent: u64, bytes_received: u64) {
        // Update health status
        let mut status = self.health_status.write().await;
        let health = status.entry(proxy_id.to_string()).or_insert(ProxyHealthStatus {
            proxy_id: proxy_id.to_string(),
            is_healthy: true,
            last_check: Utc::now(),
            consecutive_failures: 0,
            last_error: None,
            average_latency_ms: latency_ms,
            health_score: 1.0,
        });
        
        health.is_healthy = true;
        health.last_check = Utc::now();
        health.consecutive_failures = 0;
        health.last_error = None;
        // Exponential moving average for latency
        health.average_latency_ms = health.average_latency_ms * 0.8 + latency_ms * 0.2;
        health.health_score = self.calculate_health_score(health);
        
        // Update bandwidth stats
        let mut bandwidth = self.bandwidth_tracker.write().await;
        let stats = bandwidth.entry(proxy_id.to_string()).or_insert(BandwidthStats {
            proxy_id: proxy_id.to_string(),
            bytes_sent: 0,
            bytes_received: 0,
            requests_count: 0,
            start_time: Some(Utc::now()),
            last_updated: None,
        });
        
        stats.bytes_sent += bytes_sent;
        stats.bytes_received += bytes_received;
        stats.requests_count += 1;
        stats.last_updated = Some(Utc::now());
    }

    /// Record a failed request
    pub async fn record_failure(&self, proxy_id: &str, error: &str) {
        let mut status = self.health_status.write().await;
        let health = status.entry(proxy_id.to_string()).or_insert(ProxyHealthStatus {
            proxy_id: proxy_id.to_string(),
            is_healthy: true,
            last_check: Utc::now(),
            consecutive_failures: 0,
            last_error: None,
            average_latency_ms: 0.0,
            health_score: 1.0,
        });
        
        health.last_check = Utc::now();
        health.consecutive_failures += 1;
        health.last_error = Some(error.to_string());
        
        if health.consecutive_failures >= self.max_failures {
            health.is_healthy = false;
            warn!("Proxy {} marked as unhealthy after {} consecutive failures", proxy_id, health.consecutive_failures);
        }
        
        health.health_score = self.calculate_health_score(health);
    }

    /// Calculate health score (0.0 - 1.0)
    fn calculate_health_score(&self, status: &ProxyHealthStatus) -> f64 {
        if !status.is_healthy {
            return 0.0;
        }
        
        let failure_penalty = (status.consecutive_failures as f64 * 0.2).min(0.6);
        let latency_score = if status.average_latency_ms > 0.0 {
            (1.0 - (status.average_latency_ms / 10000.0)).max(0.1)
        } else {
            0.5
        };
        
        (1.0 - failure_penalty) * latency_score
    }

    /// Get health status for a proxy
    pub async fn get_health(&self, proxy_id: &str) -> Option<ProxyHealthStatus> {
        let status = self.health_status.read().await;
        status.get(proxy_id).cloned()
    }

    /// Get all healthy proxies
    pub async fn get_healthy_proxies(&self) -> Vec<String> {
        let status = self.health_status.read().await;
        status
            .iter()
            .filter(|(_, h)| h.is_healthy)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get bandwidth stats for a proxy
    pub async fn get_bandwidth_stats(&self, proxy_id: &str) -> Option<BandwidthStats> {
        let bandwidth = self.bandwidth_tracker.read().await;
        bandwidth.get(proxy_id).cloned()
    }

    /// Get total bandwidth across all proxies
    pub async fn get_total_bandwidth(&self) -> BandwidthStats {
        let bandwidth = self.bandwidth_tracker.read().await;
        let mut total = BandwidthStats::default();
        
        for stats in bandwidth.values() {
            total.bytes_sent += stats.bytes_sent;
            total.bytes_received += stats.bytes_received;
            total.requests_count += stats.requests_count;
        }
        
        total
    }

    /// Reset bandwidth stats for a proxy
    pub async fn reset_bandwidth_stats(&self, proxy_id: &str) {
        let mut bandwidth = self.bandwidth_tracker.write().await;
        if let Some(stats) = bandwidth.get_mut(proxy_id) {
            stats.bytes_sent = 0;
            stats.bytes_received = 0;
            stats.requests_count = 0;
            stats.start_time = Some(Utc::now());
            stats.last_updated = None;
        }
    }
}

impl Default for ProxyHealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Geographic diversity manager for proxy distribution
#[derive(Debug, Clone)]
pub struct GeoDiversityManager {
    /// Country usage counts
    country_usage: HashMap<String, u32>,
    /// Maximum usage per country before diversifying
    max_per_country: u32,
    /// Enable automatic diversification
    auto_diversify: bool,
}

impl GeoDiversityManager {
    pub fn new(max_per_country: u32) -> Self {
        Self {
            country_usage: HashMap::new(),
            max_per_country,
            auto_diversify: true,
        }
    }

    /// Record usage of a proxy from a country
    pub fn record_usage(&mut self, country_code: &str) {
        *self.country_usage.entry(country_code.to_string()).or_insert(0) += 1;
    }

    /// Check if a country should be avoided due to overuse
    pub fn should_avoid_country(&self, country_code: &str) -> bool {
        if !self.auto_diversify {
            return false;
        }
        self.country_usage.get(country_code).copied().unwrap_or(0) >= self.max_per_country
    }

    /// Get least used countries
    pub fn get_least_used_countries(&self, count: usize) -> Vec<String> {
        let mut sorted: Vec<_> = self.country_usage.iter().collect();
        sorted.sort_by_key(|(_, usage)| *usage);
        sorted.into_iter().take(count).map(|(c, _)| c.clone()).collect()
    }

    /// Filter proxies to favor geographic diversity
    pub fn filter_for_diversity(&self, proxies: &[FreeProxy]) -> Vec<FreeProxy> {
        proxies
            .iter()
            .filter(|p| !self.should_avoid_country(&p.country_code))
            .cloned()
            .collect()
    }

    /// Reset usage statistics
    pub fn reset(&mut self) {
        self.country_usage.clear();
    }

    /// Get diversity score (0.0 - 1.0, higher is more diverse)
    pub fn diversity_score(&self) -> f64 {
        if self.country_usage.is_empty() {
            return 1.0;
        }
        
        let total: u32 = self.country_usage.values().sum();
        let countries = self.country_usage.len() as f64;
        let ideal_per_country = total as f64 / countries;
        
        let variance: f64 = self.country_usage
            .values()
            .map(|&v| (v as f64 - ideal_per_country).powi(2))
            .sum::<f64>() / countries;
        
        let std_dev = variance.sqrt();
        let normalized = 1.0 - (std_dev / ideal_per_country).min(1.0);
        
        normalized
    }
}

impl Default for GeoDiversityManager {
    fn default() -> Self {
        Self::new(10)
    }
}

#[cfg(test)]
mod enhanced_tests {
    use super::*;

    #[test]
    fn test_smart_proxy_selector_scoring() {
        let selector = SmartProxySelector::default();
        let proxy = FreeProxy {
            ip: "1.2.3.4".to_string(),
            port: 8080,
            protocol: crate::proxy::ProxyType::Http,
            country: "US".to_string(),
            country_code: "US".to_string(),
            anonymity: "elite".to_string(),
            speed: 100,
            uptime: 95.0,
            last_checked: "2024-01-01".to_string(),
            provider: "test".to_string(),
            is_working: true,
        };
        
        let metrics = ProxyMetrics {
            response_time_ms: 200.0,
            success_rate: 0.95,
            last_success: Some(Utc::now()),
            consecutive_failures: 0,
            total_requests: 100,
            failed_requests: 5,
        };
        
        let score = selector.calculate_score(&proxy, Some(&metrics));
        assert!(score > 0.5, "Score should be high for good proxy");
    }

    #[test]
    fn test_geo_diversity_manager() {
        let mut manager = GeoDiversityManager::new(5);
        
        for _ in 0..5 {
            manager.record_usage("US");
        }
        
        assert!(manager.should_avoid_country("US"));
        assert!(!manager.should_avoid_country("DE"));
    }

    #[tokio::test]
    async fn test_health_monitor() {
        let monitor = ProxyHealthMonitor::new();
        
        monitor.record_success("proxy1", 100.0, 1000, 2000).await;
        let health = monitor.get_health("proxy1").await.unwrap();
        assert!(health.is_healthy);
        
        for _ in 0..3 {
            monitor.record_failure("proxy2", "connection timeout").await;
        }
        let health2 = monitor.get_health("proxy2").await.unwrap();
        assert!(!health2.is_healthy);
    }

    #[tokio::test]
    async fn test_bandwidth_tracking() {
        let monitor = ProxyHealthMonitor::new();
        
        monitor.record_success("proxy1", 100.0, 1000, 2000).await;
        monitor.record_success("proxy1", 150.0, 500, 1500).await;
        
        let stats = monitor.get_bandwidth_stats("proxy1").await.unwrap();
        assert_eq!(stats.bytes_sent, 1500);
        assert_eq!(stats.bytes_received, 3500);
        assert_eq!(stats.requests_count, 2);
    }
}
