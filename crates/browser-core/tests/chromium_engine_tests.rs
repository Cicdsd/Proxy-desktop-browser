//! Tests for ChromiumEngine lifecycle and operations
//! 
//! This test suite validates the core engine functionality including
//! initialization, lifecycle management, and error handling.
//! 
//! Note: Most tests are unit tests that don't require actual browser launch
//! to avoid CI/CD complexity and test flakiness.

use browser_core::chromium_engine::{
    ChromiumEngine, ChromiumEngineConfig, EngineCapabilities,
};
use browser_core::proxy::{ProxySettings, ProxyType};
use std::path::PathBuf;

#[tokio::test]
async fn test_engine_new() {
    let config = ChromiumEngineConfig::default();
    let engine = ChromiumEngine::new(config);
    
    assert!(!engine.is_running().await);
}

#[tokio::test]
async fn test_engine_not_running_initially() {
    let config = ChromiumEngineConfig::default();
    let engine = ChromiumEngine::new(config);
    
    let running = engine.is_running().await;
    assert!(!running);
}

#[tokio::test]
async fn test_engine_get_tabs_empty() {
    let config = ChromiumEngineConfig::default();
    let engine = ChromiumEngine::new(config);
    
    let tabs = engine.get_tabs().await;
    assert_eq!(tabs.len(), 0);
}

#[tokio::test]
async fn test_engine_get_active_tab_none() {
    let config = ChromiumEngineConfig::default();
    let engine = ChromiumEngine::new(config);
    
    let active_tab = engine.get_active_tab().await;
    assert!(active_tab.is_none());
}

#[tokio::test]
async fn test_engine_shutdown_without_launch() {
    let config = ChromiumEngineConfig::default();
    let mut engine = ChromiumEngine::new(config);
    
    let result = engine.shutdown().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_engine_double_shutdown() {
    let config = ChromiumEngineConfig::default();
    let mut engine = ChromiumEngine::new(config);
    
    let result1 = engine.shutdown().await;
    assert!(result1.is_ok());
    
    let result2 = engine.shutdown().await;
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_engine_with_custom_viewport() {
    let config = ChromiumEngineConfig {
        viewport_width: 1366,
        viewport_height: 768,
        ..Default::default()
    };
    
    let engine = ChromiumEngine::new(config);
    let retrieved_config = engine.get_config();
    
    assert_eq!(retrieved_config.viewport_width, 1366);
    assert_eq!(retrieved_config.viewport_height, 768);
}

#[tokio::test]
async fn test_engine_with_headless_mode() {
    let config = ChromiumEngineConfig {
        headless: true,
        ..Default::default()
    };
    
    let engine = ChromiumEngine::new(config);
    assert!(engine.get_config().headless);
}

#[tokio::test]
async fn test_engine_with_stealth_mode() {
    let config = ChromiumEngineConfig {
        stealth_mode: true,
        ..Default::default()
    };
    
    let engine = ChromiumEngine::new(config);
    assert!(engine.get_config().stealth_mode);
}

#[tokio::test]
async fn test_engine_with_webrtc_protection() {
    let config = ChromiumEngineConfig {
        webrtc_protection: true,
        ..Default::default()
    };
    
    let engine = ChromiumEngine::new(config);
    assert!(engine.get_config().webrtc_protection);
}

#[tokio::test]
async fn test_engine_with_proxy() {
    let proxy = ProxySettings {
        proxy_type: ProxyType::Http,
        host: Some("proxy.example.com".to_string()),
        port: Some(8080),
        username: None,
        password: None,
        dns_servers: vec![],
        bypass_list: vec!["localhost".to_string()],
    };
    
    let config = ChromiumEngineConfig {
        proxy: Some(proxy.clone()),
        ..Default::default()
    };
    
    let engine = ChromiumEngine::new(config);
    let retrieved_config = engine.get_config();
    
    assert!(retrieved_config.proxy.is_some());
    let retrieved_proxy = retrieved_config.proxy.as_ref().unwrap();
    assert_eq!(retrieved_proxy.host, Some("proxy.example.com".to_string()));
    assert_eq!(retrieved_proxy.port, Some(8080));
}

#[tokio::test]
async fn test_engine_with_custom_user_agent() {
    let config = ChromiumEngineConfig {
        user_agent: Some("CustomUA/1.0".to_string()),
        ..Default::default()
    };
    
    let engine = ChromiumEngine::new(config);
    assert_eq!(
        engine.get_config().user_agent,
        Some("CustomUA/1.0".to_string())
    );
}

#[tokio::test]
async fn test_engine_with_doh_server() {
    let config = ChromiumEngineConfig {
        doh_server: Some("https://dns.google/dns-query".to_string()),
        ..Default::default()
    };
    
    let engine = ChromiumEngine::new(config);
    assert_eq!(
        engine.get_config().doh_server,
        Some("https://dns.google/dns-query".to_string())
    );
}

#[tokio::test]
async fn test_engine_with_extra_args() {
    let config = ChromiumEngineConfig {
        extra_args: vec![
            "--disable-gpu".to_string(),
            "--no-sandbox".to_string(),
        ],
        ..Default::default()
    };
    
    let engine = ChromiumEngine::new(config);
    let args = &engine.get_config().extra_args;
    
    assert_eq!(args.len(), 2);
    assert!(args.contains(&"--disable-gpu".to_string()));
    assert!(args.contains(&"--no-sandbox".to_string()));
}

#[tokio::test]
async fn test_engine_with_user_data_dir() {
    let config = ChromiumEngineConfig {
        user_data_dir: Some(PathBuf::from("/tmp/chrome-data")),
        ..Default::default()
    };
    
    let engine = ChromiumEngine::new(config);
    assert!(engine.get_config().user_data_dir.is_some());
}

#[tokio::test]
async fn test_engine_with_executable_path() {
    let config = ChromiumEngineConfig {
        executable_path: Some(PathBuf::from("/usr/bin/chromium")),
        ..Default::default()
    };
    
    let engine = ChromiumEngine::new(config);
    assert!(engine.get_config().executable_path.is_some());
}

#[tokio::test]
async fn test_engine_config_update() {
    let initial_config = ChromiumEngineConfig {
        headless: false,
        viewport_width: 1920,
        ..Default::default()
    };
    
    let mut engine = ChromiumEngine::new(initial_config);
    
    let new_config = ChromiumEngineConfig {
        headless: true,
        viewport_width: 1366,
        ..Default::default()
    };
    
    engine.set_config(new_config);
    
    let retrieved = engine.get_config();
    assert!(retrieved.headless);
    assert_eq!(retrieved.viewport_width, 1366);
}

#[tokio::test]
async fn test_engine_set_tab_proxy_without_browser() {
    let config = ChromiumEngineConfig::default();
    let engine = ChromiumEngine::new(config);
    
    let proxy = ProxySettings {
        proxy_type: ProxyType::Http,
        host: Some("proxy.com".to_string()),
        port: Some(8080),
        username: None,
        password: None,
        dns_servers: vec![],
        bypass_list: vec![],
    };
    
    // Should fail because tab doesn't exist
    let result = engine.set_tab_proxy("nonexistent-tab", Some(proxy)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_engine_close_nonexistent_tab() {
    let config = ChromiumEngineConfig::default();
    let engine = ChromiumEngine::new(config);
    
    // Should succeed even if tab doesn't exist (idempotent operation)
    let result = engine.close_tab("nonexistent-tab").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_engine_set_active_tab_nonexistent() {
    let config = ChromiumEngineConfig::default();
    let engine = ChromiumEngine::new(config);
    
    let result = engine.set_active_tab("nonexistent-tab").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_engine_navigate_without_browser() {
    let config = ChromiumEngineConfig::default();
    let engine = ChromiumEngine::new(config);
    
    let result = engine.navigate("tab-1", "https://example.com").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_engine_create_tab_without_browser() {
    let config = ChromiumEngineConfig::default();
    let engine = ChromiumEngine::new(config);
    
    let result = engine.create_tab(Some("https://example.com"), None).await;
    assert!(result.is_err());
}

#[test]
fn test_engine_capabilities() {
    let caps = EngineCapabilities {
        per_tab_proxy: true,
        webrtc_protection: true,
        stealth_mode: true,
        dns_over_https: true,
        custom_user_agent: true,
        javascript_injection: true,
        network_interception: true,
        cookie_management: true,
    };
    
    assert!(caps.per_tab_proxy);
    assert!(caps.webrtc_protection);
    assert!(caps.stealth_mode);
    assert!(caps.dns_over_https);
    assert!(caps.custom_user_agent);
    assert!(caps.javascript_injection);
    assert!(caps.network_interception);
    assert!(caps.cookie_management);
}

#[test]
fn test_engine_capabilities_serialization() {
    let caps = EngineCapabilities {
        per_tab_proxy: true,
        webrtc_protection: true,
        stealth_mode: true,
        dns_over_https: true,
        custom_user_agent: true,
        javascript_injection: true,
        network_interception: true,
        cookie_management: true,
    };
    
    let serialized = serde_json::to_string(&caps).expect("Failed to serialize");
    let deserialized: EngineCapabilities = 
        serde_json::from_str(&serialized).expect("Failed to deserialize");
    
    assert_eq!(caps.per_tab_proxy, deserialized.per_tab_proxy);
    assert_eq!(caps.stealth_mode, deserialized.stealth_mode);
}

#[tokio::test]
async fn test_engine_with_blocked_urls() {
    let config = ChromiumEngineConfig {
        blocked_urls: vec![
            "*.ads.com".to_string(),
            "*.tracker.com".to_string(),
        ],
        enable_interception: true,
        ..Default::default()
    };
    
    let engine = ChromiumEngine::new(config);
    let retrieved = engine.get_config();
    
    assert_eq!(retrieved.blocked_urls.len(), 2);
    assert!(retrieved.enable_interception);
}

#[tokio::test]
async fn test_engine_with_geolocation() {
    use browser_core::chromium_engine::Geolocation;
    
    let config = ChromiumEngineConfig {
        geolocation: Some(Geolocation {
            latitude: 37.7749,
            longitude: -122.4194,
            accuracy: 100.0,
        }),
        ..Default::default()
    };
    
    let engine = ChromiumEngine::new(config);
    let retrieved = engine.get_config();
    
    assert!(retrieved.geolocation.is_some());
    let geo = retrieved.geolocation.as_ref().unwrap();
    assert_eq!(geo.latitude, 37.7749);
    assert_eq!(geo.longitude, -122.4194);
}

#[tokio::test]
async fn test_engine_with_network_condition() {
    use browser_core::chromium_engine::NetworkCondition;
    
    let config = ChromiumEngineConfig {
        network_condition: NetworkCondition::Fast3G,
        ..Default::default()
    };
    
    let engine = ChromiumEngine::new(config);
    let retrieved = engine.get_config();
    
    let (download, upload, latency) = retrieved.network_condition.get_params();
    assert_eq!(download, 1.5 * 1024.0 * 1024.0 / 8.0);
    assert_eq!(upload, 750.0 * 1024.0 / 8.0);
    assert_eq!(latency, 150.0);
}

#[tokio::test]
async fn test_engine_with_cookie_isolation() {
    use browser_core::chromium_engine::CookieIsolationMode;
    
    let config = ChromiumEngineConfig {
        cookie_isolation: CookieIsolationMode::PerTab,
        ..Default::default()
    };
    
    let engine = ChromiumEngine::new(config);
    let retrieved = engine.get_config();
    
    assert!(matches!(
        retrieved.cookie_isolation,
        CookieIsolationMode::PerTab
    ));
}

#[tokio::test]
async fn test_engine_with_fingerprint_config() {
    use browser_core::chromium_engine::FingerprintConfig;
    
    let fingerprint = FingerprintConfig {
        randomize_canvas: true,
        randomize_webgl: true,
        randomize_audio: true,
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
    
    let config = ChromiumEngineConfig {
        fingerprint,
        ..Default::default()
    };
    
    let engine = ChromiumEngine::new(config);
    let retrieved = engine.get_config();
    
    assert!(retrieved.fingerprint.randomize_canvas);
    assert_eq!(retrieved.fingerprint.screen_width, 2560);
    assert_eq!(retrieved.fingerprint.hardware_concurrency, 16);
    assert_eq!(retrieved.fingerprint.timezone, "Europe/London");
}

#[tokio::test]
async fn test_engine_with_proxy_auth() {
    use browser_core::chromium_engine::ProxyAuth;
    
    let config = ChromiumEngineConfig {
        proxy_auth: Some(ProxyAuth {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        }),
        ..Default::default()
    };
    
    let engine = ChromiumEngine::new(config);
    let retrieved = engine.get_config();
    
    assert!(retrieved.proxy_auth.is_some());
    let auth = retrieved.proxy_auth.as_ref().unwrap();
    assert_eq!(auth.username, "testuser");
    assert_eq!(auth.password, "testpass");
}

#[tokio::test]
async fn test_engine_sandbox_disabled() {
    let config = ChromiumEngineConfig {
        sandbox: false,
        ..Default::default()
    };
    
    let engine = ChromiumEngine::new(config);
    assert!(!engine.get_config().sandbox);
}

#[tokio::test]
async fn test_engine_multiple_config_updates() {
    let config1 = ChromiumEngineConfig {
        headless: false,
        ..Default::default()
    };
    
    let mut engine = ChromiumEngine::new(config1);
    
    let config2 = ChromiumEngineConfig {
        headless: true,
        viewport_width: 1366,
        ..Default::default()
    };
    
    engine.set_config(config2);
    
    let config3 = ChromiumEngineConfig {
        headless: true,
        viewport_width: 2560,
        stealth_mode: false,
        ..Default::default()
    };
    
    engine.set_config(config3);
    
    let final_config = engine.get_config();
    assert!(final_config.headless);
    assert_eq!(final_config.viewport_width, 2560);
    assert!(!final_config.stealth_mode);
}

#[tokio::test]
async fn test_engine_concurrent_operations() {
    use std::sync::Arc;
    use tokio::task::JoinSet;
    
    let config = ChromiumEngineConfig::default();
    let engine = Arc::new(ChromiumEngine::new(config));
    
    let mut set = JoinSet::new();
    
    // Spawn multiple concurrent readers
    for _ in 0..10 {
        let engine_clone = engine.clone();
        set.spawn(async move {
            engine_clone.is_running().await
        });
    }
    
    // All should succeed
    while let Some(result) = set.join_next().await {
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}

#[tokio::test]
async fn test_engine_get_tabs_concurrent() {
    use std::sync::Arc;
    use tokio::task::JoinSet;
    
    let config = ChromiumEngineConfig::default();
    let engine = Arc::new(ChromiumEngine::new(config));
    
    let mut set = JoinSet::new();
    
    for _ in 0..5 {
        let engine_clone = engine.clone();
        set.spawn(async move {
            engine_clone.get_tabs().await
        });
    }
    
    while let Some(result) = set.join_next().await {
        let tabs = result.unwrap();
        assert_eq!(tabs.len(), 0);
    }
}
