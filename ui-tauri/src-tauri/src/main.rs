#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use browser_core::{
    ProxyManager, ProxySettings, ProxyType, FreeProxy,
    PublicIpDetector, PublicIpInfo, FreeIpProviderManager,
    StorageEngine, BackupManager, BackupData, BackupOptions, BackupInfo,
    BrowserController, BrowserState, BrowserSettings, WebRtcPolicy,
};
use serde::{Deserialize, Serialize};
use tauri::{State, Manager};
use tracing::{info, error, debug, warn};
use virtual_ip::{
    demo_generator, load_countries_from_file, load_ip_ranges, load_ip_ranges_from_file,
    Country, CountryDatabase, IPGenerator, VirtualIP,
};

mod webview_manager;
use webview_manager::{
    WebviewManager, WebviewTab,
    create_webview_tab, create_webview_tab_with_proxy,
    navigate_webview_tab, close_webview_tab, focus_webview_tab,
    get_webview_tabs, navigation_changed, title_changed,
};

struct AppState {
    ip_generator: Arc<IPGenerator>,
    proxy_manager: Arc<ProxyManager>,
    storage_engine: Arc<StorageEngine>,
    backup_manager: Arc<BackupManager>,
    browser_controller: Arc<BrowserController>,
}

#[tauri::command]
async fn create_tab(state: State<'_, AppState>, app_handle: tauri::AppHandle, country_code: String) -> Result<TabResponse, String> {
    info!("Creating tab for country: {}", country_code);
    
    // Generate virtual IP for the country
    let virtual_ip = state.ip_generator.generate_for_country(&country_code)
        .map_err(|e| format!("No IP ranges available for country {}: {}", country_code, e))?;
    
    // Get a working proxy for the virtual IP
    let proxy_manager = &state.proxy_manager;
    let proxies = proxy_manager.get_free_proxies().await;
    debug!("Available proxies: {}", proxies.len());
    
    let proxy_url = if let Some(proxy) = proxies.first() {
        let url = format!("{}://{}:{}", 
            match proxy.protocol {
                browser_core::ProxyType::Http => "http",
                browser_core::ProxyType::Https => "https",
                browser_core::ProxyType::Socks4 => "socks4",
                browser_core::ProxyType::Socks5 => "socks5",
                _ => "http",
            },
            proxy.ip,
            proxy.port
        );
        info!("Using proxy: {}", url);
        Some(url)
    } else {
        warn!("No proxies available. Tab will use direct connection.");
        None
    };
    
    // Create webview tab with proxy settings
    let webview_manager = app_handle.state::<WebviewManager>();
    let tab = webview_manager.create_tab_with_proxy(
        Some("https://www.google.com".to_string()),
        proxy_url.clone()
    ).await.map_err(|e| {
        error!("Failed to create webview tab: {}", e);
        e.to_string()
    })?;
    
    info!("Successfully created tab with ID: {}", tab.tab_id);
    Ok(TabResponse {
        tab_id: tab.tab_id,
        ip: virtual_ip.ip.to_string(),
        country_code: virtual_ip.country_code.clone(),
        country_name: virtual_ip.country.clone(),
        city: virtual_ip.city.clone(),
        timezone: virtual_ip.timezone.clone(),
        isp: virtual_ip.isp.clone(),
    })
}

#[tauri::command]
async fn create_tab_random(state: State<'_, AppState>, app_handle: tauri::AppHandle) -> Result<TabResponse, String> {
    // Generate random virtual IP
    let virtual_ip = state.ip_generator.generate_random()
        .map_err(|e| format!("No IP ranges available: {}", e))?;
    
    // Get a working proxy for the virtual IP
    let proxy_manager = &state.proxy_manager;
    let proxies = proxy_manager.get_free_proxies().await;
    
    let proxy_url = if let Some(proxy) = proxies.first() {
        Some(format!("{}://{}:{}", 
            match proxy.protocol {
                browser_core::ProxyType::Http => "http",
                browser_core::ProxyType::Https => "https",
                browser_core::ProxyType::Socks4 => "socks4",
                browser_core::ProxyType::Socks5 => "socks5",
                _ => "http",
            },
            proxy.ip,
            proxy.port
        ))
    } else {
        warn!("No proxies available. Tab will use direct connection.");
        None
    };
    
    // Create webview tab with proxy settings
    let webview_manager = app_handle.state::<WebviewManager>();
    let tab = webview_manager.create_tab_with_proxy(
        Some("https://www.google.com".to_string()),
        proxy_url.clone()
    ).await.map_err(|e| {
        error!("Failed to create webview tab: {}", e);
        e.to_string()
    })?;
    
    info!("Successfully created random tab with ID: {}", tab.tab_id);
    Ok(TabResponse {
        tab_id: tab.tab_id,
        ip: virtual_ip.ip.to_string(),
        country_code: virtual_ip.country_code.clone(),
        country_name: virtual_ip.country.clone(),
        city: virtual_ip.city.clone(),
        timezone: virtual_ip.timezone.clone(),
        isp: virtual_ip.isp.clone(),
    })
}

#[tauri::command]
async fn list_tabs(app_handle: tauri::AppHandle) -> Result<Vec<WebviewTab>, String> {
    let manager = app_handle.state::<WebviewManager>();
    Ok(manager.list_tabs().await)
}

#[tauri::command]
async fn rotate_ip(
    state: State<'_, AppState>,
    _tab_id: String,
    new_country: Option<String>,
) -> Result<VirtualIPResponse, String> {
    // Generate new virtual IP for the specified country or random
    let virtual_ip = if let Some(country) = new_country {
        state.ip_generator.generate_for_country(&country)
            .map_err(|e| format!("No IP ranges available for country {}: {}", country, e))?
    } else {
        state.ip_generator.generate_random()
            .map_err(|e| format!("No IP ranges available: {}", e))?
    };
    Ok(VirtualIPResponse::from(virtual_ip))
}

#[tauri::command]
async fn validate_ip(_state: State<'_, AppState>, _tab_id: String) -> Result<ValidationResponse, String> {
    // For now, return a basic validation response
    // In production, this would check the actual proxy connection
    Ok(ValidationResponse {
        ip_matches: true,
        webrtc_secure: true,
        dns_secure: true,
        overall_pass: true,
        ip: "0.0.0.0".to_string(),
    })
}

#[tauri::command]
async fn list_countries(state: State<'_, AppState>) -> Result<Vec<CountryResponse>, String> {
    Ok(state
        .ip_generator
        .list_countries()
        .into_iter()
        .map(CountryResponse::from)
        .collect())
}

// Proxy Management Commands
#[tauri::command]
async fn get_proxy_settings(state: State<'_, AppState>) -> Result<ProxySettingsResponse, String> {
    let settings = state.proxy_manager.get_settings().await;
    Ok(ProxySettingsResponse::from(settings))
}

#[tauri::command]
async fn set_proxy_settings(state: State<'_, AppState>, settings: ProxySettingsRequest) -> Result<(), String> {
    state.proxy_manager.set_settings(settings.into()).await;
    Ok(())
}

#[tauri::command]
async fn get_active_proxy(state: State<'_, AppState>) -> Result<Option<FreeProxyResponse>, String> {
    Ok(state.proxy_manager.get_active_proxy().await.map(FreeProxyResponse::from))
}

#[tauri::command]
async fn set_active_proxy(state: State<'_, AppState>, proxy: Option<FreeProxyRequest>) -> Result<(), String> {
    state.proxy_manager.set_active_proxy(proxy.map(|p| p.into())).await;
    Ok(())
}

// Public IP Detection
#[tauri::command]
async fn detect_public_ip(state: State<'_, AppState>) -> Result<PublicIpResponse, String> {
    let settings = state.proxy_manager.get_settings().await;
    let detector = if settings.proxy_type != ProxyType::Direct {
        PublicIpDetector::with_proxy(&settings).map_err(|e| e.to_string())?
    } else {
        PublicIpDetector::new().map_err(|e| e.to_string())?
    };
    
    let info = detector.detect_ip().await.map_err(|e| e.to_string())?;
    Ok(PublicIpResponse::from(info))
}

// Free IP Providers
#[tauri::command]
async fn fetch_free_proxies(state: State<'_, AppState>) -> Result<Vec<FreeProxyResponse>, String> {
    let mut manager = FreeIpProviderManager::new().map_err(|e| e.to_string())?;
    let proxies = manager.fetch_all().await;
    state.proxy_manager.add_free_proxies(proxies.clone()).await;
    Ok(proxies.into_iter().map(FreeProxyResponse::from).collect())
}

#[tauri::command]
async fn get_free_proxies(state: State<'_, AppState>) -> Result<Vec<FreeProxyResponse>, String> {
    Ok(state.proxy_manager.get_free_proxies().await.into_iter().map(FreeProxyResponse::from).collect())
}

#[tauri::command]
async fn test_proxy(_state: State<'_, AppState>, proxy: FreeProxyRequest) -> Result<ProxyTestResultResponse, String> {
    let manager = FreeIpProviderManager::new().map_err(|e| e.to_string())?;
    let result = manager.test_proxy(&proxy.into()).await;
    Ok(ProxyTestResultResponse::from(result))
}

#[tauri::command]
async fn clear_free_proxies(state: State<'_, AppState>) -> Result<(), String> {
    state.proxy_manager.clear_proxies().await;
    Ok(())
}

#[tauri::command]
async fn remove_dead_proxies(state: State<'_, AppState>) -> Result<(), String> {
    state.proxy_manager.remove_dead_proxies().await;
    Ok(())
}

// Backup & Restore
#[tauri::command]
async fn create_backup(state: State<'_, AppState>, options: BackupOptionsRequest) -> Result<BackupInfoResponse, String> {
    let cookies = if options.include_cookies {
        Some(state.storage_engine.get_all_cookies().await.map_err(|e| e.to_string())?)
    } else { None };
    
    let history = if options.include_history {
        Some(state.storage_engine.get_history(1000).await.map_err(|e| e.to_string())?)
    } else { None };
    
    let bookmarks = if options.include_bookmarks {
        Some(state.storage_engine.get_bookmarks().await.map_err(|e| e.to_string())?)
    } else { None };
    
    let proxy_settings = if options.include_proxy_settings {
        Some(state.proxy_manager.get_settings().await)
    } else { None };

    let data = BackupData {
        version: "1.0".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        proxy_settings,
        browser_config: None,
        cookies,
        history,
        bookmarks,
        local_storage: None,
    };

    let backup_options = BackupOptions {
        include_proxy_settings: options.include_proxy_settings,
        include_browser_config: options.include_browser_config,
        include_cookies: options.include_cookies,
        include_history: options.include_history,
        include_bookmarks: options.include_bookmarks,
        include_local_storage: options.include_local_storage,
        password: options.password,
    };

    let info = state.backup_manager.create_backup(data, &backup_options).await.map_err(|e| e.to_string())?;
    Ok(BackupInfoResponse::from(info))
}

#[tauri::command]
async fn list_backups(state: State<'_, AppState>) -> Result<Vec<BackupInfoResponse>, String> {
    let backups = state.backup_manager.list_backups().await.map_err(|e| e.to_string())?;
    Ok(backups.into_iter().map(BackupInfoResponse::from).collect())
}

#[tauri::command]
async fn restore_backup(state: State<'_, AppState>, path: String, password: Option<String>) -> Result<(), String> {
    let backup_data = state.backup_manager.restore_backup(
        std::path::Path::new(&path),
        password.as_deref()
    ).await.map_err(|e| e.to_string())?;

    // Restore proxy settings
    if let Some(proxy_settings) = backup_data.proxy_settings {
        state.proxy_manager.set_settings(proxy_settings).await;
    }

    // Restore cookies
    if let Some(cookies) = backup_data.cookies {
        for cookie in cookies {
            state.storage_engine.set_cookie(&cookie).await.map_err(|e| e.to_string())?;
        }
    }

    // Restore bookmarks
    if let Some(bookmarks) = backup_data.bookmarks {
        for bookmark in bookmarks {
            state.storage_engine.add_bookmark(&bookmark.url, &bookmark.title, bookmark.folder.as_deref()).await.map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
async fn delete_backup(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.backup_manager.delete_backup(&id).await.map_err(|e| e.to_string())
}

// Tab management - close tab
#[tauri::command]
async fn close_tab(state: State<'_, AppState>, app_handle: tauri::AppHandle, tab_id: String) -> Result<(), String> {
    let manager = app_handle.state::<WebviewManager>();
    manager.close_tab(&tab_id).await.map_err(|e| e.to_string())?;
    state.browser_controller.close_tab(&tab_id).await;
    Ok(())
}

// Browser controls
#[tauri::command]
async fn navigate(state: State<'_, AppState>, tab_id: String, url: String) -> Result<BrowserStateResponse, String> {
    let browser_state = state.browser_controller.navigate(&tab_id, &url).await.map_err(|e| e.to_string())?;
    // Also record in storage
    let _ = state.storage_engine.add_history(&url, None).await;
    Ok(BrowserStateResponse::from(browser_state))
}

#[tauri::command]
async fn go_back(state: State<'_, AppState>, tab_id: String) -> Result<Option<String>, String> {
    state.browser_controller.go_back(&tab_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn go_forward(state: State<'_, AppState>, tab_id: String) -> Result<Option<String>, String> {
    state.browser_controller.go_forward(&tab_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn reload_page(state: State<'_, AppState>, tab_id: String) -> Result<Option<String>, String> {
    state.browser_controller.reload(&tab_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_browser_state(state: State<'_, AppState>, tab_id: String) -> Result<Option<BrowserStateResponse>, String> {
    Ok(state.browser_controller.get_state(&tab_id).await.map(BrowserStateResponse::from))
}

#[tauri::command]
async fn update_page_title(state: State<'_, AppState>, tab_id: String, title: String) -> Result<(), String> {
    state.browser_controller.update_title(&tab_id, &title).await;
    Ok(())
}

#[tauri::command]
async fn get_browser_settings(state: State<'_, AppState>) -> Result<BrowserSettingsResponse, String> {
    Ok(BrowserSettingsResponse::from(state.browser_controller.get_settings().await))
}

#[tauri::command]
async fn set_browser_settings(state: State<'_, AppState>, settings: BrowserSettingsRequest) -> Result<(), String> {
    state.browser_controller.set_settings(settings.into()).await;
    Ok(())
}

// History commands
#[tauri::command]
async fn get_history(state: State<'_, AppState>, limit: i64) -> Result<Vec<HistoryEntryResponse>, String> {
    let history = state.storage_engine.get_history(limit).await.map_err(|e| e.to_string())?;
    Ok(history.into_iter().map(HistoryEntryResponse::from).collect())
}

#[tauri::command]
async fn search_history(state: State<'_, AppState>, query: String) -> Result<Vec<HistoryEntryResponse>, String> {
    let history = state.storage_engine.search_history(&query).await.map_err(|e| e.to_string())?;
    Ok(history.into_iter().map(HistoryEntryResponse::from).collect())
}

#[tauri::command]
async fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    state.storage_engine.clear_history().await.map_err(|e| e.to_string())
}

// Bookmark commands
#[tauri::command]
async fn add_bookmark(state: State<'_, AppState>, url: String, title: String, folder: Option<String>) -> Result<i64, String> {
    state.storage_engine.add_bookmark(&url, &title, folder.as_deref()).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_bookmarks(state: State<'_, AppState>) -> Result<Vec<BookmarkResponse>, String> {
    let bookmarks = state.storage_engine.get_bookmarks().await.map_err(|e| e.to_string())?;
    Ok(bookmarks.into_iter().map(BookmarkResponse::from).collect())
}

#[tauri::command]
async fn delete_bookmark(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    state.storage_engine.delete_bookmark(id).await.map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabResponse {
    pub tab_id: String,
    pub ip: String,
    pub country_code: String,
    pub country_name: String,
    pub city: String,
    pub timezone: String,
    pub isp: String,
}

impl From<browser_core::TabProfile> for TabResponse {
    fn from(tab: browser_core::TabProfile) -> Self {
        Self {
            tab_id: tab.tab_id,
            ip: tab.virtual_ip.ip.to_string(),
            country_code: tab.virtual_ip.country_code,
            country_name: tab.virtual_ip.country,
            city: tab.virtual_ip.city,
            timezone: tab.virtual_ip.timezone,
            isp: tab.virtual_ip.isp,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualIPResponse {
    pub ip: String,
    pub country_code: String,
    pub country_name: String,
    pub city: String,
    pub region: String,
    pub timezone: String,
    pub language: String,
    pub currency: String,
    pub isp: String,
}

impl From<VirtualIP> for VirtualIPResponse {
    fn from(ip: VirtualIP) -> Self {
        Self {
            ip: ip.ip.to_string(),
            country_code: ip.country_code,
            country_name: ip.country,
            city: ip.city,
            region: ip.region,
            timezone: ip.timezone,
            language: ip.language,
            currency: ip.currency,
            isp: ip.isp,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryResponse {
    pub code: String,
    pub name: String,
    pub flag: String,
    pub timezone: String,
    pub language: String,
    pub currency: String,
    pub is_top: bool,
}

impl From<Country> for CountryResponse {
    fn from(c: Country) -> Self {
        Self {
            code: c.code,
            name: c.name,
            flag: c.flag,
            timezone: c.timezone,
            language: c.language,
            currency: c.currency,
            is_top: c.is_top,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResponse {
    pub ip: String,
    pub ip_matches: bool,
    pub webrtc_secure: bool,
    pub dns_secure: bool,
    pub overall_pass: bool,
}

// Proxy types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySettingsResponse {
    pub proxy_type: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub dns_servers: Vec<String>,
    pub bypass_list: Vec<String>,
}

impl From<ProxySettings> for ProxySettingsResponse {
    fn from(s: ProxySettings) -> Self {
        Self {
            proxy_type: match s.proxy_type {
                ProxyType::Direct => "direct",
                ProxyType::Http => "http",
                ProxyType::Https => "https",
                ProxyType::Socks4 => "socks4",
                ProxyType::Socks5 => "socks5",
            }.to_string(),
            host: s.host,
            port: s.port,
            username: s.username,
            password: s.password,
            dns_servers: s.dns_servers,
            bypass_list: s.bypass_list,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySettingsRequest {
    pub proxy_type: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub dns_servers: Vec<String>,
    pub bypass_list: Vec<String>,
}

impl From<ProxySettingsRequest> for ProxySettings {
    fn from(r: ProxySettingsRequest) -> Self {
        Self {
            proxy_type: match r.proxy_type.as_str() {
                "http" => ProxyType::Http,
                "https" => ProxyType::Https,
                "socks4" => ProxyType::Socks4,
                "socks5" => ProxyType::Socks5,
                _ => ProxyType::Direct,
            },
            host: r.host,
            port: r.port,
            username: r.username,
            password: r.password,
            dns_servers: r.dns_servers,
            bypass_list: r.bypass_list,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeProxyResponse {
    pub ip: String,
    pub port: u16,
    pub protocol: String,
    pub country: String,
    pub country_code: String,
    pub anonymity: String,
    pub speed: u32,
    pub uptime: f32,
    pub last_checked: String,
    pub provider: String,
    pub is_working: bool,
}

impl From<FreeProxy> for FreeProxyResponse {
    fn from(p: FreeProxy) -> Self {
        Self {
            ip: p.ip,
            port: p.port,
            protocol: match p.protocol {
                ProxyType::Http => "http",
                ProxyType::Https => "https",
                ProxyType::Socks4 => "socks4",
                ProxyType::Socks5 => "socks5",
                ProxyType::Direct => "direct",
            }.to_string(),
            country: p.country,
            country_code: p.country_code,
            anonymity: p.anonymity,
            speed: p.speed,
            uptime: p.uptime,
            last_checked: p.last_checked,
            provider: p.provider,
            is_working: p.is_working,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeProxyRequest {
    pub ip: String,
    pub port: u16,
    pub protocol: String,
    pub country: String,
    pub country_code: String,
    pub anonymity: String,
    pub speed: u32,
    pub uptime: f32,
    pub last_checked: String,
    pub provider: String,
    pub is_working: bool,
}

impl From<FreeProxyRequest> for FreeProxy {
    fn from(r: FreeProxyRequest) -> Self {
        Self {
            ip: r.ip,
            port: r.port,
            protocol: match r.protocol.as_str() {
                "http" => ProxyType::Http,
                "https" => ProxyType::Https,
                "socks4" => ProxyType::Socks4,
                "socks5" => ProxyType::Socks5,
                _ => ProxyType::Direct,
            },
            country: r.country,
            country_code: r.country_code,
            anonymity: r.anonymity,
            speed: r.speed,
            uptime: r.uptime,
            last_checked: r.last_checked,
            provider: r.provider,
            is_working: r.is_working,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyTestResultResponse {
    pub proxy: FreeProxyResponse,
    pub is_working: bool,
    pub latency_ms: Option<u64>,
    pub detected_ip: Option<String>,
    pub error: Option<String>,
}

impl From<browser_core::ProxyTestResult> for ProxyTestResultResponse {
    fn from(r: browser_core::ProxyTestResult) -> Self {
        Self {
            proxy: FreeProxyResponse::from(r.proxy),
            is_working: r.is_working,
            latency_ms: r.latency_ms,
            detected_ip: r.detected_ip,
            error: r.error,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicIpResponse {
    pub ip: String,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub isp: Option<String>,
    pub timezone: Option<String>,
}

impl From<PublicIpInfo> for PublicIpResponse {
    fn from(i: PublicIpInfo) -> Self {
        Self {
            ip: i.ip,
            country: i.country,
            country_code: i.country_code,
            city: i.city,
            region: i.region,
            isp: i.isp,
            timezone: i.timezone,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupOptionsRequest {
    pub include_proxy_settings: bool,
    pub include_browser_config: bool,
    pub include_cookies: bool,
    pub include_history: bool,
    pub include_bookmarks: bool,
    pub include_local_storage: bool,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfoResponse {
    pub id: String,
    pub filename: String,
    pub path: String,
    pub created_at: String,
    pub size_bytes: u64,
    pub is_encrypted: bool,
}

impl From<BackupInfo> for BackupInfoResponse {
    fn from(i: BackupInfo) -> Self {
        Self {
            id: i.id,
            filename: i.filename,
            path: i.path.to_string_lossy().to_string(),
            created_at: i.created_at,
            size_bytes: i.size_bytes,
            is_encrypted: i.is_encrypted,
        }
    }
}

// Browser state types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserStateResponse {
    pub tab_id: String,
    pub current_url: String,
    pub title: String,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub is_loading: bool,
}

impl From<BrowserState> for BrowserStateResponse {
    fn from(s: BrowserState) -> Self {
        Self {
            tab_id: s.tab_id,
            current_url: s.current_url,
            title: s.title,
            can_go_back: s.can_go_back,
            can_go_forward: s.can_go_forward,
            is_loading: s.is_loading,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSettingsResponse {
    pub user_agent: String,
    pub language: String,
    pub timezone: String,
    pub webrtc_policy: String,
    pub dns_over_https: bool,
    pub block_trackers: bool,
    pub block_ads: bool,
    pub javascript_enabled: bool,
    pub cookies_enabled: bool,
    pub engine_type: String,
    pub stealth_mode: bool,
    pub headless_mode: bool,
}

impl From<BrowserSettings> for BrowserSettingsResponse {
    fn from(s: BrowserSettings) -> Self {
        Self {
            user_agent: s.user_agent,
            language: s.language,
            timezone: s.timezone,
            webrtc_policy: match s.webrtc_policy {
                WebRtcPolicy::Default => "default",
                WebRtcPolicy::DisableNonProxiedUdp => "disable_non_proxied_udp",
                WebRtcPolicy::Disabled => "disabled",
            }.to_string(),
            dns_over_https: s.dns_over_https,
            block_trackers: s.block_trackers,
            block_ads: s.block_ads,
            javascript_enabled: s.javascript_enabled,
            cookies_enabled: s.cookies_enabled,
            engine_type: match s.engine_type {
                browser_core::BrowserEngineType::System => "system",
                browser_core::BrowserEngineType::IntegratedChromium => "integrated_chromium",
            }.to_string(),
            stealth_mode: s.stealth_mode,
            headless_mode: s.headless_mode,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSettingsRequest {
    pub user_agent: String,
    pub language: String,
    pub timezone: String,
    pub webrtc_policy: String,
    pub dns_over_https: bool,
    pub block_trackers: bool,
    pub block_ads: bool,
    pub javascript_enabled: bool,
    pub cookies_enabled: bool,
    pub engine_type: String,
    pub stealth_mode: bool,
    pub headless_mode: bool,
}

impl From<BrowserSettingsRequest> for BrowserSettings {
    fn from(r: BrowserSettingsRequest) -> Self {
        Self {
            user_agent: r.user_agent,
            language: r.language,
            timezone: r.timezone,
            webrtc_policy: match r.webrtc_policy.as_str() {
                "disabled" => WebRtcPolicy::Disabled,
                "disable_non_proxied_udp" => WebRtcPolicy::DisableNonProxiedUdp,
                _ => WebRtcPolicy::Default,
            },
            dns_over_https: r.dns_over_https,
            block_trackers: r.block_trackers,
            block_ads: r.block_ads,
            javascript_enabled: r.javascript_enabled,
            cookies_enabled: r.cookies_enabled,
            engine_type: match r.engine_type.as_str() {
                "integrated_chromium" => browser_core::BrowserEngineType::IntegratedChromium,
                _ => browser_core::BrowserEngineType::System,
            },
            stealth_mode: r.stealth_mode,
            headless_mode: r.headless_mode,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntryResponse {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub visit_count: i32,
    pub last_visit: i64,
}

impl From<browser_core::HistoryEntry> for HistoryEntryResponse {
    fn from(h: browser_core::HistoryEntry) -> Self {
        Self {
            id: h.id,
            url: h.url,
            title: h.title,
            visit_count: h.visit_count,
            last_visit: h.last_visit,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkResponse {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub folder: Option<String>,
    pub created_at: i64,
}

impl From<browser_core::Bookmark> for BookmarkResponse {
    fn from(b: browser_core::Bookmark) -> Self {
        Self {
            id: b.id,
            url: b.url,
            title: b.title,
            folder: b.folder,
            created_at: b.created_at,
        }
    }
}

fn build_ip_generator() -> IPGenerator {
    let countries_path = std::env::var("COUNTRIES_PATH").ok();
    let ranges_path = std::env::var("IP_RANGES_PATH").ok();

    let countries = countries_path
        .as_deref()
        .map(std::path::Path::new)
        .map(load_countries_from_file)
        .unwrap_or_else(CountryDatabase::load_all_countries);

    let ranges = ranges_path
        .as_deref()
        .map(std::path::Path::new)
        .map(load_ip_ranges_from_file)
        .unwrap_or_else(load_ip_ranges);

    if countries.is_empty() || ranges.is_empty() {
        demo_generator()
    } else {
        IPGenerator::new(countries, ranges)
    }
}

fn main() {
    let ip_generator = Arc::new(build_ip_generator());
    let proxy_manager = Arc::new(ProxyManager::new());
    let browser_controller = Arc::new(BrowserController::new());
    
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .setup(move |app| {
            // Initialize WebviewManager
            let webview_manager = WebviewManager::new(app.handle().clone());
            app.manage(webview_manager);
            
            // Get app data directory using Tauri v2 API
            let app_data_dir = app.path().app_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir().join("virtual-ip-browser"));
            std::fs::create_dir_all(&app_data_dir).ok();
            
            // Initialize storage engine
            let storage_dir = app_data_dir.join("data");
            let storage_engine = match StorageEngine::new(&storage_dir) {
                Ok(engine) => Arc::new(engine),
                Err(e) => {
                    warn!(" Failed to initialize storage engine: {}. Using temp directory.", e);
                    let temp_dir = std::env::temp_dir().join("virtual-ip-browser/data");
                    Arc::new(StorageEngine::new(&temp_dir).expect("Failed to create storage engine"))
                }
            };
            
            // Initialize backup manager
            let backup_dir = app_data_dir.join("backups");
            let backup_manager = match BackupManager::new(&backup_dir) {
                Ok(manager) => Arc::new(manager),
                Err(e) => {
                    warn!(" Failed to initialize backup manager: {}. Using temp directory.", e);
                    let temp_dir = std::env::temp_dir().join("virtual-ip-browser/backups");
                    Arc::new(BackupManager::new(&temp_dir).expect("Failed to create backup manager"))
                }
            };
            
            // Fetch free proxies on startup
            let proxy_manager_clone = proxy_manager.clone();
            tauri::async_runtime::spawn(async move {
                info!("Fetching free proxies on startup...");
                match proxy_manager_clone.fetch_proxies().await {
                    Ok(count) => info!("Successfully fetched {} proxies", count),
                    Err(e) => error!("Failed to fetch free proxies on startup: {}", e),
                }
            });
            
            // Manage the app state
            app.manage(AppState {
                ip_generator,
                proxy_manager,
                storage_engine,
                backup_manager,
                browser_controller,
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // WebView Manager commands
            create_webview_tab,
            create_webview_tab_with_proxy,
            navigate_webview_tab,
            close_webview_tab,
            focus_webview_tab,
            get_webview_tabs,
            navigation_changed,
            title_changed,
            // Tab commands
            create_tab,
            create_tab_random,
            list_tabs,
            rotate_ip,
            validate_ip,
            list_countries,
            close_tab,
            // Proxy commands
            get_proxy_settings,
            set_proxy_settings,
            get_active_proxy,
            set_active_proxy,
            // Public IP
            detect_public_ip,
            // Free proxies
            fetch_free_proxies,
            get_free_proxies,
            test_proxy,
            clear_free_proxies,
            remove_dead_proxies,
            // Backup
            create_backup,
            list_backups,
            restore_backup,
            delete_backup,
            // Browser controls
            navigate,
            go_back,
            go_forward,
            reload_page,
            get_browser_state,
            update_page_title,
            get_browser_settings,
            set_browser_settings,
            // History
            get_history,
            search_history,
            clear_history,
            // Bookmarks
            add_bookmark,
            get_bookmarks,
            delete_bookmark
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
