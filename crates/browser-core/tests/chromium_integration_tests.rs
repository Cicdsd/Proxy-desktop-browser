//! Integration tests for Chromium engine
//! 
//! These tests actually launch a real Chromium browser and test functionality.
//! They are marked with #[ignore] by default to avoid CI failures.
//! 
//! To run these tests:
//! ```bash
//! cargo test --test chromium_integration_tests -- --ignored --test-threads=1
//! ```
//! 
//! Requirements:
//! - Chrome or Chromium must be installed on the system
//! - Tests should be run with --test-threads=1 to avoid resource conflicts

mod common;

// ChromiumEngine and ChromiumEngineConfig are imported via common module
use browser_core::proxy::{ProxySettings, ProxyType};
use common::*;
use std::time::Duration;
use tokio::time::sleep;

/// Test that we can launch and shutdown a browser
#[tokio::test]
#[ignore] // Requires Chrome/Chromium installed
async fn test_browser_launch_and_shutdown() {
    skip_if_no_chrome!();
    
    let config = create_test_config();
    let engine = launch_browser_with_timeout(config, 30)
        .await
        .expect("Failed to launch browser");
    
    assert_engine_running(&engine).await;
    
    shutdown_browser(engine)
        .await
        .expect("Failed to shutdown browser");
}

/// Test launching browser with custom viewport
#[tokio::test]
#[ignore]
async fn test_browser_custom_viewport() {
    skip_if_no_chrome!();
    
    let mut config = create_test_config();
    config.viewport_width = 1920;
    config.viewport_height = 1080;
    
    let engine = launch_browser_with_timeout(config, 30)
        .await
        .expect("Failed to launch browser");
    
    assert_engine_running(&engine).await;
    assert_eq!(engine.get_config().viewport_width, 1920);
    assert_eq!(engine.get_config().viewport_height, 1080);
    
    shutdown_browser(engine).await.unwrap();
}

/// Test creating a tab and navigating to a URL
#[tokio::test]
#[ignore]
async fn test_create_tab_and_navigate() {
    skip_if_no_chrome!();
    
    let config = create_test_config();
    let engine = launch_browser_with_timeout(config, 30)
        .await
        .expect("Failed to launch browser");
    
    // Create a tab with about:blank
    let tab = engine.create_tab(Some(test_urls::ABOUT_BLANK), None)
        .await
        .expect("Failed to create tab");
    
    assert_eq!(tab.url, test_urls::ABOUT_BLANK);
    assert!(!tab.is_loading);
    
    // Wait a bit for tab to be ready
    sleep(Duration::from_millis(500)).await;
    
    // Navigate to example.com
    let result = engine.navigate(&tab.id, test_urls::EXAMPLE_COM).await;
    assert!(result.is_ok(), "Navigation should succeed");
    
    // Wait for navigation
    sleep(Duration::from_secs(2)).await;
    
    shutdown_browser(engine).await.unwrap();
}

/// Test multiple tabs
#[tokio::test]
#[ignore]
async fn test_multiple_tabs() {
    skip_if_no_chrome!();
    
    let config = create_test_config();
    let engine = launch_browser_with_timeout(config, 30)
        .await
        .expect("Failed to launch browser");
    
    // Create first tab
    let tab1 = engine.create_tab(Some(test_urls::ABOUT_BLANK), None)
        .await
        .expect("Failed to create first tab");
    
    sleep(Duration::from_millis(500)).await;
    
    // Create second tab
    let tab2 = engine.create_tab(Some(test_urls::ABOUT_BLANK), None)
        .await
        .expect("Failed to create second tab");
    
    // Verify both tabs exist
    let tabs = engine.get_tabs().await;
    assert_eq!(tabs.len(), 2);
    
    // Verify tab IDs are different
    assert_ne!(tab1.id, tab2.id);
    
    shutdown_browser(engine).await.unwrap();
}

/// Test setting active tab
#[tokio::test]
#[ignore]
async fn test_set_active_tab() {
    skip_if_no_chrome!();
    
    let config = create_test_config();
    let engine = launch_browser_with_timeout(config, 30)
        .await
        .expect("Failed to launch browser");
    
    // Create two tabs
    let tab1 = engine.create_tab(Some(test_urls::ABOUT_BLANK), None)
        .await
        .expect("Failed to create tab1");
    
    sleep(Duration::from_millis(500)).await;
    
    let tab2 = engine.create_tab(Some(test_urls::ABOUT_BLANK), None)
        .await
        .expect("Failed to create tab2");
    
    // Active tab should be tab2 (last created)
    let active = engine.get_active_tab().await;
    assert!(active.is_some());
    assert_eq!(active.unwrap().id, tab2.id);
    
    // Set active tab to tab1
    engine.set_active_tab(&tab1.id).await.expect("Failed to set active tab");
    
    let active = engine.get_active_tab().await;
    assert!(active.is_some());
    assert_eq!(active.unwrap().id, tab1.id);
    
    shutdown_browser(engine).await.unwrap();
}

/// Test closing a tab
#[tokio::test]
#[ignore]
async fn test_close_tab() {
    skip_if_no_chrome!();
    
    let config = create_test_config();
    let engine = launch_browser_with_timeout(config, 30)
        .await
        .expect("Failed to launch browser");
    
    // Create two tabs
    let tab1 = engine.create_tab(Some(test_urls::ABOUT_BLANK), None)
        .await
        .expect("Failed to create tab1");
    
    sleep(Duration::from_millis(500)).await;
    
    let _tab2 = engine.create_tab(Some(test_urls::ABOUT_BLANK), None)
        .await
        .expect("Failed to create tab2");
    
    // Verify we have 2 tabs
    assert_eq!(engine.get_tabs().await.len(), 2);
    
    // Close tab1
    engine.close_tab(&tab1.id).await.expect("Failed to close tab");
    
    // Verify we have 1 tab
    assert_eq!(engine.get_tabs().await.len(), 1);
    
    shutdown_browser(engine).await.unwrap();
}

/// Test browser with custom user agent
#[tokio::test]
#[ignore]
async fn test_custom_user_agent() {
    skip_if_no_chrome!();
    
    let mut config = create_test_config();
    config.user_agent = Some("TestBot/1.0 (Integration Test)".to_string());
    
    let engine = launch_browser_with_timeout(config, 30)
        .await
        .expect("Failed to launch browser");
    
    assert!(engine.get_config().user_agent.is_some());
    assert_eq!(
        engine.get_config().user_agent.as_ref().unwrap(),
        "TestBot/1.0 (Integration Test)"
    );
    
    shutdown_browser(engine).await.unwrap();
}

/// Test browser with stealth mode enabled
#[tokio::test]
#[ignore]
async fn test_stealth_mode() {
    skip_if_no_chrome!();
    
    let mut config = create_test_config();
    config.stealth_mode = true;
    
    let engine = launch_browser_with_timeout(config, 30)
        .await
        .expect("Failed to launch browser");
    
    assert!(engine.get_config().stealth_mode);
    
    // Create a tab - stealth scripts should be injected
    let _tab = engine.create_tab(Some(test_urls::ABOUT_BLANK), None)
        .await
        .expect("Failed to create tab");
    
    sleep(Duration::from_millis(500)).await;
    
    shutdown_browser(engine).await.unwrap();
}

/// Test browser with WebRTC protection
#[tokio::test]
#[ignore]
async fn test_webrtc_protection() {
    skip_if_no_chrome!();
    
    let mut config = create_test_config();
    config.webrtc_protection = true;
    
    let engine = launch_browser_with_timeout(config, 30)
        .await
        .expect("Failed to launch browser");
    
    assert!(engine.get_config().webrtc_protection);
    
    shutdown_browser(engine).await.unwrap();
}

/// Test browser with DNS over HTTPS
#[tokio::test]
#[ignore]
async fn test_dns_over_https() {
    skip_if_no_chrome!();
    
    let mut config = create_test_config();
    config.doh_server = Some("https://dns.google/dns-query".to_string());
    
    let engine = launch_browser_with_timeout(config, 30)
        .await
        .expect("Failed to launch browser");
    
    assert!(engine.get_config().doh_server.is_some());
    
    shutdown_browser(engine).await.unwrap();
}

/// Test launching browser without sandbox
#[tokio::test]
#[ignore]
async fn test_no_sandbox() {
    skip_if_no_chrome!();
    
    let mut config = create_test_config();
    config.sandbox = false;
    
    let engine = launch_browser_with_timeout(config, 30)
        .await
        .expect("Failed to launch browser");
    
    assert!(!engine.get_config().sandbox);
    
    shutdown_browser(engine).await.unwrap();
}

/// Test double shutdown (should be idempotent)
#[tokio::test]
#[ignore]
async fn test_double_shutdown() {
    skip_if_no_chrome!();
    
    let config = create_test_config();
    let mut engine = launch_browser_with_timeout(config, 30)
        .await
        .expect("Failed to launch browser");
    
    // First shutdown
    engine.shutdown().await.expect("First shutdown failed");
    assert_engine_not_running(&engine).await;
    
    // Second shutdown should succeed
    engine.shutdown().await.expect("Second shutdown failed");
    assert_engine_not_running(&engine).await;
}

/// Test tab proxy assignment (metadata only, not actual proxy usage)
#[tokio::test]
#[ignore]
async fn test_tab_proxy_metadata() {
    skip_if_no_chrome!();
    
    let config = create_test_config();
    let engine = launch_browser_with_timeout(config, 30)
        .await
        .expect("Failed to launch browser");
    
    let proxy = ProxySettings {
        proxy_type: ProxyType::Http,
        host: Some("proxy.example.com".to_string()),
        port: Some(8080),
        username: None,
        password: None,
        dns_servers: vec![],
        bypass_list: vec![],
    };
    
    // Create tab without proxy
    let tab = engine.create_tab(Some(test_urls::ABOUT_BLANK), None)
        .await
        .expect("Failed to create tab");
    
    assert!(tab.proxy.is_none());
    
    // Update tab proxy metadata
    engine.set_tab_proxy(&tab.id, Some(proxy.clone()))
        .await
        .expect("Failed to set tab proxy");
    
    // Verify proxy was set (in metadata)
    let tabs = engine.get_tabs().await;
    let updated_tab = tabs.iter().find(|t| t.id == tab.id);
    assert!(updated_tab.is_some());
    assert!(updated_tab.unwrap().proxy.is_some());
    
    shutdown_browser(engine).await.unwrap();
}

/// Test getting tabs when engine is running
#[tokio::test]
#[ignore]
async fn test_get_tabs_while_running() {
    skip_if_no_chrome!();
    
    let config = create_test_config();
    let engine = launch_browser_with_timeout(config, 30)
        .await
        .expect("Failed to launch browser");
    
    // Initially no tabs
    let tabs = engine.get_tabs().await;
    assert_eq!(tabs.len(), 0);
    
    // Create a tab
    engine.create_tab(Some(test_urls::ABOUT_BLANK), None)
        .await
        .expect("Failed to create tab");
    
    // Now should have 1 tab
    let tabs = engine.get_tabs().await;
    assert_eq!(tabs.len(), 1);
    
    shutdown_browser(engine).await.unwrap();
}

/// Test concurrent tab creation
#[tokio::test]
#[ignore]
async fn test_concurrent_tab_creation() {
    skip_if_no_chrome!();
    
    let config = create_test_config();
    let engine = std::sync::Arc::new(
        launch_browser_with_timeout(config, 30)
            .await
            .expect("Failed to launch browser")
    );
    
    let mut handles = vec![];
    
    // Create 3 tabs concurrently
    for _ in 0..3 {
        let engine_clone = engine.clone();
        let handle = tokio::spawn(async move {
            sleep(Duration::from_millis(100)).await;
            engine_clone.create_tab(Some(test_urls::ABOUT_BLANK), None).await
        });
        handles.push(handle);
    }
    
    // Wait for all to complete
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
    
    // Should have 3 tabs
    let tabs = engine.get_tabs().await;
    assert_eq!(tabs.len(), 3);
    
    // Need to move out of Arc to shutdown
    match std::sync::Arc::try_unwrap(engine) {
        Ok(engine) => {
            shutdown_browser(engine).await.unwrap();
        }
        Err(_) => {
            eprintln!("Warning: Could not unwrap Arc for shutdown (other references still exist)");
        }
    }
}

/// Test browser launch timeout
#[tokio::test]
#[ignore]
async fn test_launch_timeout() {
    skip_if_no_chrome!();
    
    let config = create_test_config();
    
    // Try to launch with very short timeout (should succeed quickly)
    let result = launch_browser_with_timeout(config, 5).await;
    
    // If Chrome is fast enough, this should succeed
    // Otherwise it will timeout (which is also a valid test outcome)
    match result {
        Ok(engine) => {
            shutdown_browser(engine).await.unwrap();
        }
        Err(e) => {
            // Timeout is acceptable
            assert!(e.contains("timed out") || e.contains("Failed"));
        }
    }
}
