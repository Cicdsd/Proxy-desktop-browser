//! Tests for Chromium tab operations
//! 
//! This test suite validates tab management functionality including
//! creation, state tracking, and proxy assignment.

use browser_core::chromium_engine::{ChromiumEngine, ChromiumEngineConfig, ChromiumTab};
use browser_core::proxy::{ProxySettings, ProxyType};

#[test]
fn test_chromium_tab_creation() {
    let tab = ChromiumTab {
        id: "tab-1".to_string(),
        url: "https://example.com".to_string(),
        title: "Example Domain".to_string(),
        proxy: None,
        is_loading: false,
        can_go_back: false,
        can_go_forward: false,
    };
    
    assert_eq!(tab.id, "tab-1");
    assert_eq!(tab.url, "https://example.com");
    assert_eq!(tab.title, "Example Domain");
    assert!(tab.proxy.is_none());
    assert!(!tab.is_loading);
    assert!(!tab.can_go_back);
    assert!(!tab.can_go_forward);
}

#[test]
fn test_chromium_tab_with_proxy() {
    let proxy = ProxySettings {
        proxy_type: ProxyType::Http,
        host: Some("proxy.example.com".to_string()),
        port: Some(8080),
        username: None,
        password: None,
        dns_servers: vec![],
        bypass_list: vec![],
    };
    
    let tab = ChromiumTab {
        id: "tab-2".to_string(),
        url: "about:blank".to_string(),
        title: "New Tab".to_string(),
        proxy: Some(proxy.clone()),
        is_loading: false,
        can_go_back: false,
        can_go_forward: false,
    };
    
    assert!(tab.proxy.is_some());
    let tab_proxy = tab.proxy.unwrap();
    assert_eq!(tab_proxy.host, Some("proxy.example.com".to_string()));
    assert_eq!(tab_proxy.port, Some(8080));
}

#[test]
fn test_chromium_tab_clone() {
    let tab1 = ChromiumTab {
        id: "tab-3".to_string(),
        url: "https://rust-lang.org".to_string(),
        title: "Rust".to_string(),
        proxy: None,
        is_loading: true,
        can_go_back: true,
        can_go_forward: false,
    };
    
    let tab2 = tab1.clone();
    
    assert_eq!(tab1.id, tab2.id);
    assert_eq!(tab1.url, tab2.url);
    assert_eq!(tab1.title, tab2.title);
    assert_eq!(tab1.is_loading, tab2.is_loading);
    assert_eq!(tab1.can_go_back, tab2.can_go_back);
}

#[test]
fn test_chromium_tab_serialization() {
    let tab = ChromiumTab {
        id: "tab-4".to_string(),
        url: "https://example.com".to_string(),
        title: "Test".to_string(),
        proxy: None,
        is_loading: false,
        can_go_back: false,
        can_go_forward: false,
    };
    
    let serialized = serde_json::to_string(&tab).expect("Failed to serialize");
    let deserialized: ChromiumTab = 
        serde_json::from_str(&serialized).expect("Failed to deserialize");
    
    assert_eq!(tab.id, deserialized.id);
    assert_eq!(tab.url, deserialized.url);
    assert_eq!(tab.title, deserialized.title);
}

#[tokio::test]
async fn test_engine_create_instance() {
    let config = ChromiumEngineConfig::default();
    let engine = ChromiumEngine::new(config);
    
    assert!(!engine.is_running().await);
}

#[tokio::test]
async fn test_engine_get_config() {
    let config = ChromiumEngineConfig {
        headless: true,
        viewport_width: 1366,
        viewport_height: 768,
        ..Default::default()
    };
    
    let engine = ChromiumEngine::new(config.clone());
    let retrieved_config = engine.get_config();
    
    assert_eq!(retrieved_config.headless, config.headless);
    assert_eq!(retrieved_config.viewport_width, config.viewport_width);
    assert_eq!(retrieved_config.viewport_height, config.viewport_height);
}

#[tokio::test]
async fn test_engine_set_config() {
    let initial_config = ChromiumEngineConfig::default();
    let mut engine = ChromiumEngine::new(initial_config);
    
    let new_config = ChromiumEngineConfig {
        headless: true,
        stealth_mode: false,
        viewport_width: 2560,
        ..Default::default()
    };
    
    engine.set_config(new_config.clone());
    
    let retrieved_config = engine.get_config();
    assert_eq!(retrieved_config.headless, new_config.headless);
    assert_eq!(retrieved_config.stealth_mode, new_config.stealth_mode);
    assert_eq!(retrieved_config.viewport_width, new_config.viewport_width);
}

#[tokio::test]
async fn test_engine_initial_state() {
    let config = ChromiumEngineConfig::default();
    let engine = ChromiumEngine::new(config);
    
    assert!(!engine.is_running().await);
    
    let tabs = engine.get_tabs().await;
    assert!(tabs.is_empty());
    
    let active_tab = engine.get_active_tab().await;
    assert!(active_tab.is_none());
}

#[tokio::test]
async fn test_engine_shutdown_when_not_running() {
    let config = ChromiumEngineConfig::default();
    let mut engine = ChromiumEngine::new(config);
    
    // Should succeed even if not running
    let result = engine.shutdown().await;
    assert!(result.is_ok());
    assert!(!engine.is_running().await);
}

#[tokio::test]
async fn test_tab_state_tracking() {
    let mut tab = ChromiumTab {
        id: "tab-5".to_string(),
        url: "https://example.com".to_string(),
        title: "Example".to_string(),
        proxy: None,
        is_loading: true,
        can_go_back: false,
        can_go_forward: false,
    };
    
    // Simulate navigation completion
    tab.is_loading = false;
    tab.title = "Example Domain".to_string();
    
    assert!(!tab.is_loading);
    assert_eq!(tab.title, "Example Domain");
}

#[tokio::test]
async fn test_tab_history_simulation() {
    let mut tab = ChromiumTab {
        id: "tab-6".to_string(),
        url: "https://first.com".to_string(),
        title: "First".to_string(),
        proxy: None,
        is_loading: false,
        can_go_back: false,
        can_go_forward: false,
    };
    
    // Navigate to second page
    tab.url = "https://second.com".to_string();
    tab.title = "Second".to_string();
    tab.can_go_back = true;
    
    assert!(tab.can_go_back);
    assert!(!tab.can_go_forward);
    
    // Go back
    tab.url = "https://first.com".to_string();
    tab.can_go_back = false;
    tab.can_go_forward = true;
    
    assert!(!tab.can_go_back);
    assert!(tab.can_go_forward);
}

#[tokio::test]
async fn test_multiple_tabs_with_different_proxies() {
    let proxy1 = ProxySettings {
        proxy_type: ProxyType::Http,
        host: Some("proxy1.example.com".to_string()),
        port: Some(8080),
        username: None,
        password: None,
        dns_servers: vec![],
        bypass_list: vec![],
    };
    
    let proxy2 = ProxySettings {
        proxy_type: ProxyType::Socks5,
        host: Some("proxy2.example.com".to_string()),
        port: Some(1080),
        username: Some("user".to_string()),
        password: Some("pass".to_string()),
        dns_servers: vec![],
        bypass_list: vec![],
    };
    
    let tab1 = ChromiumTab {
        id: "tab-7".to_string(),
        url: "https://site1.com".to_string(),
        title: "Site 1".to_string(),
        proxy: Some(proxy1.clone()),
        is_loading: false,
        can_go_back: false,
        can_go_forward: false,
    };
    
    let tab2 = ChromiumTab {
        id: "tab-8".to_string(),
        url: "https://site2.com".to_string(),
        title: "Site 2".to_string(),
        proxy: Some(proxy2.clone()),
        is_loading: false,
        can_go_back: false,
        can_go_forward: false,
    };
    
    let tab3 = ChromiumTab {
        id: "tab-9".to_string(),
        url: "https://site3.com".to_string(),
        title: "Site 3".to_string(),
        proxy: None,
        is_loading: false,
        can_go_back: false,
        can_go_forward: false,
    };
    
    // Verify each tab has correct proxy
    assert!(tab1.proxy.is_some());
    assert_eq!(tab1.proxy.as_ref().unwrap().host, Some("proxy1.example.com".to_string()));
    
    assert!(tab2.proxy.is_some());
    assert_eq!(tab2.proxy.as_ref().unwrap().host, Some("proxy2.example.com".to_string()));
    assert_eq!(tab2.proxy.as_ref().unwrap().port, Some(1080));
    
    assert!(tab3.proxy.is_none());
}

#[tokio::test]
async fn test_tab_proxy_update() {
    let mut tab = ChromiumTab {
        id: "tab-10".to_string(),
        url: "https://example.com".to_string(),
        title: "Example".to_string(),
        proxy: None,
        is_loading: false,
        can_go_back: false,
        can_go_forward: false,
    };
    
    assert!(tab.proxy.is_none());
    
    // Add proxy
    let proxy = ProxySettings {
        proxy_type: ProxyType::Http,
        host: Some("new-proxy.com".to_string()),
        port: Some(3128),
        username: None,
        password: None,
        dns_servers: vec![],
        bypass_list: vec![],
    };
    
    tab.proxy = Some(proxy);
    assert!(tab.proxy.is_some());
    
    // Remove proxy
    tab.proxy = None;
    assert!(tab.proxy.is_none());
}

#[tokio::test]
async fn test_tab_loading_states() {
    let mut tab = ChromiumTab {
        id: "tab-11".to_string(),
        url: "about:blank".to_string(),
        title: "".to_string(),
        proxy: None,
        is_loading: false,
        can_go_back: false,
        can_go_forward: false,
    };
    
    // Start loading
    tab.is_loading = true;
    assert!(tab.is_loading);
    
    // Finish loading
    tab.is_loading = false;
    tab.url = "https://example.com".to_string();
    tab.title = "Example Domain".to_string();
    
    assert!(!tab.is_loading);
    assert_eq!(tab.url, "https://example.com");
    assert_eq!(tab.title, "Example Domain");
}

#[test]
fn test_tab_with_special_urls() {
    let special_urls = vec![
        "about:blank",
        "chrome://settings",
        "data:text/html,<h1>Test</h1>",
        "file:///tmp/test.html",
    ];
    
    for url in special_urls {
        let tab = ChromiumTab {
            id: format!("tab-{}", url.len()),
            url: url.to_string(),
            title: "Special URL".to_string(),
            proxy: None,
            is_loading: false,
            can_go_back: false,
            can_go_forward: false,
        };
        
        assert_eq!(tab.url, url);
    }
}

#[test]
fn test_tab_with_unicode_title() {
    let tab = ChromiumTab {
        id: "tab-unicode".to_string(),
        url: "https://example.com".to_string(),
        title: "测试 テスト 테스트 🦀".to_string(),
        proxy: None,
        is_loading: false,
        can_go_back: false,
        can_go_forward: false,
    };
    
    assert!(tab.title.contains("测试"));
    assert!(tab.title.contains("🦀"));
}

#[tokio::test]
async fn test_engine_config_with_all_options() {
    use std::path::PathBuf;
    use browser_core::chromium_engine::{
        CookieIsolationMode, FingerprintConfig, Geolocation, NetworkCondition, ProxyAuth,
    };
    
    let config = ChromiumEngineConfig {
        executable_path: Some(PathBuf::from("/usr/bin/chromium")),
        headless: true,
        user_data_dir: Some(PathBuf::from("/tmp/chrome")),
        sandbox: false,
        extra_args: vec!["--disable-gpu".to_string()],
        proxy: Some(ProxySettings {
            proxy_type: ProxyType::Http,
            host: Some("proxy.com".to_string()),
            port: Some(8080),
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
            dns_servers: vec![],
            bypass_list: vec!["localhost".to_string()],
        }),
        proxy_auth: Some(ProxyAuth {
            username: "user".to_string(),
            password: "pass".to_string(),
        }),
        stealth_mode: true,
        user_agent: Some("CustomUA/1.0".to_string()),
        viewport_width: 1366,
        viewport_height: 768,
        webrtc_protection: true,
        doh_server: Some("https://dns.google/dns-query".to_string()),
        network_condition: NetworkCondition::Fast3G,
        fingerprint: FingerprintConfig::default(),
        cookie_isolation: CookieIsolationMode::PerTab,
        blocked_urls: vec!["*.ads.com".to_string()],
        enable_interception: true,
        geolocation: Some(Geolocation {
            latitude: 37.7749,
            longitude: -122.4194,
            accuracy: 100.0,
        }),
    };
    
    let engine = ChromiumEngine::new(config);
    let retrieved = engine.get_config();
    
    assert!(retrieved.headless);
    assert!(!retrieved.sandbox);
    assert!(retrieved.stealth_mode);
    assert_eq!(retrieved.viewport_width, 1366);
    assert_eq!(retrieved.extra_args.len(), 1);
    assert!(retrieved.proxy.is_some());
    assert!(retrieved.proxy_auth.is_some());
    assert!(retrieved.geolocation.is_some());
}
