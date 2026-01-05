//! Configuration Management Module
//!
//! Provides centralized configuration management with:
//! - Type-safe configuration access
//! - Environment variable overrides
//! - Hot-reloading support
//! - Validation and defaults

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// General application settings
    pub general: GeneralConfig,
    /// Proxy settings
    pub proxy: ProxyConfig,
    /// Privacy settings
    pub privacy: PrivacyConfig,
    /// Performance settings
    pub performance: PerformanceConfig,
    /// Network settings
    pub network: NetworkConfig,
    /// Storage settings
    pub storage: StorageConfig,
    /// Logging settings
    pub logging: LoggingConfig,
    /// Feature flags
    pub features: FeatureFlags,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            proxy: ProxyConfig::default(),
            privacy: PrivacyConfig::default(),
            performance: PerformanceConfig::default(),
            network: NetworkConfig::default(),
            storage: StorageConfig::default(),
            logging: LoggingConfig::default(),
            features: FeatureFlags::default(),
        }
    }
}

/// General application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Application name
    pub app_name: String,
    /// Application version
    pub version: String,
    /// Default language
    pub language: String,
    /// Theme (light, dark, system)
    pub theme: String,
    /// Auto-update enabled
    pub auto_update: bool,
    /// Telemetry enabled
    pub telemetry_enabled: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            app_name: "Proxy-Desktop-Browser".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            language: "en".to_string(),
            theme: "system".to_string(),
            auto_update: true,
            telemetry_enabled: false,
        }
    }
}

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Enable proxy by default
    pub enabled: bool,
    /// Default proxy type (http, https, socks4, socks5)
    pub default_type: String,
    /// Default proxy host
    pub default_host: Option<String>,
    /// Default proxy port
    pub default_port: Option<u16>,
    /// Proxy rotation enabled
    pub rotation_enabled: bool,
    /// Rotation strategy
    pub rotation_strategy: String,
    /// Rotation interval in seconds
    pub rotation_interval_secs: u64,
    /// Maximum proxy failures before blacklisting
    pub max_failures: u32,
    /// Proxy validation timeout in milliseconds
    pub validation_timeout_ms: u64,
    /// Use free proxy providers
    pub use_free_providers: bool,
    /// Preferred countries for proxy selection
    pub preferred_countries: Vec<String>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_type: "http".to_string(),
            default_host: None,
            default_port: None,
            rotation_enabled: true,
            rotation_strategy: "performance_based".to_string(),
            rotation_interval_secs: 300,
            max_failures: 3,
            validation_timeout_ms: 5000,
            use_free_providers: true,
            preferred_countries: vec![],
        }
    }
}

/// Privacy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    /// Block trackers
    pub block_trackers: bool,
    /// Block ads
    pub block_ads: bool,
    /// Block third-party cookies
    pub block_third_party_cookies: bool,
    /// Enable fingerprint protection
    pub fingerprint_protection: bool,
    /// Enable WebRTC leak protection
    pub webrtc_protection: bool,
    /// Clear data on exit
    pub clear_on_exit: bool,
    /// DNT header enabled
    pub do_not_track: bool,
    /// GPC header enabled
    pub global_privacy_control: bool,
    /// Cookie isolation level (none, session, strict)
    pub cookie_isolation: String,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            block_trackers: true,
            block_ads: true,
            block_third_party_cookies: true,
            fingerprint_protection: true,
            webrtc_protection: true,
            clear_on_exit: false,
            do_not_track: true,
            global_privacy_control: true,
            cookie_isolation: "session".to_string(),
        }
    }
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum concurrent tabs
    pub max_tabs: u32,
    /// Tab memory limit in MB
    pub tab_memory_limit_mb: u32,
    /// Enable lazy tab loading
    pub lazy_tab_loading: bool,
    /// Enable tab hibernation
    pub tab_hibernation: bool,
    /// Hibernation timeout in seconds
    pub hibernation_timeout_secs: u64,
    /// Enable hardware acceleration
    pub hardware_acceleration: bool,
    /// Cache size in MB
    pub cache_size_mb: u32,
    /// Enable prefetching
    pub prefetch_enabled: bool,
    /// JavaScript execution timeout in ms
    pub js_timeout_ms: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_tabs: 100,
            tab_memory_limit_mb: 512,
            lazy_tab_loading: true,
            tab_hibernation: true,
            hibernation_timeout_secs: 300,
            hardware_acceleration: true,
            cache_size_mb: 256,
            prefetch_enabled: true,
            js_timeout_ms: 30000,
        }
    }
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    /// Read timeout in milliseconds
    pub read_timeout_ms: u64,
    /// Maximum redirects
    pub max_redirects: u32,
    /// Maximum connections per host
    pub max_connections_per_host: u32,
    /// Total maximum connections
    pub max_total_connections: u32,
    /// Enable HTTP/2
    pub http2_enabled: bool,
    /// Enable QUIC/HTTP3
    pub http3_enabled: bool,
    /// DNS over HTTPS enabled
    pub doh_enabled: bool,
    /// DoH server URL
    pub doh_server: String,
    /// Custom DNS servers
    pub dns_servers: Vec<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            connection_timeout_ms: 30000,
            read_timeout_ms: 60000,
            max_redirects: 10,
            max_connections_per_host: 6,
            max_total_connections: 100,
            http2_enabled: true,
            http3_enabled: false,
            doh_enabled: true,
            doh_server: "https://cloudflare-dns.com/dns-query".to_string(),
            dns_servers: vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()],
        }
    }
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Data directory path
    pub data_dir: Option<String>,
    /// Cache directory path
    pub cache_dir: Option<String>,
    /// Enable auto-backup
    pub auto_backup: bool,
    /// Backup interval in hours
    pub backup_interval_hours: u32,
    /// Maximum backups to keep
    pub max_backups: u32,
    /// History retention days
    pub history_retention_days: u32,
    /// Maximum history entries
    pub max_history_entries: u32,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: None,
            cache_dir: None,
            auto_backup: true,
            backup_interval_hours: 24,
            max_backups: 7,
            history_retention_days: 90,
            max_history_entries: 10000,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Log to file
    pub file_logging: bool,
    /// Log file path
    pub log_file: Option<String>,
    /// Maximum log file size in MB
    pub max_file_size_mb: u32,
    /// Maximum log files to keep
    pub max_files: u32,
    /// Log format (json, pretty, compact)
    pub format: String,
    /// Include timestamps
    pub timestamps: bool,
    /// Include source location
    pub source_location: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_logging: false,
            log_file: None,
            max_file_size_mb: 10,
            max_files: 5,
            format: "pretty".to_string(),
            timestamps: true,
            source_location: false,
        }
    }
}

/// Feature flags for enabling/disabling features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Enable experimental features
    pub experimental: bool,
    /// Enable automation features
    pub automation: bool,
    /// Enable reader mode
    pub reader_mode: bool,
    /// Enable content enhancement
    pub content_enhancement: bool,
    /// Enable ad verification
    pub ad_verification: bool,
    /// Enable session management
    pub session_management: bool,
    /// Enable smart proxy selection
    pub smart_proxy_selection: bool,
    /// Enable language detection
    pub language_detection: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            experimental: false,
            automation: true,
            reader_mode: true,
            content_enhancement: true,
            ad_verification: false,
            session_management: true,
            smart_proxy_selection: true,
            language_detection: true,
        }
    }
}

/// Configuration manager for loading, saving, and managing configuration
pub struct ConfigManager {
    config: Arc<RwLock<AppConfig>>,
    config_path: Option<PathBuf>,
    env_prefix: String,
    watchers: Arc<RwLock<Vec<Box<dyn Fn(&AppConfig) + Send + Sync>>>>,
}

impl ConfigManager {
    /// Create a new configuration manager with default config
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(AppConfig::default())),
            config_path: None,
            env_prefix: "PROXY_BROWSER".to_string(),
            watchers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create configuration manager with a config file path
    pub fn with_path(path: impl AsRef<Path>) -> Self {
        Self {
            config: Arc::new(RwLock::new(AppConfig::default())),
            config_path: Some(path.as_ref().to_path_buf()),
            env_prefix: "PROXY_BROWSER".to_string(),
            watchers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Set the environment variable prefix
    pub fn with_env_prefix(mut self, prefix: &str) -> Self {
        self.env_prefix = prefix.to_string();
        self
    }

    /// Load configuration from file
    pub async fn load(&self) -> Result<()> {
        if let Some(path) = &self.config_path {
            if path.exists() {
                let content = tokio::fs::read_to_string(path).await
                    .context("Failed to read config file")?;
                
                let loaded_config: AppConfig = if path.extension().map_or(false, |e| e == "json") {
                    serde_json::from_str(&content).context("Failed to parse JSON config")?
                } else {
                    toml::from_str(&content).context("Failed to parse TOML config")?
                };

                *self.config.write().await = loaded_config;
                info!("Loaded configuration from {:?}", path);
            } else {
                info!("Config file not found, using defaults");
            }
        }

        // Apply environment variable overrides
        self.apply_env_overrides().await?;
        
        // Notify watchers
        self.notify_watchers().await;

        Ok(())
    }

    /// Save configuration to file
    pub async fn save(&self) -> Result<()> {
        if let Some(path) = &self.config_path {
            let config = self.config.read().await;
            
            let content = if path.extension().map_or(false, |e| e == "json") {
                serde_json::to_string_pretty(&*config)?
            } else {
                toml::to_string_pretty(&*config)?
            };

            // Ensure directory exists
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            tokio::fs::write(path, content).await
                .context("Failed to write config file")?;
            
            info!("Saved configuration to {:?}", path);
        }

        Ok(())
    }

    /// Get the current configuration
    pub async fn get(&self) -> AppConfig {
        self.config.read().await.clone()
    }

    /// Update the configuration
    pub async fn update<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut AppConfig),
    {
        {
            let mut config = self.config.write().await;
            updater(&mut config);
        }
        
        self.notify_watchers().await;
        
        // Auto-save if path is configured
        if self.config_path.is_some() {
            self.save().await?;
        }

        Ok(())
    }

    /// Get a specific configuration section
    pub async fn get_general(&self) -> GeneralConfig {
        self.config.read().await.general.clone()
    }

    pub async fn get_proxy(&self) -> ProxyConfig {
        self.config.read().await.proxy.clone()
    }

    pub async fn get_privacy(&self) -> PrivacyConfig {
        self.config.read().await.privacy.clone()
    }

    pub async fn get_performance(&self) -> PerformanceConfig {
        self.config.read().await.performance.clone()
    }

    pub async fn get_network(&self) -> NetworkConfig {
        self.config.read().await.network.clone()
    }

    pub async fn get_storage(&self) -> StorageConfig {
        self.config.read().await.storage.clone()
    }

    pub async fn get_logging(&self) -> LoggingConfig {
        self.config.read().await.logging.clone()
    }

    pub async fn get_features(&self) -> FeatureFlags {
        self.config.read().await.features.clone()
    }

    /// Check if a feature is enabled
    pub async fn is_feature_enabled(&self, feature: &str) -> bool {
        let features = self.config.read().await.features.clone();
        match feature {
            "experimental" => features.experimental,
            "automation" => features.automation,
            "reader_mode" => features.reader_mode,
            "content_enhancement" => features.content_enhancement,
            "ad_verification" => features.ad_verification,
            "session_management" => features.session_management,
            "smart_proxy_selection" => features.smart_proxy_selection,
            "language_detection" => features.language_detection,
            _ => false,
        }
    }

    /// Apply environment variable overrides
    async fn apply_env_overrides(&self) -> Result<()> {
        let mut config = self.config.write().await;
        
        // General overrides
        if let Ok(val) = std::env::var(format!("{}_THEME", self.env_prefix)) {
            config.general.theme = val;
        }
        if let Ok(val) = std::env::var(format!("{}_LANGUAGE", self.env_prefix)) {
            config.general.language = val;
        }

        // Proxy overrides
        if let Ok(val) = std::env::var(format!("{}_PROXY_ENABLED", self.env_prefix)) {
            config.proxy.enabled = val.parse().unwrap_or(config.proxy.enabled);
        }
        if let Ok(val) = std::env::var(format!("{}_PROXY_HOST", self.env_prefix)) {
            config.proxy.default_host = Some(val);
        }
        if let Ok(val) = std::env::var(format!("{}_PROXY_PORT", self.env_prefix)) {
            config.proxy.default_port = val.parse().ok();
        }

        // Logging overrides
        if let Ok(val) = std::env::var(format!("{}_LOG_LEVEL", self.env_prefix)) {
            config.logging.level = val;
        }

        // Feature flag overrides
        if let Ok(val) = std::env::var(format!("{}_EXPERIMENTAL", self.env_prefix)) {
            config.features.experimental = val.parse().unwrap_or(config.features.experimental);
        }

        debug!("Applied environment variable overrides");
        Ok(())
    }

    /// Notify configuration watchers
    async fn notify_watchers(&self) {
        let config = self.config.read().await;
        let watchers = self.watchers.read().await;
        for watcher in watchers.iter() {
            watcher(&config);
        }
    }

    /// Reset configuration to defaults
    pub async fn reset(&self) -> Result<()> {
        *self.config.write().await = AppConfig::default();
        self.notify_watchers().await;
        
        if self.config_path.is_some() {
            self.save().await?;
        }

        info!("Configuration reset to defaults");
        Ok(())
    }

    /// Validate the current configuration
    pub async fn validate(&self) -> Result<Vec<String>> {
        let config = self.config.read().await;
        let mut warnings = Vec::new();

        // Validate proxy settings
        if config.proxy.enabled {
            if config.proxy.default_host.is_none() {
                warnings.push("Proxy enabled but no host configured".to_string());
            }
            if config.proxy.default_port.is_none() {
                warnings.push("Proxy enabled but no port configured".to_string());
            }
        }

        // Validate performance settings
        if config.performance.max_tabs == 0 {
            warnings.push("max_tabs cannot be 0, using default".to_string());
        }
        if config.performance.tab_memory_limit_mb < 64 {
            warnings.push("tab_memory_limit_mb is very low, may cause issues".to_string());
        }

        // Validate network settings
        if config.network.connection_timeout_ms < 1000 {
            warnings.push("connection_timeout_ms is very low".to_string());
        }

        // Validate logging settings
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&config.logging.level.to_lowercase().as_str()) {
            warnings.push(format!("Invalid log level: {}", config.logging.level));
        }

        Ok(warnings)
    }

    /// Export configuration as JSON string
    pub async fn export_json(&self) -> Result<String> {
        let config = self.config.read().await;
        serde_json::to_string_pretty(&*config).context("Failed to serialize config")
    }

    /// Import configuration from JSON string
    pub async fn import_json(&self, json: &str) -> Result<()> {
        let imported: AppConfig = serde_json::from_str(json)
            .context("Failed to parse JSON config")?;
        *self.config.write().await = imported;
        self.notify_watchers().await;
        
        if self.config_path.is_some() {
            self.save().await?;
        }

        Ok(())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_config_manager_default() {
        let manager = ConfigManager::new();
        let config = manager.get().await;
        
        assert_eq!(config.general.app_name, "Proxy-Desktop-Browser");
        assert!(config.privacy.block_trackers);
    }

    #[tokio::test]
    async fn test_config_update() {
        let manager = ConfigManager::new();
        
        manager.update(|config| {
            config.proxy.enabled = true;
            config.proxy.default_host = Some("127.0.0.1".to_string());
            config.proxy.default_port = Some(8080);
        }).await.expect("Async operation should succeed");

        let config = manager.get().await;
        assert!(config.proxy.enabled);
        assert_eq!(config.proxy.default_host, Some("127.0.0.1".to_string()));
        assert_eq!(config.proxy.default_port, Some(8080));
    }

    #[tokio::test]
    async fn test_config_save_load() {
        let temp_dir = TempDir::new().expect("Operation should succeed in test");
        let config_path = temp_dir.path().join("config.json");
        
        // Save config
        {
            let manager = ConfigManager::with_path(&config_path);
            manager.update(|config| {
                config.general.theme = "dark".to_string();
            }).await.expect("Async operation should succeed");
            manager.save().await.expect("Save operation should succeed");
        }

        // Load config
        {
            let manager = ConfigManager::with_path(&config_path);
            manager.load().await.expect("Load operation should succeed");
            let config = manager.get().await;
            assert_eq!(config.general.theme, "dark");
        }
    }

    #[tokio::test]
    async fn test_feature_flags() {
        let manager = ConfigManager::new();
        
        assert!(manager.is_feature_enabled("reader_mode").await);
        assert!(!manager.is_feature_enabled("experimental").await);
        
        manager.update(|config| {
            config.features.experimental = true;
        }).await.expect("Async operation should succeed");
        
        assert!(manager.is_feature_enabled("experimental").await);
    }

    #[tokio::test]
    async fn test_config_validation() {
        let manager = ConfigManager::new();
        
        manager.update(|config| {
            config.proxy.enabled = true;
            // No host/port configured - should generate warnings
        }).await.expect("Async operation should succeed");

        let warnings = manager.validate().await.expect("Validate operation should succeed");
        assert!(!warnings.is_empty());
    }

    #[tokio::test]
    async fn test_config_export_import() {
        let manager = ConfigManager::new();
        
        manager.update(|config| {
            config.general.theme = "custom_theme".to_string();
        }).await.expect("Async operation should succeed");

        let json = manager.export_json().await.expect("Export operation should succeed");
        assert!(json.contains("custom_theme"));

        // Reset and import
        manager.reset().await.expect("Reset operation should succeed");
        manager.import_json(&json).await.expect("Import operation should succeed");
        
        let config = manager.get().await;
        assert_eq!(config.general.theme, "custom_theme");
    }
}
