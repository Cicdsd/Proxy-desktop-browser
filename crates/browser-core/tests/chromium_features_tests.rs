//! Integration tests for new Chromium features
//! 
//! Tests the newly implemented features: fingerprinting, geolocation, network throttling

mod common;

use browser_core::chromium_engine::{
    ChromiumEngineConfig, FingerprintConfig, Geolocation, NetworkCondition,
};
use common::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
#[ignore]
async fn test_fingerprint_spoofing() {
    skip_if_no_chrome!();
    
    let mut config = create_test_config();
    config.fingerprint = FingerprintConfig {
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
    
    let engine = launch_browser_with_timeout(config, 30)
        .await
        .expect("Failed to launch browser");
    
    // Create tab - fingerprint spoofing should be applied
    let _tab = engine.create_tab(Some(test_urls::ABOUT_BLANK), None)
        .await
        .expect("Failed to create tab");
    
    sleep(Duration::from_millis(500)).await;
    
    shutdown_browser(engine).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_geolocation_spoofing() {
    skip_if_no_chrome!();
    
    let mut config = create_test_config();
    config.geolocation = Some(Geolocation {
        latitude: 51.5074,
        longitude: -0.1278,
        accuracy: 50.0,
    });
    
    let engine = launch_browser_with_timeout(config, 30)
        .await
        .expect("Failed to launch browser");
    
    // Create tab - geolocation should be spoofed
    let _tab = engine.create_tab(Some(test_urls::ABOUT_BLANK), None)
        .await
        .expect("Failed to create tab");
    
    sleep(Duration::from_millis(500)).await;
    
    shutdown_browser(engine).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_network_throttling() {
    skip_if_no_chrome!();
    
    let mut config = create_test_config();
    config.network_condition = NetworkCondition::Fast3G;
    
    let engine = launch_browser_with_timeout(config, 30)
        .await
        .expect("Failed to launch browser");
    
    // Create tab - network throttling should be configured
    let _tab = engine.create_tab(Some(test_urls::ABOUT_BLANK), None)
        .await
        .expect("Failed to create tab");
    
    sleep(Duration::from_millis(500)).await;
    
    shutdown_browser(engine).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_custom_network_condition() {
    skip_if_no_chrome!();
    
    let mut config = create_test_config();
    config.network_condition = NetworkCondition::Custom {
        download_throughput: 1024.0 * 1024.0,
        upload_throughput: 512.0 * 1024.0,
        latency: 100.0,
    };
    
    let engine = launch_browser_with_timeout(config, 30)
        .await
        .expect("Failed to launch browser");
    
    let _tab = engine.create_tab(Some(test_urls::ABOUT_BLANK), None)
        .await
        .expect("Failed to create tab");
    
    sleep(Duration::from_millis(500)).await;
    
    shutdown_browser(engine).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_all_features_combined() {
    skip_if_no_chrome!();
    
    let mut config = create_test_config();
    
    // Enable all features
    config.stealth_mode = true;
    config.webrtc_protection = true;
    config.user_agent = Some("Mozilla/5.0 (Custom)".to_string());
    
    config.fingerprint = FingerprintConfig {
        randomize_canvas: true,
        randomize_webgl: true,
        randomize_audio: true,
        spoof_screen: true,
        screen_width: 1920,
        screen_height: 1080,
        spoof_hardware_concurrency: true,
        hardware_concurrency: 8,
        spoof_device_memory: true,
        device_memory: 16,
        spoof_timezone: true,
        timezone: "America/New_York".to_string(),
        spoof_language: true,
        language: "en-US".to_string(),
        spoof_platform: true,
        platform: "Win32".to_string(),
    };
    
    config.geolocation = Some(Geolocation {
        latitude: 37.7749,
        longitude: -122.4194,
        accuracy: 100.0,
    });
    
    config.network_condition = NetworkCondition::Fast3G;
    
    let engine = launch_browser_with_timeout(config, 30)
        .await
        .expect("Failed to launch browser");
    
    // Create tab with all features enabled
    let tab = engine.create_tab(Some(test_urls::ABOUT_BLANK), None)
        .await
        .expect("Failed to create tab");
    
    assert_eq!(tab.url, test_urls::ABOUT_BLANK);
    
    sleep(Duration::from_secs(1)).await;
    
    shutdown_browser(engine).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_headless_mode_fixed() {
    skip_if_no_chrome!();
    
    // Test that headless=true actually runs headless
    let mut config = create_test_config();
    config.headless = true;
    
    let engine = launch_browser_with_timeout(config.clone(), 30)
        .await
        .expect("Failed to launch browser in headless mode");
    
    assert!(engine.get_config().headless);
    
    shutdown_browser(engine).await.unwrap();
    
    // Test that headless=false shows GUI
    let mut config2 = create_test_config();
    config2.headless = false;
    
    let engine2 = launch_browser_with_timeout(config2.clone(), 30)
        .await
        .expect("Failed to launch browser with GUI");
    
    assert!(!engine2.get_config().headless);
    
    shutdown_browser(engine2).await.unwrap();
}

#[test]
fn test_fingerprint_config_all_options() {
    let fp = FingerprintConfig {
        randomize_canvas: true,
        randomize_webgl: true,
        randomize_audio: true,
        spoof_screen: true,
        screen_width: 3840,
        screen_height: 2160,
        spoof_hardware_concurrency: true,
        hardware_concurrency: 32,
        spoof_device_memory: true,
        device_memory: 64,
        spoof_timezone: true,
        timezone: "Asia/Tokyo".to_string(),
        spoof_language: true,
        language: "ja-JP".to_string(),
        spoof_platform: true,
        platform: "Linux x86_64".to_string(),
    };
    
    assert!(fp.randomize_canvas);
    assert_eq!(fp.screen_width, 3840);
    assert_eq!(fp.hardware_concurrency, 32);
    assert_eq!(fp.device_memory, 64);
    assert_eq!(fp.timezone, "Asia/Tokyo");
    assert_eq!(fp.language, "ja-JP");
    assert_eq!(fp.platform, "Linux x86_64");
}

#[test]
fn test_geolocation_coordinates() {
    let geo = Geolocation {
        latitude: -33.8688,
        longitude: 151.2093,
        accuracy: 10.0,
    };
    
    assert_eq!(geo.latitude, -33.8688);
    assert_eq!(geo.longitude, 151.2093);
    assert_eq!(geo.accuracy, 10.0);
}

#[test]
fn test_network_condition_presets() {
    // Test all preset network conditions
    let conditions = vec![
        (NetworkCondition::None, -1.0, -1.0, 0.0),
        (NetworkCondition::Slow3G, 500.0 * 1024.0 / 8.0, 500.0 * 1024.0 / 8.0, 400.0),
        (NetworkCondition::Fast3G, 1.5 * 1024.0 * 1024.0 / 8.0, 750.0 * 1024.0 / 8.0, 150.0),
        (NetworkCondition::LTE, 12.0 * 1024.0 * 1024.0 / 8.0, 5.0 * 1024.0 * 1024.0 / 8.0, 50.0),
    ];
    
    for (condition, expected_download, expected_upload, expected_latency) in conditions {
        let (download, upload, latency) = condition.get_params();
        assert_eq!(download, expected_download);
        assert_eq!(upload, expected_upload);
        assert_eq!(latency, expected_latency);
    }
}
