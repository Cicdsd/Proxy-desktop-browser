//! Tests for Chromium engine configuration structures
//! 
//! This test suite validates all configuration-related structs and enums
//! used by the Chromium engine implementation.

use browser_core::chromium_engine::{
    BrowserEngineType, ChromiumEngineConfig, CookieIsolationMode, FingerprintConfig,
    Geolocation, NetworkCondition, ProxyAuth,
};

#[test]
fn test_browser_engine_type_default() {
    let engine_type = BrowserEngineType::default();
    assert_eq!(engine_type, BrowserEngineType::System);
}

#[test]
fn test_browser_engine_type_variants() {
    let system = BrowserEngineType::System;
    let chromium = BrowserEngineType::IntegratedChromium;
    
    assert_ne!(system, chromium);
    assert_eq!(system, BrowserEngineType::System);
    assert_eq!(chromium, BrowserEngineType::IntegratedChromium);
}

#[test]
fn test_network_condition_none() {
    let condition = NetworkCondition::None;
    let (download, upload, latency) = condition.get_params();
    
    assert_eq!(download, -1.0);
    assert_eq!(upload, -1.0);
    assert_eq!(latency, 0.0);
}

#[test]
fn test_network_condition_slow_3g() {
    let condition = NetworkCondition::Slow3G;
    let (download, upload, latency) = condition.get_params();
    
    // 500 kbps = 500 * 1024 / 8 bytes/sec
    assert_eq!(download, 500.0 * 1024.0 / 8.0);
    assert_eq!(upload, 500.0 * 1024.0 / 8.0);
    assert_eq!(latency, 400.0);
}

#[test]
fn test_network_condition_fast_3g() {
    let condition = NetworkCondition::Fast3G;
    let (download, upload, latency) = condition.get_params();
    
    assert_eq!(download, 1.5 * 1024.0 * 1024.0 / 8.0);
    assert_eq!(upload, 750.0 * 1024.0 / 8.0);
    assert_eq!(latency, 150.0);
}

#[test]
fn test_network_condition_lte() {
    let condition = NetworkCondition::LTE;
    let (download, upload, latency) = condition.get_params();
    
    assert_eq!(download, 12.0 * 1024.0 * 1024.0 / 8.0);
    assert_eq!(upload, 5.0 * 1024.0 * 1024.0 / 8.0);
    assert_eq!(latency, 50.0);
}

#[test]
fn test_network_condition_custom() {
    let condition = NetworkCondition::Custom {
        download_throughput: 1000.0,
        upload_throughput: 500.0,
        latency: 100.0,
    };
    let (download, upload, latency) = condition.get_params();
    
    assert_eq!(download, 1000.0);
    assert_eq!(upload, 500.0);
    assert_eq!(latency, 100.0);
}

#[test]
fn test_network_condition_default() {
    let condition = NetworkCondition::default();
    let (download, upload, latency) = condition.get_params();
    
    assert_eq!(download, -1.0);
    assert_eq!(upload, -1.0);
    assert_eq!(latency, 0.0);
}

#[test]
fn test_fingerprint_config_default() {
    let config = FingerprintConfig::default();
    
    assert!(config.randomize_canvas);
    assert!(config.randomize_webgl);
    assert!(config.randomize_audio);
    assert!(!config.spoof_screen);
    assert_eq!(config.screen_width, 1920);
    assert_eq!(config.screen_height, 1080);
    assert!(!config.spoof_hardware_concurrency);
    assert_eq!(config.hardware_concurrency, 8);
    assert!(!config.spoof_device_memory);
    assert_eq!(config.device_memory, 8);
    assert!(!config.spoof_timezone);
    assert_eq!(config.timezone, "America/New_York");
    assert!(!config.spoof_language);
    assert_eq!(config.language, "en-US");
    assert!(!config.spoof_platform);
    assert_eq!(config.platform, "Win32");
}

#[test]
fn test_fingerprint_config_custom() {
    let config = FingerprintConfig {
        randomize_canvas: false,
        randomize_webgl: false,
        randomize_audio: false,
        spoof_screen: true,
        screen_width: 2560,
        screen_height: 1440,
        spoof_hardware_concurrency: true,
        hardware_concurrency: 16,
        spoof_device_memory: true,
        device_memory: 32,
        spoof_timezone: true,
        timezone: "Europe/London".to_string(),
        spoof_language: true,
        language: "en-GB".to_string(),
        spoof_platform: true,
        platform: "MacIntel".to_string(),
    };
    
    assert!(!config.randomize_canvas);
    assert!(config.spoof_screen);
    assert_eq!(config.screen_width, 2560);
    assert_eq!(config.screen_height, 1440);
    assert_eq!(config.hardware_concurrency, 16);
    assert_eq!(config.device_memory, 32);
    assert_eq!(config.timezone, "Europe/London");
    assert_eq!(config.language, "en-GB");
    assert_eq!(config.platform, "MacIntel");
}

#[test]
fn test_proxy_auth() {
    let auth = ProxyAuth {
        username: "testuser".to_string(),
        password: "testpass".to_string(),
    };
    
    assert_eq!(auth.username, "testuser");
    assert_eq!(auth.password, "testpass");
}

#[test]
fn test_cookie_isolation_mode_default() {
    let mode = CookieIsolationMode::default();
    assert!(matches!(mode, CookieIsolationMode::None));
}

#[test]
fn test_cookie_isolation_mode_variants() {
    let modes = vec![
        CookieIsolationMode::None,
        CookieIsolationMode::PerTab,
        CookieIsolationMode::PerDomain,
        CookieIsolationMode::FullContext,
    ];
    
    assert_eq!(modes.len(), 4);
}

#[test]
fn test_geolocation() {
    let geo = Geolocation {
        latitude: 37.7749,
        longitude: -122.4194,
        accuracy: 100.0,
    };
    
    assert_eq!(geo.latitude, 37.7749);
    assert_eq!(geo.longitude, -122.4194);
    assert_eq!(geo.accuracy, 100.0);
}

#[test]
fn test_chromium_engine_config_default() {
    let config = ChromiumEngineConfig::default();
    
    assert!(config.executable_path.is_none());
    assert!(!config.headless);
    assert!(config.user_data_dir.is_none());
    assert!(config.sandbox);
    assert!(config.extra_args.is_empty());
    assert!(config.proxy.is_none());
    assert!(config.proxy_auth.is_none());
    assert!(config.stealth_mode);
    assert!(config.user_agent.is_none());
    assert_eq!(config.viewport_width, 1920);
    assert_eq!(config.viewport_height, 1080);
    assert!(config.webrtc_protection);
    assert_eq!(config.doh_server, Some("https://cloudflare-dns.com/dns-query".to_string()));
    assert!(matches!(config.network_condition, NetworkCondition::None));
    assert!(matches!(config.cookie_isolation, CookieIsolationMode::None));
    assert!(config.blocked_urls.is_empty());
    assert!(!config.enable_interception);
    assert!(config.geolocation.is_none());
}

#[test]
fn test_chromium_engine_config_custom() {
    use std::path::PathBuf;
    use browser_core::proxy::{ProxySettings, ProxyType};
    
    let proxy = ProxySettings {
        proxy_type: ProxyType::Http,
        host: Some("proxy.example.com".to_string()),
        port: Some(8080),
        username: Some("user".to_string()),
        password: Some("pass".to_string()),
        dns_servers: vec!["1.1.1.1".to_string()],
        bypass_list: vec!["localhost".to_string()],
    };
    
    let config = ChromiumEngineConfig {
        executable_path: Some(PathBuf::from("/usr/bin/chromium")),
        headless: true,
        user_data_dir: Some(PathBuf::from("/tmp/chrome-data")),
        sandbox: false,
        extra_args: vec!["--disable-gpu".to_string()],
        proxy: Some(proxy),
        proxy_auth: Some(ProxyAuth {
            username: "user".to_string(),
            password: "pass".to_string(),
        }),
        stealth_mode: false,
        user_agent: Some("CustomUA/1.0".to_string()),
        viewport_width: 1366,
        viewport_height: 768,
        webrtc_protection: false,
        doh_server: None,
        network_condition: NetworkCondition::Fast3G,
        fingerprint: FingerprintConfig::default(),
        cookie_isolation: CookieIsolationMode::PerTab,
        blocked_urls: vec!["*.ads.com".to_string()],
        enable_interception: true,
        geolocation: Some(Geolocation {
            latitude: 51.5074,
            longitude: -0.1278,
            accuracy: 50.0,
        }),
    };
    
    assert!(config.executable_path.is_some());
    assert!(config.headless);
    assert!(config.user_data_dir.is_some());
    assert!(!config.sandbox);
    assert_eq!(config.extra_args.len(), 1);
    assert!(config.proxy.is_some());
    assert!(config.proxy_auth.is_some());
    assert!(!config.stealth_mode);
    assert_eq!(config.user_agent, Some("CustomUA/1.0".to_string()));
    assert_eq!(config.viewport_width, 1366);
    assert_eq!(config.viewport_height, 768);
    assert!(!config.webrtc_protection);
    assert!(config.doh_server.is_none());
    assert!(matches!(config.network_condition, NetworkCondition::Fast3G));
    assert!(matches!(config.cookie_isolation, CookieIsolationMode::PerTab));
    assert_eq!(config.blocked_urls.len(), 1);
    assert!(config.enable_interception);
    assert!(config.geolocation.is_some());
}

#[test]
fn test_config_serialization() {
    let config = ChromiumEngineConfig::default();
    let serialized = serde_json::to_string(&config).expect("Failed to serialize");
    let deserialized: ChromiumEngineConfig = 
        serde_json::from_str(&serialized).expect("Failed to deserialize");
    
    assert_eq!(config.headless, deserialized.headless);
    assert_eq!(config.stealth_mode, deserialized.stealth_mode);
    assert_eq!(config.viewport_width, deserialized.viewport_width);
}

#[test]
fn test_network_condition_serialization() {
    let conditions = vec![
        NetworkCondition::None,
        NetworkCondition::Slow3G,
        NetworkCondition::Fast3G,
        NetworkCondition::LTE,
        NetworkCondition::Custom {
            download_throughput: 1000.0,
            upload_throughput: 500.0,
            latency: 100.0,
        },
    ];
    
    for condition in conditions {
        let serialized = serde_json::to_string(&condition).expect("Failed to serialize");
        let deserialized: NetworkCondition = 
            serde_json::from_str(&serialized).expect("Failed to deserialize");
        
        let (d1, u1, l1) = condition.get_params();
        let (d2, u2, l2) = deserialized.get_params();
        
        assert_eq!(d1, d2);
        assert_eq!(u1, u2);
        assert_eq!(l1, l2);
    }
}
