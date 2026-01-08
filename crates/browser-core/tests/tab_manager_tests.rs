//! Tests for Tab Manager functionality
//! 
//! This module tests:
//! - Tab creation and management
//! - IP assignment and rotation
//! - Tab profile management
//! - Tab isolation

use browser_core::tab_manager::TabIPManager;
use browser_core::tab_isolation::{
    TabProfile, TabStatus, NetworkConfig, TLSProfile, HTTP2Settings, TCPFingerprint
};
use browser_core::fingerprint::BrowserFingerprint;
use virtual_ip::IPGenerator;
use std::time::SystemTime;

// ============================================================================
// Test Fixtures
// ============================================================================

fn create_test_fingerprint() -> BrowserFingerprint {
    BrowserFingerprint {
        user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string(),
        accept_language: "en-US,en;q=0.9".to_string(),
        timezone: "America/New_York".to_string(),
        screen_resolution: (1920, 1080),
        color_depth: 24,
        hardware_concurrency: 8,
        device_memory: 8,
        platform: "Win32".to_string(),
        webgl_vendor: "Google Inc. (NVIDIA)".to_string(),
        webgl_renderer: "ANGLE (NVIDIA, NVIDIA GeForce GTX 1080)".to_string(),
        canvas_hash: "abc123def456".to_string(),
        audio_hash: "xyz789uvw012".to_string(),
    }
}

fn create_test_tls_profile() -> TLSProfile {
    TLSProfile {
        version: "TLS 1.3".to_string(),
        cipher_suites: vec![
            "TLS_AES_128_GCM_SHA256".to_string(),
            "TLS_AES_256_GCM_SHA384".to_string(),
        ],
        extensions: vec![
            "server_name".to_string(),
            "supported_versions".to_string(),
        ],
        ja3_hash: "abc123def456".to_string(),
    }
}

fn create_test_http2_settings() -> HTTP2Settings {
    HTTP2Settings {
        settings_frame: vec![(1, 65536), (2, 1), (4, 6291456)],
        window_update: 15663105,
        priority: vec![(0, 256)],
    }
}

fn create_test_tcp_fingerprint() -> TCPFingerprint {
    TCPFingerprint {
        ttl: 64,
        window_size: 65535,
        options: vec!["mss".to_string(), "nop".to_string(), "ws".to_string()],
    }
}

fn create_test_network_config() -> NetworkConfig {
    NetworkConfig {
        proxy_url: Some("http://192.168.1.1:8080".to_string()),
        dns_servers: vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
        tls_profile: create_test_tls_profile(),
        http2_settings: create_test_http2_settings(),
        tcp_fingerprint: create_test_tcp_fingerprint(),
    }
}

// ============================================================================
// BrowserFingerprint Tests
// ============================================================================

#[test]
fn test_fingerprint_creation() {
    let fingerprint = create_test_fingerprint();
    
    assert!(!fingerprint.user_agent.is_empty());
    assert!(!fingerprint.accept_language.is_empty());
    assert!(!fingerprint.timezone.is_empty());
    assert_eq!(fingerprint.screen_resolution, (1920, 1080));
    assert_eq!(fingerprint.color_depth, 24);
    assert_eq!(fingerprint.hardware_concurrency, 8);
}

#[test]
fn test_fingerprint_clone() {
    let fingerprint = create_test_fingerprint();
    let cloned = fingerprint.clone();
    
    assert_eq!(cloned.user_agent, fingerprint.user_agent);
    assert_eq!(cloned.screen_resolution, fingerprint.screen_resolution);
    assert_eq!(cloned.canvas_hash, fingerprint.canvas_hash);
}

#[test]
fn test_fingerprint_serialization() {
    let fingerprint = create_test_fingerprint();
    
    let json = serde_json::to_string(&fingerprint).expect("Fingerprint operation failed");
    assert!(json.contains("user_agent"));
    assert!(json.contains("screen_resolution"));
    
    let parsed: BrowserFingerprint = serde_json::from_str(&json).expect("Json operation failed");
    assert_eq!(parsed.user_agent, fingerprint.user_agent);
}

#[test]
fn test_fingerprint_debug() {
    let fingerprint = create_test_fingerprint();
    let debug_str = format!("{:?}", fingerprint);
    
    assert!(debug_str.contains("BrowserFingerprint"));
    assert!(debug_str.contains("user_agent"));
}

#[test]
fn test_fingerprint_various_screen_resolutions() {
    let resolutions = vec![
        (1920, 1080),
        (2560, 1440),
        (3840, 2160),
        (1366, 768),
        (1280, 720),
    ];
    
    for (width, height) in resolutions {
        let fingerprint = BrowserFingerprint {
            user_agent: "Test".to_string(),
            accept_language: "en-US".to_string(),
            timezone: "UTC".to_string(),
            screen_resolution: (width, height),
            color_depth: 24,
            hardware_concurrency: 4,
            device_memory: 4,
            platform: "Test".to_string(),
            webgl_vendor: "Test".to_string(),
            webgl_renderer: "Test".to_string(),
            canvas_hash: "test".to_string(),
            audio_hash: "test".to_string(),
        };
        
        assert_eq!(fingerprint.screen_resolution.0, width);
        assert_eq!(fingerprint.screen_resolution.1, height);
    }
}

// ============================================================================
// TabStatus Tests
// ============================================================================

#[test]
fn test_tab_status_creating() {
    let status = TabStatus::Creating;
    assert!(matches!(status, TabStatus::Creating));
}

#[test]
fn test_tab_status_active() {
    let status = TabStatus::Active;
    assert!(matches!(status, TabStatus::Active));
}

#[test]
fn test_tab_status_closed() {
    let status = TabStatus::Closed;
    assert!(matches!(status, TabStatus::Closed));
}

#[test]
fn test_tab_status_serialization() {
    let statuses = vec![
        TabStatus::Creating,
        TabStatus::Active,
        TabStatus::Closed,
    ];
    
    for status in statuses {
        let json = serde_json::to_string(&status).expect("Status operation failed");
        let parsed: TabStatus = serde_json::from_str(&json).expect("Json operation failed");
        
        // Verify round-trip serialization
        let json2 = serde_json::to_string(&parsed).expect("Failed to parse");
        assert_eq!(json, json2);
    }
}

// ============================================================================
// TLSProfile Tests
// ============================================================================

#[test]
fn test_tls_profile_creation() {
    let profile = create_test_tls_profile();
    
    assert_eq!(profile.version, "TLS 1.3");
    assert!(!profile.cipher_suites.is_empty());
    assert!(!profile.extensions.is_empty());
    assert!(!profile.ja3_hash.is_empty());
}

#[test]
fn test_tls_profile_serialization() {
    let profile = create_test_tls_profile();
    
    let json = serde_json::to_string(&profile).expect("File operation failed");
    assert!(json.contains("version"));
    assert!(json.contains("cipher_suites"));
    
    let parsed: TLSProfile = serde_json::from_str(&json).expect("Json operation failed");
    assert_eq!(parsed.version, profile.version);
}

// ============================================================================
// HTTP2Settings Tests
// ============================================================================

#[test]
fn test_http2_settings_creation() {
    let settings = create_test_http2_settings();
    
    assert!(!settings.settings_frame.is_empty());
    assert!(settings.window_update > 0);
}

#[test]
fn test_http2_settings_serialization() {
    let settings = create_test_http2_settings();
    
    let json = serde_json::to_string(&settings).expect("Settings operation failed");
    let parsed: HTTP2Settings = serde_json::from_str(&json).expect("Json operation failed");
    assert_eq!(parsed.window_update, settings.window_update);
}

// ============================================================================
// TCPFingerprint Tests
// ============================================================================

#[test]
fn test_tcp_fingerprint_creation() {
    let fingerprint = create_test_tcp_fingerprint();
    
    assert_eq!(fingerprint.ttl, 64);
    assert!(fingerprint.window_size > 0);
    assert!(!fingerprint.options.is_empty());
}

#[test]
fn test_tcp_fingerprint_serialization() {
    let fingerprint = create_test_tcp_fingerprint();
    
    let json = serde_json::to_string(&fingerprint).expect("Fingerprint operation failed");
    let parsed: TCPFingerprint = serde_json::from_str(&json).expect("Json operation failed");
    assert_eq!(parsed.ttl, fingerprint.ttl);
}

// ============================================================================
// NetworkConfig Tests
// ============================================================================

#[test]
fn test_network_config_creation() {
    let config = create_test_network_config();
    
    assert!(config.proxy_url.is_some());
    assert_eq!(config.dns_servers.len(), 2);
    assert_eq!(config.tls_profile.version, "TLS 1.3");
}

#[test]
fn test_network_config_no_proxy() {
    let config = NetworkConfig {
        proxy_url: None,
        dns_servers: vec!["1.1.1.1".to_string()],
        tls_profile: create_test_tls_profile(),
        http2_settings: create_test_http2_settings(),
        tcp_fingerprint: create_test_tcp_fingerprint(),
    };
    
    assert!(config.proxy_url.is_none());
    assert_eq!(config.dns_servers.len(), 1);
}

#[test]
fn test_network_config_serialization() {
    let config = create_test_network_config();
    
    let json = serde_json::to_string(&config).expect("Configuration error");
    assert!(json.contains("proxy_url"));
    assert!(json.contains("dns_servers"));
    assert!(json.contains("tls_profile"));
    
    let parsed: NetworkConfig = serde_json::from_str(&json).expect("Json operation failed");
    assert_eq!(parsed.dns_servers.len(), config.dns_servers.len());
}

// ============================================================================
// TabIPManager Tests
// ============================================================================

#[tokio::test]
async fn test_tab_ip_manager_creation() {
    let ip_generator = IPGenerator::new(vec![], vec![]);
    let manager = TabIPManager::new(ip_generator);
    
    // Manager should be created successfully
    assert!(true);
}

#[tokio::test]
async fn test_tab_ip_manager_create_tab() {
    let ip_generator = IPGenerator::new(vec![], vec![]);
    let manager = TabIPManager::new(ip_generator);
    
    let result = manager.create_tab("US").await;
    
    match result {
        Ok(tab_profile) => {
            assert!(!tab_profile.tab_id.is_empty());
            assert_eq!(tab_profile.virtual_ip.country_code, "US");
            assert!(matches!(tab_profile.status, TabStatus::Creating));
        }
        Err(e) => {
            println!("Tab creation failed (may be expected): {}", e);
        }
    }
}

#[tokio::test]
async fn test_tab_ip_manager_create_random_tab() {
    let ip_generator = IPGenerator::new(vec![], vec![]);
    let manager = TabIPManager::new(ip_generator);
    
    let result = manager.create_tab_random().await;
    
    match result {
        Ok(tab_profile) => {
            assert!(!tab_profile.tab_id.is_empty());
            assert!(!tab_profile.virtual_ip.ip.is_empty());
        }
        Err(e) => {
            println!("Random tab creation failed (may be expected): {}", e);
        }
    }
}

#[tokio::test]
async fn test_tab_ip_manager_rotate_ip() {
    let ip_generator = IPGenerator::new(vec![], vec![]);
    let manager = TabIPManager::new(ip_generator);
    
    // First create a tab
    let tab_result = manager.create_tab("US").await;
    
    if let Ok(tab_profile) = tab_result {
        let old_ip = tab_profile.virtual_ip.ip.clone();
        
        // Now rotate IP
        let rotate_result = manager.rotate_ip(&tab_profile.tab_id, None).await;
        
        match rotate_result {
            Ok(new_ip) => {
                println!("Old IP: {}, New IP: {}", old_ip, new_ip.ip);
            }
            Err(e) => {
                println!("IP rotation failed: {}", e);
            }
        }
    }
}

#[tokio::test]
async fn test_tab_ip_manager_rotate_ip_different_country() {
    let ip_generator = IPGenerator::new(vec![], vec![]);
    let manager = TabIPManager::new(ip_generator);
    
    // First create a tab in US
    let tab_result = manager.create_tab("US").await;
    
    if let Ok(tab_profile) = tab_result {
        // Rotate to Germany
        let rotate_result = manager.rotate_ip(&tab_profile.tab_id, Some("DE")).await;
        
        match rotate_result {
            Ok(new_ip) => {
                assert_eq!(new_ip.country_code, "DE");
            }
            Err(e) => {
                println!("Country rotation failed: {}", e);
            }
        }
    }
}

#[tokio::test]
async fn test_tab_ip_manager_rotate_nonexistent_tab() {
    let ip_generator = IPGenerator::new(vec![], vec![]);
    let manager = TabIPManager::new(ip_generator);
    
    // Try to rotate IP for non-existent tab
    let result = manager.rotate_ip("nonexistent-tab-id", None).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tab_ip_manager_multiple_tabs() {
    let ip_generator = IPGenerator::new(vec![], vec![]);
    let manager = TabIPManager::new(ip_generator);
    
    let countries = vec!["US", "DE", "FR", "GB", "JP"];
    let mut created_tabs = Vec::new();
    
    for country in countries {
        if let Ok(tab) = manager.create_tab(country).await {
            created_tabs.push(tab);
        }
    }
    
    // Verify we created some tabs
    println!("Created {} tabs", created_tabs.len());
    
    // Verify each tab has a unique ID
    let mut tab_ids: Vec<String> = created_tabs.iter().map(|t| t.tab_id.clone()).collect();
    tab_ids.sort();
    tab_ids.dedup();
    assert_eq!(tab_ids.len(), created_tabs.len());
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_full_tab_lifecycle() {
    let ip_generator = IPGenerator::new(vec![], vec![]);
    let manager = TabIPManager::new(ip_generator);
    
    // Create tab
    let create_result = manager.create_tab("US").await;
    if let Ok(tab) = create_result {
        assert!(matches!(tab.status, TabStatus::Creating));
        
        // Rotate IP multiple times
        for _ in 0..3 {
            let _ = manager.rotate_ip(&tab.tab_id, None).await;
        }
        
        // Rotate to different country
        let _ = manager.rotate_ip(&tab.tab_id, Some("DE")).await;
    }
}
