use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{debug, info, warn, error};
use std::sync::Arc;

use crate::proxy::{FreeProxy, ProxySettings};
use crate::http_client::HttpClient;

// Internal struct for test results
#[derive(Debug, Clone)]
struct InternalTestResult {
    is_working: bool,
    detected_ip: Option<String>,
    detected_country: Option<String>,
    supports_https: bool,
    has_ip_leak: bool,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_working: bool,
    pub response_time_ms: u64,
    pub detected_country: Option<String>,
    pub detected_ip: Option<String>,
    pub supports_https: bool,
    pub has_ip_leak: bool,
    pub error: Option<String>,
    pub validated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ProxyValidatorConfig {
    pub timeout: Duration,
    pub concurrent_checks: usize,
    pub test_urls: Vec<String>,
    pub max_retries: u32,
}

impl Default for ProxyValidatorConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            concurrent_checks: 20,
            test_urls: vec![
                "https://api.ipify.org?format=json".to_string(),
                "https://httpbin.org/ip".to_string(),
                "https://jsonip.com".to_string(),
            ],
            max_retries: 3,
        }
    }
}

pub struct ProxyValidator {
    config: ProxyValidatorConfig,
    semaphore: Arc<Semaphore>,
}

impl ProxyValidator {
    pub fn new(config: ProxyValidatorConfig) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(config.concurrent_checks)),
            config,
        }
    }

    pub async fn validate_proxy(&self, proxy: &FreeProxy) -> Result<ValidationResult> {
        let _permit = self.semaphore.acquire().await
            .map_err(|e| anyhow!("Failed to acquire semaphore: {}", e))?;

        let settings = proxy.to_proxy_settings();
        
        info!("Validating proxy {}:{}", proxy.ip, proxy.port);
        
        let mut last_error = None;
        let mut best_result = None;
        
        for attempt in 0..self.config.max_retries {
            if attempt > 0 {
                debug!("Retry attempt {} for proxy {}:{}", attempt + 1, proxy.ip, proxy.port);
            }
            
            match self.validate_single_attempt(&settings, &proxy).await {
                Ok(result) => {
                    if result.is_working {
                        best_result = Some(result);
                        break;
                    } else {
                        last_error = result.error.clone();
                    }
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                }
            }
            
            // Wait before retry
            if attempt < self.config.max_retries - 1 {
                tokio::time::sleep(Duration::from_millis(500 * (attempt + 1) as u64)).await;
            }
        }
        
        let result = best_result.unwrap_or_else(|| ValidationResult {
            is_working: false,
            response_time_ms: u64::MAX,
            detected_country: None,
            detected_ip: None,
            supports_https: false,
            has_ip_leak: false,
            error: last_error,
            validated_at: Utc::now(),
        });
        
        if result.is_working {
            info!("Proxy {}:{} is working ({}ms)", proxy.ip, proxy.port, result.response_time_ms);
        } else {
            warn!("Proxy {}:{} failed validation: {:?}", proxy.ip, proxy.port, result.error);
        }
        
        Ok(result)
    }

    async fn validate_single_attempt(&self, settings: &ProxySettings, proxy: &FreeProxy) -> Result<ValidationResult> {
        let start = std::time::Instant::now();
        
        // Create HTTP client with proxy
        let client = HttpClient::with_proxy(settings)?;
        
        // Test basic connectivity
        let test_result = self.test_connectivity(&client, proxy).await?;
        
        let elapsed = start.elapsed();
        
        Ok(ValidationResult {
            is_working: test_result.is_working,
            response_time_ms: elapsed.as_millis() as u64,
            detected_country: test_result.detected_country,
            detected_ip: test_result.detected_ip,
            supports_https: test_result.supports_https,
            has_ip_leak: test_result.has_ip_leak,
            error: test_result.error,
            validated_at: Utc::now(),
        })
    }

    async fn test_connectivity(&self, client: &HttpClient, proxy: &FreeProxy) -> Result<InternalTestResult> {
        // Try each test URL until one succeeds
        for url in &self.config.test_urls {
            match self.test_url(client, url).await {
                Ok(result) => {
                    if result.is_working {
                        return Ok(result);
                    }
                }
                Err(e) => {
                    debug!("Failed to test {} with proxy {}: {}: {}", url, proxy.ip, proxy.port, e);
                }
            }
        }
        
        // All URLs failed
        Ok(InternalTestResult {
            is_working: false,
            detected_ip: None,
            detected_country: None,
            supports_https: false,
            has_ip_leak: false,
            error: Some("All test URLs failed".to_string()),
        })
    }

    async fn test_url(&self, client: &HttpClient, url: &str) -> Result<InternalTestResult> {
        // Make request through proxy
        let response = client.get(url).await?;
        
        // Parse response to detect IP
        #[derive(Deserialize)]
        struct IpResponse {
            ip: String,
            country: Option<String>,
        }
        
        let ip_response: IpResponse = serde_json::from_str(&response)
            .map_err(|e| anyhow!("Failed to parse response: {}", e))?;
        
        // Check if IP is different from local IP (simple leak detection)
        let has_ip_leak = self.check_ip_leak(&ip_response.ip).await.unwrap_or(false);
        
        Ok(InternalTestResult {
            is_working: true,
            detected_ip: Some(ip_response.ip.clone()),
            detected_country: ip_response.country,
            supports_https: url.starts_with("https://"),
            has_ip_leak,
            error: None,
        })
    }

    async fn check_ip_leak(&self, detected_ip: &str) -> Result<bool> {
        // Get local IP without proxy
        let local_client = HttpClient::new()?;
        let local_response = local_client.get("https://api.ipify.org?format=json").await?;
        
        #[derive(Deserialize)]
        struct LocalIpResponse {
            ip: String,
        }
        
        let local_ip: LocalIpResponse = serde_json::from_str(&local_response)?;
        
        Ok(local_ip.ip == detected_ip)
    }

    pub async fn validate_batch(&self, proxies: &[FreeProxy]) -> Vec<(FreeProxy, ValidationResult)> {
        let mut results = Vec::new();
        
        // Process proxies in parallel with semaphore limiting
        let mut tasks = Vec::new();
        
        for proxy in proxies {
            let proxy = proxy.clone();
            let validator = ProxyValidator::new(self.config.clone());
            
            let task = tokio::spawn(async move {
                let result = validator.validate_proxy(&proxy).await;
                (proxy, result)
            });
            
            tasks.push(task);
        }
        
        // Wait for all tasks to complete
        for task in tasks {
            match task.await {
                Ok((proxy, Ok(result))) => {
                    results.push((proxy, result));
                }
                Ok((proxy, Err(e))) => {
                    error!("Failed to validate proxy: {}", e);
                    results.push((proxy, ValidationResult {
                        is_working: false,
                        response_time_ms: u64::MAX,
                        detected_country: None,
                        detected_ip: None,
                        supports_https: false,
                        has_ip_leak: false,
                        error: Some(e.to_string()),
                        validated_at: Utc::now(),
                    }));
                }
                Err(e) => {
                    error!("Task failed: {}", e);
                }
            }
        }
        
        results
    }
}

#[allow(dead_code)]
pub struct ProxyHealthChecker {
    validator: ProxyValidator,
    check_interval: Duration,
    max_consecutive_failures: u32,
    quarantine_duration: Duration,
}

impl ProxyHealthChecker {
    pub fn new(
        validator: ProxyValidator,
        check_interval: Duration,
        max_consecutive_failures: u32,
        quarantine_duration: Duration,
    ) -> Self {
        Self {
            validator,
            check_interval,
            max_consecutive_failures,
            quarantine_duration,
        }
    }

    pub async fn start_health_monitoring(&self, proxies: Arc<tokio::sync::RwLock<Vec<FreeProxy>>>) {
        let mut interval = tokio::time::interval(self.check_interval);
        
        loop {
            interval.tick().await;
            
            let proxies_to_check = {
                let proxies = proxies.read().await;
                proxies.clone()
            };
            
            if proxies_to_check.is_empty() {
                continue;
            }
            
            info!("Starting health check for {} proxies", proxies_to_check.len());
            
            let results = self.validator.validate_batch(&proxies_to_check).await;
            
            // Update proxy status based on results
            let mut proxies = proxies.write().await;
            for (proxy, result) in results {
                if let Some(p) = proxies.iter_mut().find(|p| p.ip == proxy.ip && p.port == proxy.port) {
                    p.is_working = result.is_working;
                    p.last_checked = Utc::now().to_rfc3339();
                    
                    if !result.is_working {
                        warn!("Proxy {}:{} marked as unhealthy: {:?}", proxy.ip, proxy.port, result.error);
                    }
                }
            }
            
            let working_count = proxies.iter().filter(|p| p.is_working).count();
            info!("Health check completed: {}/{} proxies working", working_count, proxies.len());
        }
    }
}

// ============================================================================
// Quarantine System for Failed Proxies
// ============================================================================

use std::collections::HashMap;
use tokio::sync::RwLock;

/// Represents a quarantined proxy with failure tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantinedProxy {
    pub proxy: FreeProxy,
    pub consecutive_failures: u32,
    pub quarantined_at: DateTime<Utc>,
    pub release_at: DateTime<Utc>,
    pub failure_reasons: Vec<String>,
}

/// Manages quarantined proxies and their release
pub struct ProxyQuarantineManager {
    quarantined: Arc<RwLock<HashMap<String, QuarantinedProxy>>>,
    max_consecutive_failures: u32,
    quarantine_duration: Duration,
    max_quarantine_duration: Duration,
}

impl ProxyQuarantineManager {
    pub fn new(
        max_consecutive_failures: u32,
        quarantine_duration: Duration,
        max_quarantine_duration: Duration,
    ) -> Self {
        Self {
            quarantined: Arc::new(RwLock::new(HashMap::new())),
            max_consecutive_failures,
            quarantine_duration,
            max_quarantine_duration,
        }
    }

    /// Generate a unique key for a proxy
    fn proxy_key(proxy: &FreeProxy) -> String {
        format!("{}:{}", proxy.ip, proxy.port)
    }

    /// Record a failure for a proxy, potentially quarantining it
    pub async fn record_failure(&self, proxy: &FreeProxy, reason: String) -> bool {
        let key = Self::proxy_key(proxy);
        let mut quarantined = self.quarantined.write().await;

        if let Some(entry) = quarantined.get_mut(&key) {
            // Proxy already in quarantine, increment failures
            entry.consecutive_failures += 1;
            entry.failure_reasons.push(reason);
            
            // Extend quarantine with exponential backoff, up to max
            let multiplier = 2u32.pow(entry.consecutive_failures.min(5));
            let extended_duration = self.quarantine_duration * multiplier;
            let actual_duration = extended_duration.min(self.max_quarantine_duration);
            entry.release_at = Utc::now() + chrono::Duration::from_std(actual_duration).unwrap_or(chrono::Duration::hours(24));
            
            warn!(
                "Proxy {} failure #{}: {}. Quarantine extended to {:?}",
                key, entry.consecutive_failures, entry.failure_reasons.last().unwrap_or(&String::new()), entry.release_at
            );
            true
        } else {
            // First failure, check if we should quarantine
            let entry = QuarantinedProxy {
                proxy: proxy.clone(),
                consecutive_failures: 1,
                quarantined_at: Utc::now(),
                release_at: Utc::now() + chrono::Duration::from_std(self.quarantine_duration).unwrap_or(chrono::Duration::minutes(5)),
                failure_reasons: vec![reason.clone()],
            };
            
            if entry.consecutive_failures >= self.max_consecutive_failures {
                info!("Quarantining proxy {} after {} failures", key, entry.consecutive_failures);
                quarantined.insert(key, entry);
                true
            } else {
                // Track failure but don't quarantine yet
                quarantined.insert(key, entry);
                false
            }
        }
    }

    /// Record a success for a proxy, potentially releasing it from quarantine
    pub async fn record_success(&self, proxy: &FreeProxy) {
        let key = Self::proxy_key(proxy);
        let mut quarantined = self.quarantined.write().await;
        
        if quarantined.remove(&key).is_some() {
            info!("Proxy {} released from quarantine after successful validation", key);
        }
    }

    /// Check if a proxy is currently quarantined
    pub async fn is_quarantined(&self, proxy: &FreeProxy) -> bool {
        let key = Self::proxy_key(proxy);
        let quarantined = self.quarantined.read().await;
        
        if let Some(entry) = quarantined.get(&key) {
            entry.release_at > Utc::now()
        } else {
            false
        }
    }

    /// Get all quarantined proxies
    pub async fn get_quarantined(&self) -> Vec<QuarantinedProxy> {
        let quarantined = self.quarantined.read().await;
        quarantined.values().cloned().collect()
    }

    /// Release proxies that have served their quarantine time
    pub async fn release_expired(&self) -> Vec<FreeProxy> {
        let mut quarantined = self.quarantined.write().await;
        let now = Utc::now();
        
        let expired_keys: Vec<String> = quarantined
            .iter()
            .filter(|(_, entry)| entry.release_at <= now)
            .map(|(key, _)| key.clone())
            .collect();
        
        let mut released = Vec::new();
        for key in expired_keys {
            if let Some(entry) = quarantined.remove(&key) {
                info!("Releasing proxy {} from quarantine (served time)", key);
                released.push(entry.proxy);
            }
        }
        
        released
    }

    /// Get quarantine statistics
    pub async fn get_stats(&self) -> QuarantineStats {
        let quarantined = self.quarantined.read().await;
        let now = Utc::now();
        
        let total = quarantined.len();
        let active = quarantined.values().filter(|e| e.release_at > now).count();
        let pending_release = total - active;
        let avg_failures = if total > 0 {
            quarantined.values().map(|e| e.consecutive_failures as f64).sum::<f64>() / total as f64
        } else {
            0.0
        };
        
        QuarantineStats {
            total_quarantined: total,
            actively_quarantined: active,
            pending_release,
            average_failures: avg_failures,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineStats {
    pub total_quarantined: usize,
    pub actively_quarantined: usize,
    pub pending_release: usize,
    pub average_failures: f64,
}

// ============================================================================
// Geographic Verification System
// ============================================================================

/// Configuration for geographic verification
#[derive(Debug, Clone)]
pub struct GeoVerificationConfig {
    pub enabled: bool,
    pub tolerance_km: f64,
    pub geoip_api_urls: Vec<String>,
}

impl Default for GeoVerificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tolerance_km: 500.0, // Allow 500km tolerance for geo verification
            geoip_api_urls: vec![
                "https://ipapi.co/{ip}/json/".to_string(),
                "https://ip-api.com/json/{ip}".to_string(),
            ],
        }
    }
}

/// Geographic verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoVerificationResult {
    pub is_verified: bool,
    pub expected_country: String,
    pub detected_country: Option<String>,
    pub expected_location: Option<(f64, f64)>, // (lat, lon)
    pub detected_location: Option<(f64, f64)>,
    pub distance_km: Option<f64>,
    pub error: Option<String>,
}

/// Geographic verifier for proxies
pub struct GeoVerifier {
    config: GeoVerificationConfig,
    http_client: HttpClient,
}

impl GeoVerifier {
    pub fn new(config: GeoVerificationConfig) -> Result<Self> {
        Ok(Self {
            config,
            http_client: HttpClient::new()?,
        })
    }

    /// Verify that a proxy's detected IP matches its claimed geographic location
    pub async fn verify_proxy_location(&self, proxy: &FreeProxy, detected_ip: &str) -> GeoVerificationResult {
        if !self.config.enabled {
            return GeoVerificationResult {
                is_verified: true,
                expected_country: proxy.country.clone(),
                detected_country: Some(proxy.country.clone()),
                expected_location: None,
                detected_location: None,
                distance_km: None,
                error: Some("Geographic verification disabled".to_string()),
            };
        }

        // Try to get geo info for the detected IP
        let geo_result = self.lookup_ip_location(detected_ip).await;
        
        match geo_result {
            Ok((country, location)) => {
                let is_country_match = country.to_lowercase() == proxy.country.to_lowercase()
                    || country.to_lowercase() == proxy.country_code.to_lowercase();
                
                GeoVerificationResult {
                    is_verified: is_country_match,
                    expected_country: proxy.country.clone(),
                    detected_country: Some(country),
                    expected_location: None, // We don't have expected coordinates
                    detected_location: location,
                    distance_km: None,
                    error: None,
                }
            }
            Err(e) => {
                GeoVerificationResult {
                    is_verified: false,
                    expected_country: proxy.country.clone(),
                    detected_country: None,
                    expected_location: None,
                    detected_location: None,
                    distance_km: None,
                    error: Some(e.to_string()),
                }
            }
        }
    }

    /// Lookup IP location using geo IP APIs
    async fn lookup_ip_location(&self, ip: &str) -> Result<(String, Option<(f64, f64)>)> {
        for api_url_template in &self.config.geoip_api_urls {
            let url = api_url_template.replace("{ip}", ip);
            
            match self.http_client.get(&url).await {
                Ok(response) => {
                    // Try to parse common geo IP response formats
                    if let Ok(result) = self.parse_geo_response(&response) {
                        return Ok(result);
                    }
                }
                Err(e) => {
                    debug!("Geo API {} failed: {}", url, e);
                    continue;
                }
            }
        }
        
        Err(anyhow!("All geo IP APIs failed"))
    }

    /// Parse various geo IP API response formats
    fn parse_geo_response(&self, response: &str) -> Result<(String, Option<(f64, f64)>)> {
        // Try ipapi.co format
        #[derive(Deserialize)]
        struct IpApiCoResponse {
            country_name: Option<String>,
            country: Option<String>,
            latitude: Option<f64>,
            longitude: Option<f64>,
        }
        
        if let Ok(parsed) = serde_json::from_str::<IpApiCoResponse>(response) {
            let country = parsed.country_name
                .or(parsed.country)
                .ok_or_else(|| anyhow!("No country in response"))?;
            let location = match (parsed.latitude, parsed.longitude) {
                (Some(lat), Some(lon)) => Some((lat, lon)),
                _ => None,
            };
            return Ok((country, location));
        }
        
        // Try ip-api.com format
        #[derive(Deserialize)]
        struct IpApiResponse {
            country: Option<String>,
            lat: Option<f64>,
            lon: Option<f64>,
        }
        
        if let Ok(parsed) = serde_json::from_str::<IpApiResponse>(response) {
            let country = parsed.country.ok_or_else(|| anyhow!("No country in response"))?;
            let location = match (parsed.lat, parsed.lon) {
                (Some(lat), Some(lon)) => Some((lat, lon)),
                _ => None,
            };
            return Ok((country, location));
        }
        
        Err(anyhow!("Failed to parse geo response"))
    }

    /// Calculate distance between two coordinates in kilometers (Haversine formula)
    #[allow(dead_code)]
    fn calculate_distance_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        const EARTH_RADIUS_KM: f64 = 6371.0;
        
        let lat1_rad = lat1.to_radians();
        let lat2_rad = lat2.to_radians();
        let delta_lat = (lat2 - lat1).to_radians();
        let delta_lon = (lon2 - lon1).to_radians();
        
        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().asin();
        
        EARTH_RADIUS_KM * c
    }
}

// ============================================================================
// Enhanced Proxy Health Checker with Quarantine
// ============================================================================

/// Enhanced health checker that integrates with quarantine system
pub struct EnhancedProxyHealthChecker {
    validator: ProxyValidator,
    quarantine_manager: ProxyQuarantineManager,
    geo_verifier: Option<GeoVerifier>,
    check_interval: Duration,
}

impl EnhancedProxyHealthChecker {
    pub fn new(
        validator: ProxyValidator,
        quarantine_manager: ProxyQuarantineManager,
        geo_verifier: Option<GeoVerifier>,
        check_interval: Duration,
    ) -> Self {
        Self {
            validator,
            quarantine_manager,
            geo_verifier,
            check_interval,
        }
    }

    /// Process a single proxy validation result
    async fn process_validation_result(
        &self,
        proxy: &FreeProxy,
        result: &ValidationResult,
        proxies_lock: &mut tokio::sync::RwLockWriteGuard<'_, Vec<FreeProxy>>,
    ) {
        if result.is_working {
            self.handle_working_proxy(proxy, result, proxies_lock).await;
        } else {
            self.handle_failed_proxy(proxy, result, proxies_lock).await;
        }
    }

    /// Handle a proxy that passed validation
    async fn handle_working_proxy(
        &self,
        proxy: &FreeProxy,
        result: &ValidationResult,
        proxies_lock: &mut tokio::sync::RwLockWriteGuard<'_, Vec<FreeProxy>>,
    ) {
        let geo_verified = self.verify_geo_location(proxy, result).await;
        
        if geo_verified && !result.has_ip_leak {
            self.mark_proxy_success(proxy, proxies_lock).await;
        } else {
            let reason = if !geo_verified {
                "Geographic verification failed".to_string()
            } else {
                "IP leak detected".to_string()
            };
            self.quarantine_proxy(proxy, reason, proxies_lock).await;
        }
    }

    /// Handle a proxy that failed validation
    async fn handle_failed_proxy(
        &self,
        proxy: &FreeProxy,
        result: &ValidationResult,
        proxies_lock: &mut tokio::sync::RwLockWriteGuard<'_, Vec<FreeProxy>>,
    ) {
        let reason = result.error.clone().unwrap_or_else(|| "Connection failed".to_string());
        let quarantined = self.quarantine_manager.record_failure(proxy, reason).await;
        
        if quarantined {
            proxies_lock.retain(|p| !(p.ip == proxy.ip && p.port == proxy.port));
        } else {
            self.update_proxy_status(proxy, false, proxies_lock);
        }
    }

    /// Verify geographic location of proxy
    async fn verify_geo_location(&self, proxy: &FreeProxy, result: &ValidationResult) -> bool {
        match (&self.geo_verifier, &result.detected_ip) {
            (Some(verifier), Some(detected_ip)) => {
                verifier.verify_proxy_location(proxy, detected_ip).await.is_verified
            }
            _ => true, // No geo verification available, assume OK
        }
    }

    /// Mark proxy as successful and update status
    async fn mark_proxy_success(
        &self,
        proxy: &FreeProxy,
        proxies_lock: &mut tokio::sync::RwLockWriteGuard<'_, Vec<FreeProxy>>,
    ) {
        self.quarantine_manager.record_success(proxy).await;
        self.update_proxy_status(proxy, true, proxies_lock);
    }

    /// Quarantine a proxy and optionally remove from pool
    async fn quarantine_proxy(
        &self,
        proxy: &FreeProxy,
        reason: String,
        proxies_lock: &mut tokio::sync::RwLockWriteGuard<'_, Vec<FreeProxy>>,
    ) {
        let quarantined = self.quarantine_manager.record_failure(proxy, reason).await;
        if quarantined {
            proxies_lock.retain(|p| !(p.ip == proxy.ip && p.port == proxy.port));
        }
    }

    /// Update proxy status in the pool
    fn update_proxy_status(
        &self,
        proxy: &FreeProxy,
        is_working: bool,
        proxies_lock: &mut tokio::sync::RwLockWriteGuard<'_, Vec<FreeProxy>>,
    ) {
        if let Some(p) = proxies_lock.iter_mut().find(|p| p.ip == proxy.ip && p.port == proxy.port) {
            p.is_working = is_working;
            p.last_checked = Utc::now().to_rfc3339();
        }
    }

    /// Release expired quarantined proxies back to the pool
    async fn release_expired_proxies(&self, proxies: &Arc<RwLock<Vec<FreeProxy>>>) {
        let released = self.quarantine_manager.release_expired().await;
        if released.is_empty() {
            return;
        }
        
        info!("Released {} proxies from quarantine", released.len());
        let mut proxies_lock = proxies.write().await;
        for proxy in released {
            let already_exists = proxies_lock.iter().any(|p| p.ip == proxy.ip && p.port == proxy.port);
            if !already_exists {
                proxies_lock.push(proxy);
            }
        }
    }

    /// Get non-quarantined proxies to check
    async fn get_proxies_to_check(&self, proxies: &Arc<RwLock<Vec<FreeProxy>>>) -> Vec<FreeProxy> {
        let proxies_lock = proxies.read().await;
        let mut to_check = Vec::new();
        for proxy in proxies_lock.iter() {
            if !self.quarantine_manager.is_quarantined(proxy).await {
                to_check.push(proxy.clone());
            }
        }
        to_check
    }

    /// Start the enhanced health monitoring loop
    pub async fn start_monitoring(&self, proxies: Arc<RwLock<Vec<FreeProxy>>>) {
        let mut interval = tokio::time::interval(self.check_interval);
        
        loop {
            interval.tick().await;
            
            self.release_expired_proxies(&proxies).await;
            
            let proxies_to_check = self.get_proxies_to_check(&proxies).await;
            
            if proxies_to_check.is_empty() {
                debug!("No proxies to check (all quarantined or empty pool)");
                continue;
            }
            
            info!("Starting enhanced health check for {} proxies", proxies_to_check.len());
            
            let results = self.validator.validate_batch(&proxies_to_check).await;
            
            let mut proxies_lock = proxies.write().await;
            for (proxy, result) in results {
                self.process_validation_result(&proxy, &result, &mut proxies_lock).await;
            }
            
            self.log_health_check_stats(&proxies_lock).await;
        }
    }

    /// Log health check statistics
    async fn log_health_check_stats(&self, proxies_lock: &tokio::sync::RwLockWriteGuard<'_, Vec<FreeProxy>>) {
        let stats = self.quarantine_manager.get_stats().await;
        let working_count = proxies_lock.iter().filter(|p| p.is_working).count();
        info!(
            "Health check completed: {}/{} working, {} quarantined",
            working_count, proxies_lock.len(), stats.actively_quarantined
        );
    }
}
