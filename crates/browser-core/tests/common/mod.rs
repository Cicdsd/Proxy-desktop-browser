//! Common test utilities for integration tests
//! 
//! This module provides helper functions and utilities for testing
//! the Chromium engine with real browser instances.

use browser_core::chromium_engine::{ChromiumEngine, ChromiumEngineConfig};
use std::time::Duration;
use tokio::time::timeout;

/// Helper to create a test-friendly Chromium engine configuration
pub fn create_test_config() -> ChromiumEngineConfig {
    ChromiumEngineConfig {
        headless: true, // Run headless for tests
        sandbox: false, // Disable sandbox for test environments
        viewport_width: 1280,
        viewport_height: 720,
        stealth_mode: true,
        extra_args: vec![
            "--disable-gpu".to_string(),
            "--no-first-run".to_string(),
            "--disable-dev-shm-usage".to_string(),
        ],
        ..Default::default()
    }
}

/// Helper to launch a browser with timeout
pub async fn launch_browser_with_timeout(
    config: ChromiumEngineConfig,
    timeout_secs: u64,
) -> Result<ChromiumEngine, String> {
    let mut engine = ChromiumEngine::new(config);
    
    match timeout(Duration::from_secs(timeout_secs), engine.launch()).await {
        Ok(Ok(())) => Ok(engine),
        Ok(Err(e)) => Err(format!("Failed to launch browser: {}", e)),
        Err(_) => Err(format!("Browser launch timed out after {} seconds", timeout_secs)),
    }
}

/// Helper to safely shutdown browser
pub async fn shutdown_browser(mut engine: ChromiumEngine) -> Result<(), String> {
    match timeout(Duration::from_secs(10), engine.shutdown()).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(format!("Failed to shutdown browser: {}", e)),
        Err(_) => Err("Browser shutdown timed out".to_string()),
    }
}

/// Check if Chrome/Chromium is available on the system
pub fn is_chrome_available() -> bool {
    // Check common Chrome/Chromium locations
    let paths = if cfg!(target_os = "windows") {
        vec![
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files\Chromium\Application\chrome.exe",
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
        ]
    } else {
        vec![
            "/usr/bin/google-chrome",
            "/usr/bin/chromium",
            "/usr/bin/chromium-browser",
            "/snap/bin/chromium",
        ]
    };
    
    paths.iter().any(|p| std::path::Path::new(p).exists())
}

/// Skip test if Chrome is not available
#[macro_export]
macro_rules! skip_if_no_chrome {
    () => {
        if !common::is_chrome_available() {
            eprintln!("Skipping test: Chrome/Chromium not found on system");
            return;
        }
    };
}

/// Test URLs for integration tests
pub mod test_urls {
    pub const EXAMPLE_COM: &str = "https://example.com";
    pub const HTTPBIN_IP: &str = "https://httpbin.org/ip";
    pub const HTTPBIN_USER_AGENT: &str = "https://httpbin.org/user-agent";
    pub const HTTPBIN_HEADERS: &str = "https://httpbin.org/headers";
    pub const ABOUT_BLANK: &str = "about:blank";
}

/// Assert that engine is running
pub async fn assert_engine_running(engine: &ChromiumEngine) {
    assert!(
        engine.is_running().await,
        "Engine should be running"
    );
}

/// Assert that engine is not running
pub async fn assert_engine_not_running(engine: &ChromiumEngine) {
    assert!(
        !engine.is_running().await,
        "Engine should not be running"
    );
}

/// Wait for a condition with timeout
pub async fn wait_for<F, Fut>(
    condition: F,
    timeout_secs: u64,
    check_interval_ms: u64,
) -> Result<(), String>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = std::time::Instant::now();
    let timeout_duration = Duration::from_secs(timeout_secs);
    let check_interval = Duration::from_millis(check_interval_ms);
    
    loop {
        if condition().await {
            return Ok(());
        }
        
        if start.elapsed() >= timeout_duration {
            return Err(format!("Condition not met within {} seconds", timeout_secs));
        }
        
        tokio::time::sleep(check_interval).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_config() {
        let config = create_test_config();
        assert!(config.headless);
        assert!(!config.sandbox);
        assert_eq!(config.viewport_width, 1280);
        assert_eq!(config.viewport_height, 720);
    }

    #[test]
    fn test_test_urls() {
        assert_eq!(test_urls::EXAMPLE_COM, "https://example.com");
        assert_eq!(test_urls::ABOUT_BLANK, "about:blank");
    }

    #[test]
    fn test_is_chrome_available() {
        // Just ensure it doesn't panic
        let _ = is_chrome_available();
    }
}
