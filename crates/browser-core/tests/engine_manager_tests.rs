//! Tests for BrowserEngineManager
//! 
//! This test suite validates the engine manager's ability to switch between
//! different browser engines and manage their configurations.

use browser_core::chromium_engine::{
    BrowserEngineManager, BrowserEngineType, ChromiumEngineConfig,
};
use browser_core::proxy::{ProxySettings, ProxyType};

#[tokio::test]
async fn test_engine_manager_creation() {
    let manager = BrowserEngineManager::new();
    let engine_type = manager.get_engine_type().await;
    
    assert_eq!(engine_type, BrowserEngineType::System);
}

#[tokio::test]
async fn test_engine_manager_default() {
    let manager = BrowserEngineManager::default();
    let engine_type = manager.get_engine_type().await;
    
    assert_eq!(engine_type, BrowserEngineType::System);
}

#[tokio::test]
async fn test_get_initial_config() {
    let manager = BrowserEngineManager::new();
    let config = manager.get_config().await;
    
    assert_eq!(config.viewport_width, 1920);
    assert_eq!(config.viewport_height, 1080);
    assert!(config.stealth_mode);
}

#[tokio::test]
async fn test_update_chromium_config() {
    let manager = BrowserEngineManager::new();
    
    let mut config = ChromiumEngineConfig::default();
    config.headless = true;
    config.viewport_width = 1366;
    config.viewport_height = 768;
    
    let result = manager.update_chromium_config(config.clone()).await;
    assert!(result.is_ok());
    
    let updated_config = manager.get_config().await;
    assert_eq!(updated_config.headless, true);
    assert_eq!(updated_config.viewport_width, 1366);
    assert_eq!(updated_config.viewport_height, 768);
}

#[tokio::test]
async fn test_set_proxy() {
    let manager = BrowserEngineManager::new();
    
    let proxy = ProxySettings {
        proxy_type: ProxyType::Http,
        host: Some("proxy.example.com".to_string()),
        port: Some(8080),
        username: Some("user".to_string()),
        password: Some("pass".to_string()),
        dns_servers: vec![],
        bypass_list: vec!["localhost".to_string()],
    };
    
    let result = manager.set_proxy(Some(proxy.clone())).await;
    assert!(result.is_ok());
    
    let config = manager.get_config().await;
    assert!(config.proxy.is_some());
    
    let stored_proxy = config.proxy.unwrap();
    assert_eq!(stored_proxy.host, Some("proxy.example.com".to_string()));
    assert_eq!(stored_proxy.port, Some(8080));
}

#[tokio::test]
async fn test_clear_proxy() {
    let manager = BrowserEngineManager::new();
    
    // Set a proxy first
    let proxy = ProxySettings {
        proxy_type: ProxyType::Http,
        host: Some("proxy.example.com".to_string()),
        port: Some(8080),
        username: None,
        password: None,
        dns_servers: vec![],
        bypass_list: vec![],
    };
    
    manager.set_proxy(Some(proxy)).await.unwrap();
    
    // Clear it
    let result = manager.set_proxy(None).await;
    assert!(result.is_ok());
    
    let config = manager.get_config().await;
    assert!(config.proxy.is_none());
}

#[tokio::test]
async fn test_engine_type_no_change() {
    let manager = BrowserEngineManager::new();
    
    // Should succeed without doing anything
    let result = manager.set_engine_type(BrowserEngineType::System).await;
    assert!(result.is_ok());
    
    let engine_type = manager.get_engine_type().await;
    assert_eq!(engine_type, BrowserEngineType::System);
}

#[tokio::test]
async fn test_supports_per_tab_proxy() {
    let manager = BrowserEngineManager::new();
    
    // Should always return true for integrated Chromium engine capability
    assert!(manager.supports_per_tab_proxy());
}

#[tokio::test]
async fn test_get_capabilities() {
    let manager = BrowserEngineManager::new();
    let capabilities = manager.get_capabilities();
    
    assert!(capabilities.per_tab_proxy);
    assert!(capabilities.webrtc_protection);
    assert!(capabilities.stealth_mode);
    assert!(capabilities.dns_over_https);
    assert!(capabilities.custom_user_agent);
    assert!(capabilities.javascript_injection);
    assert!(capabilities.network_interception);
    assert!(capabilities.cookie_management);
}

#[tokio::test]
async fn test_get_chromium_engine_when_system() {
    let manager = BrowserEngineManager::new();
    
    // Should return None when using System engine
    let engine = manager.get_chromium_engine().await;
    assert!(engine.is_none());
}

#[tokio::test]
async fn test_multiple_config_updates() {
    let manager = BrowserEngineManager::new();
    
    // First update
    let mut config1 = ChromiumEngineConfig::default();
    config1.headless = true;
    manager.update_chromium_config(config1).await.unwrap();
    
    // Second update
    let mut config2 = ChromiumEngineConfig::default();
    config2.headless = false;
    config2.stealth_mode = false;
    manager.update_chromium_config(config2).await.unwrap();
    
    // Verify latest config
    let final_config = manager.get_config().await;
    assert!(!final_config.headless);
    assert!(!final_config.stealth_mode);
}

#[tokio::test]
async fn test_proxy_with_authentication() {
    let manager = BrowserEngineManager::new();
    
    let proxy = ProxySettings {
        proxy_type: ProxyType::Https,
        host: Some("secure-proxy.example.com".to_string()),
        port: Some(443),
        username: Some("admin".to_string()),
        password: Some("secret123".to_string()),
        dns_servers: vec![],
        bypass_list: vec!["*.local".to_string(), "127.0.0.1".to_string()],
    };
    
    manager.set_proxy(Some(proxy)).await.unwrap();
    
    let config = manager.get_config().await;
    let stored_proxy = config.proxy.unwrap();
    
    assert_eq!(stored_proxy.username, Some("admin".to_string()));
    assert_eq!(stored_proxy.password, Some("secret123".to_string()));
    assert_eq!(stored_proxy.bypass_list.len(), 2);
}

#[tokio::test]
async fn test_config_persistence_across_gets() {
    let manager = BrowserEngineManager::new();
    
    let mut config = ChromiumEngineConfig::default();
    config.viewport_width = 2560;
    config.viewport_height = 1440;
    
    manager.update_chromium_config(config).await.unwrap();
    
    // Get config multiple times
    let config1 = manager.get_config().await;
    let config2 = manager.get_config().await;
    let config3 = manager.get_config().await;
    
    assert_eq!(config1.viewport_width, config2.viewport_width);
    assert_eq!(config2.viewport_width, config3.viewport_width);
    assert_eq!(config1.viewport_height, config2.viewport_height);
}

#[tokio::test]
async fn test_concurrent_config_reads() {
    use tokio::task::JoinSet;
    
    let manager = std::sync::Arc::new(BrowserEngineManager::new());
    let mut set = JoinSet::new();
    
    // Spawn multiple concurrent readers
    for _ in 0..10 {
        let manager_clone = manager.clone();
        set.spawn(async move {
            let config = manager_clone.get_config().await;
            config.viewport_width
        });
    }
    
    // All should succeed and return the same value
    let mut results = Vec::new();
    while let Some(result) = set.join_next().await {
        results.push(result.unwrap());
    }
    
    assert_eq!(results.len(), 10);
    assert!(results.iter().all(|&w| w == 1920));
}

#[tokio::test]
async fn test_concurrent_config_writes() {
    use tokio::task::JoinSet;
    
    let manager = std::sync::Arc::new(BrowserEngineManager::new());
    let mut set = JoinSet::new();
    
    // Spawn multiple concurrent writers
    for i in 0..5 {
        let manager_clone = manager.clone();
        set.spawn(async move {
            let mut config = ChromiumEngineConfig::default();
            config.viewport_width = 1000 + (i * 100) as u32;
            manager_clone.update_chromium_config(config).await
        });
    }
    
    // All writes should succeed
    while let Some(result) = set.join_next().await {
        assert!(result.unwrap().is_ok());
    }
    
    // Final config should have one of the written values
    let final_config = manager.get_config().await;
    let width = final_config.viewport_width;
    assert!(width >= 1000 && width <= 1400);
}

#[tokio::test]
async fn test_proxy_with_different_types() {
    let manager = BrowserEngineManager::new();
    
    let proxy_types = vec![
        ProxyType::Http,
        ProxyType::Https,
        ProxyType::Socks5,
    ];
    
    for proxy_type in proxy_types {
        let proxy = ProxySettings {
            proxy_type: proxy_type.clone(),
            host: Some("proxy.test.com".to_string()),
            port: Some(8080),
            username: None,
            password: None,
            dns_servers: vec![],
            bypass_list: vec![],
        };
        
        manager.set_proxy(Some(proxy.clone())).await.unwrap();
        
        let config = manager.get_config().await;
        assert!(config.proxy.is_some());
        assert_eq!(config.proxy.as_ref().unwrap().proxy_type, proxy_type);
    }
}

#[tokio::test]
async fn test_config_cloning() {
    let manager = BrowserEngineManager::new();
    
    let mut config = ChromiumEngineConfig::default();
    config.headless = true;
    config.stealth_mode = false;
    
    manager.update_chromium_config(config).await.unwrap();
    
    let config1 = manager.get_config().await;
    let config2 = config1.clone();
    
    assert_eq!(config1.headless, config2.headless);
    assert_eq!(config1.stealth_mode, config2.stealth_mode);
    assert_eq!(config1.viewport_width, config2.viewport_width);
}
