pub mod database;
pub mod tab_manager;
pub mod tab_isolation;
pub mod fingerprint;
pub mod proxy;
pub mod http_client;
pub mod scraper_util;
pub mod security;
pub mod webview_manager;
pub mod browser_tab_manager;
pub mod free_ip_providers;
pub mod storage;
pub mod backup;
pub mod browser_controls;
pub mod local_proxy;
pub mod pac_server;
pub mod proxy_rotation;
pub mod proxy_validator;
pub mod chromium_engine;
pub mod ad_verification;

pub use database::{Database, DatabaseError};
pub use tab_manager::TabIPManager;
pub use tab_isolation::{TabProfile, NetworkConfig, TabStatus};
pub use fingerprint::BrowserFingerprint;
pub use proxy::{ProxyManager, ProxySettings, ProxyType, FreeProxy, ProxyTestResult};
pub use http_client::{HttpClient, PublicIpDetector, PublicIpInfo};
pub use scraper_util::ProxyScraper;
pub use security::{SecurityManager, BookmarkInput, ProxyInput};
pub use webview_manager::{
    WebviewManager, WebviewTab,
    // Tauri command handlers from webview_manager
    create_webview_tab, navigate_webview_tab, close_webview_tab,
    focus_webview_tab, get_webview_tabs, navigation_changed,
    title_changed, go_back_tab, go_forward_tab, reload_tab,
    stop_tab, set_tab_zoom, get_active_tab, execute_script_in_tab,
    rotate_proxy_for_tab, update_rotation_strategy, get_proxy_session_stats,
    fetch_proxies_from_provider,
};
pub use browser_tab_manager::{
    BrowserTabManager, BrowserTab, CreateTabConfig, TabStats,
    // Tauri command handlers from browser_tab_manager
    create_browser_tab, close_browser_tab, switch_browser_tab,
    navigate_browser_tab, browser_tab_go_back, browser_tab_go_forward,
    reload_browser_tab, stop_browser_tab, set_browser_tab_zoom,
    rotate_browser_tab_ip, get_browser_tabs, get_browser_tab,
    get_active_browser_tab, execute_script_in_browser_tab,
    clear_browser_tab_data, get_browser_tab_stats,
    update_webview_tab_state,
};
pub use free_ip_providers::{FreeIpProvider, FreeIpProviderManager};
pub use storage::{StorageEngine, Cookie, HistoryEntry, Bookmark};
pub use backup::{BackupManager, BackupData, BackupOptions, BackupInfo, AutoBackupSettings};
pub use browser_controls::{BrowserController, BrowserState, BrowserSettings, WebRtcPolicy};
pub use local_proxy::{LocalProxyServer, LocalProxyManager, ProxyConnection};
pub use pac_server::{PacServer, PacManager};
pub use proxy_rotation::{ProxyRotationManager, ProxyRotationStrategy, ProxyMetrics, ProxySessionStats};
pub use proxy_validator::{ProxyValidator, ProxyValidatorConfig, ValidationResult, ProxyHealthChecker};
pub use chromium_engine::{ChromiumEngine, ChromiumEngineConfig, BrowserEngineManager, BrowserEngineType, ChromiumTab, EngineCapabilities};
pub use ad_verification::{AdVerificationManager, AdVerificationConfig, ImpressionVerification, ImpressionData, VastVerification, VpaidVerification, VerificationSession, SessionStats, ViewabilityStatus, FraudSignal, AdFormat, VerificationStandard};
