//! Common prelude for browser-core crate
//! 
//! This module provides standard imports and utilities for consistent
//! error handling and logging across the crate.

pub use anyhow::{Context, Result, bail, ensure};
pub use thiserror::Error;
pub use tracing::{debug, error, info, trace, warn, instrument};

/// Extension trait for Option types to provide better error messages
pub trait OptionExt<T> {
    /// Converts Option to Result with a descriptive error message
    fn ok_or_err(self, msg: &str) -> Result<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_err(self, msg: &str) -> Result<T> {
        self.ok_or_else(|| anyhow::anyhow!("{}", msg))
    }
}

/// Extension trait for Result types to add context
pub trait ResultExt<T, E> {
    /// Adds context to an error
    fn with_ctx(self, msg: &str) -> Result<T>;
}

impl<T, E: std::error::Error + Send + Sync + 'static> ResultExt<T, E> for std::result::Result<T, E> {
    fn with_ctx(self, msg: &str) -> Result<T> {
        self.map_err(|e| anyhow::anyhow!("{}: {}", msg, e))
    }
}

/// Get current Unix timestamp in seconds
/// Returns 0 if system time is before Unix epoch (should never happen)
pub fn unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Get current Unix timestamp in milliseconds
pub fn unix_timestamp_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

// =============================================================================
// Enhanced Maintainability Utilities
// =============================================================================

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Retry configuration for operations that may fail
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries in milliseconds
    pub initial_delay_ms: u64,
    /// Maximum delay between retries in milliseconds
    pub max_delay_ms: u64,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Whether to add jitter to delays
    pub use_jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
            use_jitter: true,
        }
    }
}

impl RetryConfig {
    /// Create a new retry config with custom settings
    pub fn new(max_attempts: u32, initial_delay_ms: u64) -> Self {
        Self {
            max_attempts,
            initial_delay_ms,
            ..Default::default()
        }
    }

    /// Calculate delay for a specific attempt (0-indexed)
    pub fn delay_for_attempt(&self, attempt: u32) -> u64 {
        let base_delay = (self.initial_delay_ms as f64 * self.backoff_multiplier.powi(attempt as i32)) as u64;
        let capped_delay = base_delay.min(self.max_delay_ms);
        
        if self.use_jitter {
            // Add up to 25% jitter
            let jitter = (capped_delay as f64 * 0.25 * rand::random::<f64>()) as u64;
            capped_delay + jitter
        } else {
            capped_delay
        }
    }
}

/// Execute an async operation with retry logic
pub async fn retry_async<F, Fut, T, E>(
    config: &RetryConfig,
    operation: F,
) -> std::result::Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = std::result::Result<T, E>>,
    E: std::fmt::Display,
{
    let mut last_error = None;
    
    for attempt in 0..config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt + 1 < config.max_attempts {
                    let delay = config.delay_for_attempt(attempt);
                    warn!("Attempt {} failed: {}. Retrying in {}ms...", attempt + 1, e, delay);
                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                }
                last_error = Some(e);
            }
        }
    }
    
    Err(last_error.expect("At least one attempt should have been made"))
}

/// Rate limiter for controlling operation frequency
pub struct RateLimiter {
    /// Maximum operations per time window
    max_ops: u32,
    /// Time window in milliseconds
    window_ms: u64,
    /// Operation timestamps
    timestamps: Arc<RwLock<Vec<u128>>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(max_ops: u32, window_ms: u64) -> Self {
        Self {
            max_ops,
            window_ms,
            timestamps: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Check if an operation is allowed and record it if so
    pub async fn try_acquire(&self) -> bool {
        let now = unix_timestamp_ms();
        let mut timestamps = self.timestamps.write().await;
        
        // Remove old timestamps outside the window
        let cutoff = now.saturating_sub(self.window_ms as u128);
        timestamps.retain(|&ts| ts > cutoff);
        
        if timestamps.len() < self.max_ops as usize {
            timestamps.push(now);
            true
        } else {
            false
        }
    }

    /// Wait until an operation is allowed
    pub async fn acquire(&self) {
        loop {
            if self.try_acquire().await {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }

    /// Get current usage (operations in current window)
    pub async fn current_usage(&self) -> u32 {
        let now = unix_timestamp_ms();
        let timestamps = self.timestamps.read().await;
        let cutoff = now.saturating_sub(self.window_ms as u128);
        timestamps.iter().filter(|&&ts| ts > cutoff).count() as u32
    }
}

/// Circuit breaker for preventing cascade failures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    /// Current state
    state: Arc<RwLock<CircuitState>>,
    /// Failure count
    failure_count: Arc<RwLock<u32>>,
    /// Success count (for half-open state)
    success_count: Arc<RwLock<u32>>,
    /// Failure threshold to open circuit
    failure_threshold: u32,
    /// Success threshold to close circuit from half-open
    success_threshold: u32,
    /// Time to wait before moving from open to half-open (ms)
    reset_timeout_ms: u64,
    /// Last failure timestamp
    last_failure: Arc<RwLock<Option<u128>>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(failure_threshold: u32, success_threshold: u32, reset_timeout_ms: u64) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            success_count: Arc::new(RwLock::new(0)),
            failure_threshold,
            success_threshold,
            reset_timeout_ms,
            last_failure: Arc::new(RwLock::new(None)),
        }
    }

    /// Check if operation should be allowed
    pub async fn should_allow(&self) -> bool {
        let mut state = self.state.write().await;
        
        match *state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if we should transition to half-open
                let last_failure = self.last_failure.read().await;
                if let Some(ts) = *last_failure {
                    let now = unix_timestamp_ms();
                    if now - ts > self.reset_timeout_ms as u128 {
                        *state = CircuitState::HalfOpen;
                        *self.success_count.write().await = 0;
                        info!("Circuit breaker transitioning to half-open");
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful operation
    pub async fn record_success(&self) {
        let mut state = self.state.write().await;
        
        match *state {
            CircuitState::Closed => {
                *self.failure_count.write().await = 0;
            }
            CircuitState::HalfOpen => {
                let mut count = self.success_count.write().await;
                *count += 1;
                if *count >= self.success_threshold {
                    *state = CircuitState::Closed;
                    *self.failure_count.write().await = 0;
                    info!("Circuit breaker closed after {} successes", self.success_threshold);
                }
            }
            CircuitState::Open => {}
        }
    }

    /// Record a failed operation
    pub async fn record_failure(&self) {
        let mut state = self.state.write().await;
        
        match *state {
            CircuitState::Closed => {
                let mut count = self.failure_count.write().await;
                *count += 1;
                if *count >= self.failure_threshold {
                    *state = CircuitState::Open;
                    *self.last_failure.write().await = Some(unix_timestamp_ms());
                    warn!("Circuit breaker opened after {} failures", self.failure_threshold);
                }
            }
            CircuitState::HalfOpen => {
                *state = CircuitState::Open;
                *self.last_failure.write().await = Some(unix_timestamp_ms());
                *self.success_count.write().await = 0;
                warn!("Circuit breaker re-opened from half-open state");
            }
            CircuitState::Open => {}
        }
    }

    /// Get current state
    pub async fn state(&self) -> CircuitState {
        *self.state.read().await
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(5, 3, 30000)
    }
}

/// Validation helpers for common input types
pub mod validators {
    use super::*;

    /// Validate URL format
    pub fn validate_url(url: &str) -> Result<()> {
        if url.is_empty() {
            bail!("URL cannot be empty");
        }
        if !url.starts_with("http://") && !url.starts_with("https://") {
            bail!("URL must start with http:// or https://");
        }
        // Basic URL validation
        if url.contains(' ') {
            bail!("URL cannot contain spaces");
        }
        Ok(())
    }

    /// Validate port number
    pub fn validate_port(port: u16) -> Result<()> {
        if port == 0 {
            bail!("Port cannot be 0");
        }
        Ok(())
    }

    /// Validate IP address format (basic check)
    pub fn validate_ip(ip: &str) -> Result<()> {
        if ip.is_empty() {
            bail!("IP address cannot be empty");
        }
        
        // Check IPv4
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() == 4 {
            for part in parts {
                let num: u8 = part.parse().map_err(|_| anyhow::anyhow!("Invalid IP octet: {}", part))?;
                let _ = num; // Just to validate parsing
            }
            return Ok(());
        }
        
        // Check IPv6 (basic)
        if ip.contains(':') && !ip.contains('.') {
            return Ok(()); // Basic IPv6 check
        }
        
        bail!("Invalid IP address format: {}", ip);
    }

    /// Validate non-empty string
    pub fn validate_non_empty(value: &str, field_name: &str) -> Result<()> {
        if value.trim().is_empty() {
            bail!("{} cannot be empty", field_name);
        }
        Ok(())
    }

    /// Validate string length
    pub fn validate_length(value: &str, min: usize, max: usize, field_name: &str) -> Result<()> {
        let len = value.len();
        if len < min {
            bail!("{} must be at least {} characters (got {})", field_name, min, len);
        }
        if len > max {
            bail!("{} must be at most {} characters (got {})", field_name, max, len);
        }
        Ok(())
    }

    /// Validate value is within range
    pub fn validate_range<T: PartialOrd + std::fmt::Display>(
        value: T, 
        min: T, 
        max: T, 
        field_name: &str
    ) -> Result<()> {
        if value < min || value > max {
            bail!("{} must be between {} and {} (got {})", field_name, min, max, value);
        }
        Ok(())
    }
}

/// Metrics collection utilities
pub struct MetricsCollector {
    counters: Arc<RwLock<HashMap<String, u64>>>,
    gauges: Arc<RwLock<HashMap<String, f64>>>,
    histograms: Arc<RwLock<HashMap<String, Vec<f64>>>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Increment a counter
    pub async fn increment(&self, name: &str) {
        let mut counters = self.counters.write().await;
        *counters.entry(name.to_string()).or_insert(0) += 1;
    }

    /// Increment a counter by a specific amount
    pub async fn increment_by(&self, name: &str, amount: u64) {
        let mut counters = self.counters.write().await;
        *counters.entry(name.to_string()).or_insert(0) += amount;
    }

    /// Set a gauge value
    pub async fn set_gauge(&self, name: &str, value: f64) {
        let mut gauges = self.gauges.write().await;
        gauges.insert(name.to_string(), value);
    }

    /// Record a histogram value
    pub async fn record_histogram(&self, name: &str, value: f64) {
        let mut histograms = self.histograms.write().await;
        histograms.entry(name.to_string()).or_insert_with(Vec::new).push(value);
    }

    /// Get counter value
    pub async fn get_counter(&self, name: &str) -> u64 {
        let counters = self.counters.read().await;
        counters.get(name).copied().unwrap_or(0)
    }

    /// Get gauge value
    pub async fn get_gauge(&self, name: &str) -> Option<f64> {
        let gauges = self.gauges.read().await;
        gauges.get(name).copied()
    }

    /// Get histogram statistics
    pub async fn get_histogram_stats(&self, name: &str) -> Option<HistogramStats> {
        let histograms = self.histograms.read().await;
        histograms.get(name).map(|values| {
            if values.is_empty() {
                return HistogramStats::default();
            }
            
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = sum / count as f64;
            
            let mut sorted = values.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            
            let min = sorted.first().copied().unwrap_or(0.0);
            let max = sorted.last().copied().unwrap_or(0.0);
            let median = sorted[count / 2];
            let p95 = sorted[(count as f64 * 0.95) as usize].min(*sorted.last().unwrap_or(&0.0));
            let p99 = sorted[(count as f64 * 0.99) as usize].min(*sorted.last().unwrap_or(&0.0));
            
            HistogramStats {
                count,
                sum,
                mean,
                min,
                max,
                median,
                p95,
                p99,
            }
        })
    }

    /// Get all metrics as a snapshot
    pub async fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            counters: self.counters.read().await.clone(),
            gauges: self.gauges.read().await.clone(),
            timestamp: unix_timestamp(),
        }
    }

    /// Reset all metrics
    pub async fn reset(&self) {
        self.counters.write().await.clear();
        self.gauges.write().await.clear();
        self.histograms.write().await.clear();
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct HistogramStats {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub median: f64,
    pub p95: f64,
    pub p99: f64,
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
    pub timestamp: u64,
}

/// String utilities
pub mod string_utils {
    /// Truncate string to max length with ellipsis
    pub fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else if max_len <= 3 {
            s.chars().take(max_len).collect()
        } else {
            format!("{}...", s.chars().take(max_len - 3).collect::<String>())
        }
    }

    /// Sanitize string for safe logging (remove sensitive data patterns)
    pub fn sanitize_for_log(s: &str) -> String {
        // Hide potential passwords, tokens, etc.
        let mut result = s.to_string();
        
        // Hide anything that looks like a password
        let password_pattern = regex::Regex::new(r"(?i)(password|passwd|pwd|secret|token|api_key|apikey)[\s:=]+\S+").expect("Valid regex pattern");
        result = password_pattern.replace_all(&result, "$1=***HIDDEN***").to_string();
        
        // Hide bearer tokens
        let bearer_pattern = regex::Regex::new(r"(?i)bearer\s+\S+").expect("Valid bearer regex pattern");
        result = bearer_pattern.replace_all(&result, "Bearer ***HIDDEN***").to_string();
        
        result
    }

    /// Convert camelCase to snake_case
    pub fn camel_to_snake(s: &str) -> String {
        let mut result = String::new();
        for (i, c) in s.chars().enumerate() {
            if c.is_uppercase() {
                if i > 0 {
                    result.push('_');
                }
                result.push(c.to_lowercase().next().expect("Character has lowercase representation"));
            } else {
                result.push(c);
            }
        }
        result
    }

    /// Convert snake_case to camelCase
    pub fn snake_to_camel(s: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = false;
        for c in s.chars() {
            if c == '_' {
                capitalize_next = true;
            } else if capitalize_next {
                result.push(c.to_uppercase().next().expect("Character has uppercase representation"));
                capitalize_next = false;
            } else {
                result.push(c);
            }
        }
        result
    }
}

#[cfg(test)]
mod prelude_tests {
    use super::*;

    #[test]
    fn test_retry_config_delay() {
        let config = RetryConfig {
            initial_delay_ms: 100,
            max_delay_ms: 1000,
            backoff_multiplier: 2.0,
            use_jitter: false,
            ..Default::default()
        };
        
        assert_eq!(config.delay_for_attempt(0), 100);
        assert_eq!(config.delay_for_attempt(1), 200);
        assert_eq!(config.delay_for_attempt(2), 400);
        assert_eq!(config.delay_for_attempt(10), 1000); // Capped at max
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(3, 1000);
        
        assert!(limiter.try_acquire().await);
        assert!(limiter.try_acquire().await);
        assert!(limiter.try_acquire().await);
        assert!(!limiter.try_acquire().await); // Should be rate limited
        
        assert_eq!(limiter.current_usage().await, 3);
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        let cb = CircuitBreaker::new(2, 1, 100);
        
        assert!(cb.should_allow().await);
        cb.record_failure().await;
        assert!(cb.should_allow().await);
        cb.record_failure().await;
        assert!(!cb.should_allow().await); // Circuit open
        
        // Wait for reset timeout
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        assert!(cb.should_allow().await); // Should be half-open now
        
        cb.record_success().await;
        assert_eq!(cb.state().await, CircuitState::Closed);
    }

    #[test]
    fn test_validators() {
        assert!(validators::validate_url("https://example.com").is_ok());
        assert!(validators::validate_url("invalid").is_err());
        
        assert!(validators::validate_port(8080).is_ok());
        assert!(validators::validate_port(0).is_err());
        
        assert!(validators::validate_ip("192.168.1.1").is_ok());
        assert!(validators::validate_ip("invalid").is_err());
    }

    #[tokio::test]
    async fn test_metrics_collector() {
        let metrics = MetricsCollector::new();
        
        metrics.increment("requests").await;
        metrics.increment("requests").await;
        assert_eq!(metrics.get_counter("requests").await, 2);
        
        metrics.set_gauge("temperature", 25.5).await;
        assert_eq!(metrics.get_gauge("temperature").await, Some(25.5));
        
        metrics.record_histogram("latency", 100.0).await;
        metrics.record_histogram("latency", 200.0).await;
        let stats = metrics.get_histogram_stats("latency").await.expect("Get operation should succeed");
        assert_eq!(stats.count, 2);
        assert_eq!(stats.mean, 150.0);
    }

    #[test]
    fn test_string_utils() {
        assert_eq!(string_utils::truncate("hello world", 5), "he...");
        assert_eq!(string_utils::truncate("hi", 5), "hi");
        
        assert_eq!(string_utils::camel_to_snake("camelCase"), "camel_case");
        assert_eq!(string_utils::snake_to_camel("snake_case"), "snakeCase");
    }
}
