//! Browser Tab Manager Module
//!
//! Provides comprehensive browser tab management including:
//! - Tab creation with proxy and fingerprint configuration
//! - Tab navigation (back, forward, reload, stop)
//! - Tab switching and focus management
//! - IP rotation per tab
//! - Session isolation between tabs
//! - Browsing data management

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{info, debug, warn};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use tauri::State;

use crate::tab_manager::TabIPManager;
use crate::webview_manager::{WebviewManager, WebviewTab};
use crate::tab_isolation::TabProfile;
use crate::proxy::ProxySettings;
use virtual_ip::VirtualIP;
use virtual_ip::IPGenerator;
// Database removed - using in-memory storage


/// Combined browser tab information
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a BrowserTab.
pub struct BrowserTab {
    pub tab_id: String,
    pub profile: TabProfile,
    pub webview: WebviewTab,
    pub virtual_ip: VirtualIP,
    pub proxy_config: Option<ProxySettings>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

/// Configuration for creating a new browser tab
#[derive(Debug, Clone)]
/// Represents a CreateTabConfig.
pub struct CreateTabConfig {
    pub url: Option<String>,
    pub country_code: Option<String>,
    pub proxy_config: Option<ProxySettings>,
    pub user_agent: Option<String>,
    pub title: Option<String>,
    pub background: bool,
}

impl Default for CreateTabConfig {
    fn default() -> Self {
        Self {
            url: Some("https://www.google.com".to_string()),
            country_code: Some("US".to_string()),
            proxy_config: None,
            user_agent: None,
            title: None,
            background: false,
        }
    }
}

/// Manages browser tabs with both IP isolation and WebView functionality
pub struct BrowserTabManager {
    tab_ip_manager: Arc<TabIPManager>,
    webview_manager: Arc<WebviewManager>,
    tabs: Arc<RwLock<HashMap<String, BrowserTab>>>,
    active_tab: Arc<RwLock<Option<String>>>,
}

impl BrowserTabManager {
    /// Create a new BrowserTabManager
    pub fn new(
        ip_generator: IPGenerator,
        app_handle: tauri::AppHandle,
    ) -> Self {
        // Create the managers with in-memory storage
        let tab_ip_manager = Arc::new(TabIPManager::new(ip_generator));
        
        let webview_manager = Arc::new(
            WebviewManager::new(app_handle)
        );

        Self {
            tab_ip_manager,
            webview_manager,
            tabs: Arc::new(RwLock::new(HashMap::new())),
            active_tab: Arc::new(RwLock::new(None)),
        }
    }


    /// Create a new browser tab
    /// Create a new browser tab with the specified configuration
    ///
    /// # Arguments
    /// * `config` - Tab creation configuration
    pub async fn create_tab(&self, config: CreateTabConfig) -> Result<BrowserTab> {
        let tab_id = Uuid::new_v4().to_string();
        info!("Creating new browser tab: {}", tab_id);

        // Create the tab profile with IP isolation
        let country_code = config.country_code.as_deref().unwrap_or("US");
        let profile = self.tab_ip_manager
            .create_tab(country_code)
            .await
            .map_err(|e| anyhow!("Failed to create tab profile: {}", e))?;

        // Extract virtual IP from profile
        let virtual_ip = profile.virtual_ip.clone();

        // Create proxy config from virtual IP if not provided
        let proxy_config = if let Some(proxy) = config.proxy_config {
            Some(proxy)
        } else {
            self.create_proxy_from_virtual_ip(&virtual_ip)?
        };

        // Create the WebView
        let webview = self.webview_manager
            .create_tab(config.url.clone())
            .await
            .map_err(|e| anyhow!("Failed to create WebView: {}", e))?;

        // Create the combined browser tab
        let browser_tab = BrowserTab {
            tab_id: tab_id.clone(),
            profile,
            webview,
            virtual_ip,
            proxy_config,
            is_active: !config.background,
            created_at: Utc::now(),
            last_active: Utc::now(),
        };

        // Store the tab
        let mut tabs = self.tabs.write().await;
        tabs.insert(tab_id.clone(), browser_tab.clone());

        // Set as active if not background
        if !config.background {
            *self.active_tab.write().await = Some(tab_id.clone());
        }

        // Apply proxy configuration if needed
        if let Some(ref proxy) = browser_tab.proxy_config {
            self.apply_proxy_config(&tab_id, proxy).await?;
        }

        info!("Successfully created browser tab: {}", tab_id);
        Ok(browser_tab)
    }

    /// Close a browser tab
    /// Close a browser tab
    ///
    /// # Arguments
    /// * `tab_id` - ID of the tab to close
    pub async fn close_tab(&self, tab_id: &str) -> Result<()> {
        info!("Closing browser tab: {}", tab_id);

        let mut tabs = self.tabs.write().await;
        
        if let Some(tab) = tabs.remove(tab_id) {
            // Close the WebView
            if let Err(e) = self.webview_manager.close_tab(&tab.webview.tab_id).await {
                warn!("Failed to close WebView: {}", e);
            }

            // Close the tab profile
            if let Err(e) = self.tab_ip_manager.close_tab(&tab.profile.tab_id).await {
                warn!("Failed to close tab profile: {}", e);
            }

            // Update active tab if necessary
            let mut active_tab = self.active_tab.write().await;
            if *active_tab == Some(tab_id.to_string()) {
                *active_tab = tabs.keys().next().cloned();
            }

            info!("Successfully closed browser tab: {}", tab_id);
            Ok(())
        } else {
            Err(anyhow!("Tab not found: {}", tab_id))
        }
    }

    /// Switch to a specific tab
    /// Switch to a specific tab
    ///
    /// # Arguments
    /// * `tab_id` - ID of the tab to switch to
    pub async fn switch_to_tab(&self, tab_id: &str) -> Result<()> {
        debug!("Switching to browser tab: {}", tab_id);

        let tabs = self.tabs.read().await;
        
        if let Some(tab) = tabs.get(tab_id) {
            // Focus the WebView
            self.webview_manager.focus_tab(&tab.webview.tab_id).await?;
            
            // Update active tab
            *self.active_tab.write().await = Some(tab_id.to_string());
            
            // Update last active time
            drop(tabs);
            let mut tabs_mut = self.tabs.write().await;
            if let Some(tab_mut) = tabs_mut.get_mut(tab_id) {
                tab_mut.is_active = true;
                tab_mut.last_active = Utc::now();
            }

            info!("Successfully switched to browser tab: {}", tab_id);
            Ok(())
        } else {
            Err(anyhow!("Tab not found: {}", tab_id))
        }
    }

    /// Navigate a tab to a new URL
    /// Navigate a tab to a URL
    ///
    /// # Arguments
    /// * `tab_id` - ID of the tab
    /// * `url` - URL to navigate to
    pub async fn navigate(&self, tab_id: &str, url: &str) -> Result<()> {
        debug!("Navigating browser tab {} to: {}", tab_id, url);

        let tabs = self.tabs.read().await;
        
        if let Some(tab) = tabs.get(tab_id) {
            // Navigate the WebView
            self.webview_manager.navigate(&tab.webview.tab_id, url).await?;
            
            // Update the tab profile
            let _ = self.tab_ip_manager.navigate(&tab.profile.tab_id, url).await;
            
            Ok(())
        } else {
            Err(anyhow!("Tab not found: {}", tab_id))
        }
    }

    /// Go back in navigation history
    /// Navigate back in tab history
    ///
    /// # Arguments
    /// * `tab_id` - ID of the tab
    pub async fn go_back(&self, tab_id: &str) -> Result<()> {
        let tabs = self.tabs.read().await;
        
        if let Some(tab) = tabs.get(tab_id) {
            self.webview_manager.go_back(&tab.webview.tab_id).await?;
            Ok(())
        } else {
            Err(anyhow!("Tab not found: {}", tab_id))
        }
    }

    /// Go forward in navigation history
    /// Navigate forward in tab history
    ///
    /// # Arguments
    /// * `tab_id` - ID of the tab
    pub async fn go_forward(&self, tab_id: &str) -> Result<()> {
        let tabs = self.tabs.read().await;
        
        if let Some(tab) = tabs.get(tab_id) {
            self.webview_manager.go_forward(&tab.webview.tab_id).await?;
            Ok(())
        } else {
            Err(anyhow!("Tab not found: {}", tab_id))
        }
    }

    /// Reload the current page
    /// Reload the current page in a tab
    ///
    /// # Arguments
    /// * `tab_id` - ID of the tab
    pub async fn reload(&self, tab_id: &str) -> Result<()> {
        let tabs = self.tabs.read().await;
        
        if let Some(tab) = tabs.get(tab_id) {
            self.webview_manager.reload(&tab.webview.tab_id).await?;
            Ok(())
        } else {
            Err(anyhow!("Tab not found: {}", tab_id))
        }
    }

    /// Stop loading the current page
    /// Stop loading in a tab
    ///
    /// # Arguments
    /// * `tab_id` - ID of the tab
    pub async fn stop(&self, tab_id: &str) -> Result<()> {
        let tabs = self.tabs.read().await;
        
        if let Some(tab) = tabs.get(tab_id) {
            self.webview_manager.stop(&tab.webview.tab_id).await?;
            Ok(())
        } else {
            Err(anyhow!("Tab not found: {}", tab_id))
        }
    }

    /// Set zoom level for a tab
    /// Set the zoom level for a tab
    ///
    /// # Arguments
    /// * `tab_id` - ID of the tab
    /// * `level` - Zoom level (1.0 = 100%)
    pub async fn set_zoom(&self, tab_id: &str, level: f64) -> Result<()> {
        let tabs = self.tabs.read().await;
        
        if let Some(tab) = tabs.get(tab_id) {
            self.webview_manager.set_zoom(&tab.webview.tab_id, level).await?;
            Ok(())
        } else {
            Err(anyhow!("Tab not found: {}", tab_id))
        }
    }

    /// Rotate IP for a tab
    /// Rotate the IP for a tab
    ///
    /// # Arguments
    /// * `tab_id` - ID of the tab
    /// * `country_code` - Optional country code for geo-targeting
    pub async fn rotate_ip(&self, tab_id: &str, country_code: Option<&str>) -> Result<VirtualIP> {
        debug!("Rotating IP for browser tab: {}", tab_id);

        let tabs = self.tabs.read().await;
        
        if let Some(tab) = tabs.get(tab_id) {
            // Rotate IP in the tab profile
            let new_ip = self.tab_ip_manager
                .rotate_ip(&tab.profile.tab_id, country_code)
                .await?;
            
            // Create new proxy config
            if let Some(proxy) = self.create_proxy_from_virtual_ip(&new_ip)? {
                // Apply the new proxy config
                drop(tabs);
                let mut tabs_mut = self.tabs.write().await;
                if let Some(tab_mut) = tabs_mut.get_mut(tab_id) {
                    tab_mut.virtual_ip = new_ip.clone();
                    tab_mut.proxy_config = Some(proxy.clone());
                }
                
                self.apply_proxy_config(tab_id, &proxy).await?;
            }
            
            info!("Successfully rotated IP for browser tab: {}", tab_id);
            Ok(new_ip)
        } else {
            Err(anyhow!("Tab not found: {}", tab_id))
        }
    }

    /// Get all browser tabs
    /// Get all open tabs
    pub async fn get_tabs(&self) -> Vec<BrowserTab> {
        self.tabs.read().await.values().cloned().collect()
    }

    /// Get a specific browser tab
    /// Get a specific tab by ID
    ///
    /// # Arguments
    /// * `tab_id` - ID of the tab
    pub async fn get_tab(&self, tab_id: &str) -> Option<BrowserTab> {
        self.tabs.read().await.get(tab_id).cloned()
    }

    /// Get the currently active tab
    /// Get the currently active tab
    pub async fn get_active_tab(&self) -> Option<BrowserTab> {
        let active_id = self.active_tab.read().await.clone()?;
        self.tabs.read().await.get(&active_id).cloned()
    }

    /// Execute JavaScript in a tab
    /// Execute JavaScript in a tab
    ///
    /// # Arguments
    /// * `tab_id` - ID of the tab
    /// * `script` - JavaScript code to execute
    pub async fn execute_script(&self, tab_id: &str, script: &str) -> Result<()> {
        let tabs = self.tabs.read().await;
        
        if let Some(tab) = tabs.get(tab_id) {
            self.webview_manager.execute_script(&tab.webview.tab_id, script).await?;
            Ok(())
        } else {
            Err(anyhow!("Tab not found: {}", tab_id))
        }
    }

    /// Update tab state from WebView events
    pub async fn update_webview_state(
        &self,
        webview_tab_id: &str,
        url: Option<String>,
        title: Option<String>,
        is_loading: Option<bool>,
    ) -> Result<()> {
        // Find the browser tab by webview ID
        let tabs = self.tabs.read().await;
        let target_tab_id = tabs
            .iter()
            .find(|(_, tab)| tab.webview.tab_id == webview_tab_id)
            .map(|(id, _)| id.clone());
        
        if let Some(tab_id) = target_tab_id {
            drop(tabs);
            let mut tabs_mut = self.tabs.write().await;
            
            if let Some(tab) = tabs_mut.get_mut(&tab_id) {
                if let Some(url) = url {
                    tab.webview.url = url;
                }
                if let Some(title) = title {
                    tab.webview.title = title;
                }
                if let Some(is_loading) = is_loading {
                    tab.webview.is_loading = is_loading;
                }
            }
            
            Ok(())
        } else {
            Err(anyhow!("WebView tab not found: {}", webview_tab_id))
        }
    }

    /// Create proxy configuration from virtual IP
    fn create_proxy_from_virtual_ip(&self, _virtual_ip: &VirtualIP) -> Result<Option<ProxySettings>> {
        // For now, return None - in a real implementation, you'd create
        // a proxy config that routes through the virtual IP
        Ok(None)
    }

    /// Apply proxy configuration to a tab
    async fn apply_proxy_config(&self, tab_id: &str, proxy: &ProxySettings) -> Result<()> {
        debug!("Applying proxy config to tab {}: {:?}", tab_id, proxy);
        
        // In a real implementation, you'd configure the WebView to use the proxy
        // This might involve injecting PAC scripts or configuring system proxy
        
        Ok(())
    }

    /// Clear browsing data for a tab
    /// Clear browsing data for a tab
    ///
    /// # Arguments
    /// * `tab_id` - ID of the tab
    /// * `data_types` - Types of data to clear (cookies, cache, history, etc.)
    pub async fn clear_browsing_data(&self, tab_id: &str, data_types: &[&str]) -> Result<()> {
        let tabs = self.tabs.read().await;
        
        if let Some(tab) = tabs.get(tab_id) {
            // Clear data from the WebView
            for data_type in data_types {
                match *data_type {
                    "cookies" => {
                        // Clear cookies
                        let script = r#"
                            document.cookie.split(";").forEach(function(c) {
                                document.cookie = c.replace(/^ +/, "").replace(/=.*/, "=;expires=" + new Date().toUTCString() + ";path=/");
                            });
                        "#;
                        let _ = self.webview_manager.execute_script(&tab.webview.tab_id, script).await;
                    }
                    "localStorage" => {
                        // Clear localStorage
                        let script = "localStorage.clear();";
                        let _ = self.webview_manager.execute_script(&tab.webview.tab_id, script).await;
                    }
                    "sessionStorage" => {
                        // Clear sessionStorage
                        let script = "sessionStorage.clear();";
                        let _ = self.webview_manager.execute_script(&tab.webview.tab_id, script).await;
                    }
                    "cache" => {
                        // Clear cache (limited via JavaScript)
                        let script = r#"
                            if (window.caches) {
                                caches.keys().then(function(names) {
                                    names.forEach(function(name) {
                                        caches.delete(name);
                                    });
                                });
                            }
                        "#;
                        let _ = self.webview_manager.execute_script(&tab.webview.tab_id, script).await;
                    }
                    _ => {
                        warn!("Unknown data type for clearing: {}", data_type);
                    }
                }
            }
            
            Ok(())
        } else {
            Err(anyhow!("Tab not found: {}", tab_id))
        }
    }

    /// Get tab statistics
    /// Get tab statistics
    pub async fn get_stats(&self) -> TabStats {
        let tabs = self.tabs.read().await;
        let active_tab_id = self.active_tab.read().await.clone();
        
        TabStats {
            total_tabs: tabs.len(),
            active_tabs: tabs.values().filter(|t| t.is_active).count(),
            current_tab: active_tab_id,
            tabs_by_country: {
                let mut map = HashMap::new();
                for tab in tabs.values() {
                    let country = &tab.virtual_ip.country_code;
                    *map.entry(country.clone()).or_insert(0) += 1;
                }
                map
            },
        }
    }
}

/// Statistics about browser tabs
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents a TabStats.
pub struct TabStats {
    pub total_tabs: usize,
    pub active_tabs: usize,
    pub current_tab: Option<String>,
    pub tabs_by_country: HashMap<String, usize>,
}

// Tauri command handlers for BrowserTabManager
#[tauri::command]
/// Creates a new browser tab.
pub async fn create_browser_tab(
    manager: State<'_, Arc<BrowserTabManager>>,
    url: Option<String>,
    country_code: Option<String>,
    background: Option<bool>,
) -> Result<BrowserTab, String> {
    let config = CreateTabConfig {
        url,
        country_code,
        background: background.unwrap_or(false),
        ..Default::default()
    };
    
    manager.create_tab(config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
/// Closes browser tab.
pub async fn close_browser_tab(
    manager: State<'_, Arc<BrowserTabManager>>,
    tab_id: String,
) -> Result<(), String> {
    manager.close_tab(&tab_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
/// Performs switch browser tab operation.
pub async fn switch_browser_tab(
    manager: State<'_, Arc<BrowserTabManager>>,
    tab_id: String,
) -> Result<(), String> {
    manager.switch_to_tab(&tab_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
/// Performs navigate browser tab operation.
pub async fn navigate_browser_tab(
    manager: State<'_, Arc<BrowserTabManager>>,
    tab_id: String,
    url: String,
) -> Result<(), String> {
    manager.navigate(&tab_id, &url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
/// Performs browser tab go back operation.
pub async fn browser_tab_go_back(
    manager: State<'_, Arc<BrowserTabManager>>,
    tab_id: String,
) -> Result<(), String> {
    manager.go_back(&tab_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
/// Performs browser tab go forward operation.
pub async fn browser_tab_go_forward(
    manager: State<'_, Arc<BrowserTabManager>>,
    tab_id: String,
) -> Result<(), String> {
    manager.go_forward(&tab_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
/// Reloads browser tab.
pub async fn reload_browser_tab(
    manager: State<'_, Arc<BrowserTabManager>>,
    tab_id: String,
) -> Result<(), String> {
    manager.reload(&tab_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
/// Stops the browser tab.
pub async fn stop_browser_tab(
    manager: State<'_, Arc<BrowserTabManager>>,
    tab_id: String,
) -> Result<(), String> {
    manager.stop(&tab_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
/// Sets the browser tab zoom.
pub async fn set_browser_tab_zoom(
    manager: State<'_, Arc<BrowserTabManager>>,
    tab_id: String,
    level: f64,
) -> Result<(), String> {
    manager.set_zoom(&tab_id, level)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
/// Rotates browser tab ip.
pub async fn rotate_browser_tab_ip(
    manager: State<'_, Arc<BrowserTabManager>>,
    tab_id: String,
    country_code: Option<String>,
) -> Result<VirtualIP, String> {
    manager.rotate_ip(&tab_id, country_code.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
/// Gets the browser tabs.
pub async fn get_browser_tabs(
    manager: State<'_, Arc<BrowserTabManager>>,
) -> Result<Vec<BrowserTab>, String> {
    Ok(manager.get_tabs().await)
}

#[tauri::command]
/// Gets the browser tab.
pub async fn get_browser_tab(
    manager: State<'_, Arc<BrowserTabManager>>,
    tab_id: String,
) -> Result<Option<BrowserTab>, String> {
    Ok(manager.get_tab(&tab_id).await)
}

#[tauri::command]
/// Gets the active browser tab.
pub async fn get_active_browser_tab(
    manager: State<'_, Arc<BrowserTabManager>>,
) -> Result<Option<BrowserTab>, String> {
    Ok(manager.get_active_tab().await)
}

#[tauri::command]
/// Executes script in browser tab.
pub async fn execute_script_in_browser_tab(
    manager: State<'_, Arc<BrowserTabManager>>,
    tab_id: String,
    script: String,
) -> Result<(), String> {
    manager.execute_script(&tab_id, &script)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
/// Clears browser tab data.
pub async fn clear_browser_tab_data(
    manager: State<'_, Arc<BrowserTabManager>>,
    tab_id: String,
    data_types: Vec<String>,
) -> Result<(), String> {
    let types: Vec<&str> = data_types.iter().map(|s| s.as_str()).collect();
    manager.clear_browsing_data(&tab_id, &types)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
/// Gets the browser tab stats.
pub async fn get_browser_tab_stats(
    manager: State<'_, Arc<BrowserTabManager>>,
) -> Result<TabStats, String> {
    Ok(manager.get_stats().await)
}

#[tauri::command]
/// Updates the webview tab state.
pub async fn update_webview_tab_state(
    manager: State<'_, Arc<BrowserTabManager>>,
    webview_tab_id: String,
    url: Option<String>,
    title: Option<String>,
    is_loading: Option<bool>,
) -> Result<(), String> {
    manager.update_webview_state(&webview_tab_id, url, title, is_loading)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::test::mock_app;
    use virtual_ip::IPGenerator;

    #[tokio::test]
    async fn test_browser_tab_manager_creation() {
        let app = mock_app();
        let ip_generator = IPGenerator::new(vec![], vec![]);
        
        let manager = BrowserTabManager::new(
            ip_generator,
            app.handle(),
        );
        
        // Manager should be created successfully
        assert!(manager.tabs.read().await.is_empty());
    }


    #[tokio::test]
    async fn test_create_tab_config_default() {
        let config = CreateTabConfig::default();
        
        assert_eq!(config.url, Some("https://www.google.com".to_string()));
        assert_eq!(config.country_code, Some("US".to_string()));
        assert_eq!(config.proxy_config, None);
        assert_eq!(config.background, false);
    }
}
