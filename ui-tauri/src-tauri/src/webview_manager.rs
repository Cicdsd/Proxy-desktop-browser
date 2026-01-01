use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Manager, WebviewWindow};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebviewTab {
    pub tab_id: String,
    pub window_label: String,
    pub url: String,
    pub title: String,
    pub is_loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub created_at: std::time::SystemTime,
}

pub struct WebviewManager {
    app_handle: AppHandle,
    tabs: Arc<RwLock<HashMap<String, WebviewTab>>>,
    window_counter: RwLock<u32>,
}

impl WebviewManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            tabs: Arc::new(RwLock::new(HashMap::new())),
            window_counter: RwLock::new(0),
        }
    }

    /// Create a new webview tab with native window and proxy settings
    pub fn create_tab_with_proxy_sync(&self, initial_url: Option<String>, proxy_url: Option<String>, window_label: String, tab_id: String) -> Result<(WebviewTab, WebviewWindow)> {
        let url = initial_url.unwrap_or_else(|| "https://www.google.com".to_string());
        
        // Apply proxy settings if provided (environment variables for now)
        let title = if let Some(ref proxy) = proxy_url {
            // For now, use environment variables (affects all windows)
            // NOTE: Per-webview proxy configuration requires WebView2 environment setup
            // which must be done before the WebView is created. Current implementation
            // uses environment variables as a fallback. For true per-tab isolation,
            // consider using the IntegratedChromium engine from browser-core crate
            // which provides full CDP-based proxy control per browser context.

            if cfg!(target_os = "windows") {
                std::env::set_var("HTTP_PROXY", proxy);
                std::env::set_var("HTTPS_PROXY", proxy);
            }
            format!("New Tab - Virtual IP Browser ({})", proxy)
        } else {
            "New Tab - Virtual IP Browser".to_string()
        };
        
        // Create new webview window with Tauri v2 API
        let window = tauri::WebviewWindowBuilder::new(
            &self.app_handle,
            &window_label,
            tauri::WebviewUrl::External(url.parse()?),
        )
        .title(&title)
        .inner_size(1200.0, 800.0)
        .min_inner_size(400.0, 300.0)
        .center()
        .decorations(true)
        .resizable(true)
        .build()?;
        
        let tab = WebviewTab {
            tab_id: tab_id.clone(),
            window_label: window_label.clone(),
            url: url.clone(),
            title: "New Tab".to_string(),
            is_loading: false,
            can_go_back: false,
            can_go_forward: false,
            created_at: std::time::SystemTime::now(),
        };
        
        Ok((tab, window))
    }

    /// Create a new webview tab with native window and proxy settings
    pub async fn create_tab_with_proxy(&self, initial_url: Option<String>, proxy_url: Option<String>) -> Result<WebviewTab> {
        let tab_id = Uuid::new_v4().to_string();
        let counter = {
            let mut c = self.window_counter.write().await;
            *c += 1;
            *c
        };
        let window_label = format!("tab_{}", counter);
        
        // Do synchronous window creation first
        let (tab, _window) = self.create_tab_with_proxy_sync(initial_url, proxy_url, window_label, tab_id.clone())?;
        
        // Store tab reference
        self.tabs.write().await.insert(tab_id.clone(), tab.clone());
        
        Ok(tab)
    }

    /// Navigate a tab to a new URL
    pub async fn navigate(&self, tab_id: &str, url: &str) -> Result<()> {
        let tabs = self.tabs.read().await;
        let tab = tabs.get(tab_id).ok_or_else(|| anyhow!("Tab not found"))?;
        
        if let Some(window) = self.app_handle.get_webview_window(&tab.window_label) {
            window.eval(&format!("window.location.href = '{}';", url))?;
            
            // Update tab info
            drop(tabs);
            let mut tabs = self.tabs.write().await;
            if let Some(tab) = tabs.get_mut(tab_id) {
                tab.url = url.to_string();
                tab.is_loading = true;
            }
        }
        
        Ok(())
    }

    /// Get all tabs
    pub async fn list_tabs(&self) -> Vec<WebviewTab> {
        self.tabs.read().await.values().cloned().collect()
    }

    /// Close a tab
    pub async fn close_tab(&self, tab_id: &str) -> Result<()> {
        let tabs = self.tabs.read().await;
        let tab = tabs.get(tab_id).ok_or_else(|| anyhow!("Tab not found"))?;
        
        if let Some(window) = self.app_handle.get_webview_window(&tab.window_label) {
            window.close()?;
        }
        
        drop(tabs);
        self.tabs.write().await.remove(tab_id);
        
        Ok(())
    }

    /// Focus a tab's window
    pub async fn focus_tab(&self, tab_id: &str) -> Result<()> {
        let tabs = self.tabs.read().await;
        let tab = tabs.get(tab_id).ok_or_else(|| anyhow!("Tab not found"))?;
        
        if let Some(window) = self.app_handle.get_webview_window(&tab.window_label) {
            window.set_focus()?;
            window.unminimize()?;
        }
        
        Ok(())
    }

    /// Get tab by ID
    #[allow(dead_code)]
    pub async fn get_tab(&self, tab_id: &str) -> Option<WebviewTab> {
        self.tabs.read().await.get(tab_id).cloned()
    }

    /// Update tab title
    pub async fn update_tab_title(&self, tab_id: &str, title: String) -> Result<()> {
        let mut tabs = self.tabs.write().await;
        if let Some(tab) = tabs.get_mut(tab_id) {
            tab.title = title;
        }
        Ok(())
    }
}

// Tauri command handlers
#[tauri::command]
pub async fn create_webview_tab_with_proxy(
    app_handle: tauri::AppHandle,
    url: Option<String>,
    proxy_url: Option<String>
) -> Result<WebviewTab, String> {
    let manager = app_handle.state::<WebviewManager>();
    manager.create_tab_with_proxy(url, proxy_url).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_webview_tab(app_handle: tauri::AppHandle, url: Option<String>) -> Result<WebviewTab, String> {
    let manager = app_handle.state::<WebviewManager>();
    manager.create_tab_with_proxy(url, None).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn navigate_webview_tab(app_handle: tauri::AppHandle, tab_id: String, url: String) -> Result<(), String> {
    let manager = app_handle.state::<WebviewManager>();
    manager.navigate(&tab_id, &url).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn close_webview_tab(app_handle: tauri::AppHandle, tab_id: String) -> Result<(), String> {
    let manager = app_handle.state::<WebviewManager>();
    manager.close_tab(&tab_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn focus_webview_tab(app_handle: tauri::AppHandle, tab_id: String) -> Result<(), String> {
    let manager = app_handle.state::<WebviewManager>();
    manager.focus_tab(&tab_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_webview_tabs(app_handle: tauri::AppHandle) -> Result<Vec<WebviewTab>, String> {
    let manager = app_handle.state::<WebviewManager>();
    Ok(manager.list_tabs().await)
}

#[tauri::command]
pub async fn navigation_changed(
    app_handle: tauri::AppHandle,
    tab_id: String,
    url: String,
    title: String,
    can_go_back: bool,
    can_go_forward: bool
) -> Result<(), String> {
    let manager = app_handle.state::<WebviewManager>();
    let mut tabs = manager.tabs.write().await;
    if let Some(tab) = tabs.get_mut(&tab_id) {
        tab.url = url;
        tab.title = title;
        tab.can_go_back = can_go_back;
        tab.can_go_forward = can_go_forward;
        tab.is_loading = false;
    }
    Ok(())
}

#[tauri::command]
pub async fn title_changed(
    app_handle: tauri::AppHandle,
    tab_id: String,
    title: String
) -> Result<(), String> {
    let manager = app_handle.state::<WebviewManager>();
    manager.update_tab_title(&tab_id, title).await.map_err(|e| e.to_string())
}
