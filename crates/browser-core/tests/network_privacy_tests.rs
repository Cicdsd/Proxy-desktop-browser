use browser_core::{
    DnsManager, DnsConfig, DnsStrategy,
    PrivacyManager, PrivacyConfig, PrivacyLevel, TrackingCategory,
    WebRtcIpPolicy, ReferrerPolicy,
};

/// Test DNS manager creation
#[tokio::test]
async fn test_dns_manager_creation() {
    let manager = DnsManager::new();
    let config = manager.get_config().await;
    
    assert_eq!(config.strategy, DnsStrategy::DoH);
    assert!(config.cache_enabled);
    assert!(config.validate_dnssec);
}

/// Test DNS config customization
#[tokio::test]
async fn test_dns_config_custom() {
    let config = DnsConfig {
        strategy: DnsStrategy::DoT,
        providers: vec![],
        cache_enabled: false,
        cache_max_entries: 5000,
        timeout_ms: 3000,
        retry_count: 2,
        validate_dnssec: false,
        block_malicious: true,
    };
    
    let manager = DnsManager::with_config(config);
    let retrieved = manager.get_config().await;
    
    assert_eq!(retrieved.strategy, DnsStrategy::DoT);
    assert!(!retrieved.cache_enabled);
    assert_eq!(retrieved.timeout_ms, 3000);
}

/// Test DNS domain blocking
#[tokio::test]
async fn test_dns_domain_blocking() {
    let manager = DnsManager::new();
    
    // Block a domain
    manager.block_domain("malware.example.com").await;
    
    assert!(manager.is_blocked("malware.example.com").await);
    assert!(manager.is_blocked("sub.malware.example.com").await);
    assert!(!manager.is_blocked("safe.example.com").await);
    
    // Unblock
    manager.unblock_domain("malware.example.com").await;
    assert!(!manager.is_blocked("malware.example.com").await);
}

/// Test DNS blocked domains list
#[tokio::test]
async fn test_dns_blocked_domains_list() {
    let manager = DnsManager::new();
    
    manager.block_domain("bad1.com").await;
    manager.block_domain("bad2.com").await;
    manager.block_domain("bad3.com").await;
    
    let blocked = manager.get_blocked_domains().await;
    assert_eq!(blocked.len(), 3);
    assert!(blocked.contains(&"bad1.com".to_string()));
    assert!(blocked.contains(&"bad2.com".to_string()));
}

/// Test DNS cache operations
#[tokio::test]
async fn test_dns_cache_clear() {
    let manager = DnsManager::new();
    
    // Clear cache should not panic
    manager.clear_cache().await;
    
    let stats = manager.get_stats().await;
    assert_eq!(stats.total_queries, 0);
}

/// Test DNS statistics
#[tokio::test]
async fn test_dns_stats() {
    let manager = DnsManager::new();
    
    let stats = manager.get_stats().await;
    assert_eq!(stats.total_queries, 0);
    assert_eq!(stats.cache_hits, 0);
    assert_eq!(stats.cache_misses, 0);
    assert_eq!(stats.blocked_queries, 0);
}

/// Test DNS cache hit rate calculation
#[tokio::test]
async fn test_dns_cache_hit_rate() {
    let manager = DnsManager::new();
    
    // With no queries, rate should be 0
    let rate = manager.get_cache_hit_rate().await;
    assert_eq!(rate, 0.0);
}

/// Test DNS config update
#[tokio::test]
async fn test_dns_config_update() {
    let manager = DnsManager::new();
    
    let new_config = DnsConfig {
        strategy: DnsStrategy::System,
        providers: vec![],
        cache_enabled: true,
        cache_max_entries: 1000,
        timeout_ms: 10000,
        retry_count: 5,
        validate_dnssec: true,
        block_malicious: false,
    };
    
    manager.set_config(new_config).await;
    
    let config = manager.get_config().await;
    assert_eq!(config.strategy, DnsStrategy::System);
    assert_eq!(config.timeout_ms, 10000);
    assert!(!config.block_malicious);
}

/// Test privacy manager creation
#[tokio::test]
async fn test_privacy_manager_creation() {
    let manager = PrivacyManager::new();
    let config = manager.get_config().await;
    
    assert_eq!(config.level, PrivacyLevel::Standard);
    assert!(config.block_trackers);
    assert!(config.block_cookies_third_party);
    assert!(config.send_dnt_header);
}

/// Test privacy level changes
#[tokio::test]
async fn test_privacy_levels() {
    let manager = PrivacyManager::new();
    
    // Set to Maximum
    manager.set_privacy_level(PrivacyLevel::Maximum).await;
    let config = manager.get_config().await;
    
    assert_eq!(config.level, PrivacyLevel::Maximum);
    assert!(config.block_cookies_all);
    assert!(config.clear_cookies_on_exit);
    assert!(config.clear_history_on_exit);
    assert_eq!(config.webrtc_ip_handling, WebRtcIpPolicy::Disabled);
    
    // Set to Minimal
    manager.set_privacy_level(PrivacyLevel::Minimal).await;
    let config = manager.get_config().await;
    
    assert_eq!(config.level, PrivacyLevel::Minimal);
    assert!(!config.block_trackers);
}

/// Test privacy headers
#[tokio::test]
async fn test_privacy_headers() {
    let manager = PrivacyManager::new();
    let headers = manager.get_privacy_headers().await;
    
    // Check DNT header
    assert!(headers.iter().any(|(k, v)| k == "DNT" && v == "1"));
    
    // Check GPC header
    assert!(headers.iter().any(|(k, v)| k == "Sec-GPC" && v == "1"));
    
    // Check Referrer-Policy
    assert!(headers.iter().any(|(k, _)| k == "Referrer-Policy"));
}

/// Test fingerprint spoofing
#[tokio::test]
async fn test_fingerprint_spoofing() {
    let manager = PrivacyManager::new();
    
    let fp = manager.get_spoofed_fingerprint().await;
    
    assert!(fp.canvas_noise_enabled);
    assert!(fp.webgl_vendor.is_some());
    assert!(fp.webgl_renderer.is_some());
    assert!(fp.audio_noise_enabled);
}

/// Test fingerprint regeneration
#[tokio::test]
async fn test_fingerprint_regeneration() {
    let manager = PrivacyManager::new();
    
    let fp1 = manager.get_spoofed_fingerprint().await;
    let seed1 = fp1.canvas_noise_seed;
    
    manager.regenerate_fingerprint().await;
    
    let fp2 = manager.get_spoofed_fingerprint().await;
    let seed2 = fp2.canvas_noise_seed;
    
    // Seeds should be different after regeneration
    assert_ne!(seed1, seed2);
}

/// Test tracker blocklist management
#[tokio::test]
async fn test_tracker_blocklist() {
    let manager = PrivacyManager::new();
    
    // Add custom tracker
    manager.add_to_blocklist("custom-tracker.com", TrackingCategory::Analytics).await;
    
    // Remove from blocklist
    manager.remove_from_blocklist("custom-tracker.com").await;
}

/// Test privacy statistics
#[tokio::test]
async fn test_privacy_stats() {
    let manager = PrivacyManager::new();
    
    // Record some events
    manager.record_fingerprint_blocked().await;
    manager.record_fingerprint_blocked().await;
    manager.record_webrtc_leak_prevented().await;
    
    let stats = manager.get_stats().await;
    assert_eq!(stats.fingerprint_attempts_blocked, 2);
    assert_eq!(stats.webrtc_leaks_prevented, 1);
}

/// Test privacy config update
#[tokio::test]
async fn test_privacy_config_update() {
    let manager = PrivacyManager::new();
    
    let new_config = PrivacyConfig {
        level: PrivacyLevel::Strict,
        block_trackers: true,
        block_cookies_third_party: true,
        block_cookies_all: false,
        clear_cookies_on_exit: true,
        clear_history_on_exit: false,
        clear_cache_on_exit: false,
        send_dnt_header: true,
        send_gpc_header: false,
        fingerprint_protection: browser_core::FingerprintConfig::default(),
        webrtc_ip_handling: WebRtcIpPolicy::DisableNonProxied,
        referrer_policy: ReferrerPolicy::NoReferrer,
    };
    
    manager.set_config(new_config).await;
    
    let config = manager.get_config().await;
    assert_eq!(config.level, PrivacyLevel::Strict);
    assert!(config.clear_cookies_on_exit);
    assert!(!config.send_gpc_header);
}

/// Test referrer policy values
#[test]
fn test_referrer_policy_header_values() {
    assert_eq!(ReferrerPolicy::NoReferrer.as_header_value(), "no-referrer");
    assert_eq!(ReferrerPolicy::Origin.as_header_value(), "origin");
    assert_eq!(ReferrerPolicy::SameOrigin.as_header_value(), "same-origin");
    assert_eq!(ReferrerPolicy::StrictOriginWhenCrossOrigin.as_header_value(), "strict-origin-when-cross-origin");
}

/// Test clear all privacy data
#[tokio::test]
async fn test_clear_all_privacy_data() {
    let manager = PrivacyManager::new();
    
    // Record some data
    manager.record_fingerprint_blocked().await;
    manager.record_webrtc_leak_prevented().await;
    
    let stats_before = manager.get_stats().await;
    assert!(stats_before.fingerprint_attempts_blocked > 0);
    
    // Clear all
    manager.clear_all_data().await;
    
    let stats_after = manager.get_stats().await;
    assert_eq!(stats_after.fingerprint_attempts_blocked, 0);
    assert_eq!(stats_after.webrtc_leaks_prevented, 0);
}

/// Test blocked tracker category counting
#[tokio::test]
async fn test_blocked_by_category() {
    let manager = PrivacyManager::new();
    
    // Get blocked by category (should be empty initially)
    let counts = manager.get_blocked_by_category().await;
    assert!(counts.is_empty());
}

/// Test recent blocked trackers
#[tokio::test]
async fn test_recent_blocked_trackers() {
    let manager = PrivacyManager::new();
    
    // Get recent (should be empty initially)
    let recent = manager.get_recent_blocked(10).await;
    assert!(recent.is_empty());
}
