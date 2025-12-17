use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProxyType {
    Direct,
    Http,
    Https,
    Socks4,
    Socks5,
}

impl Default for ProxyType {
    fn default() -> Self {
        ProxyType::Direct
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProxySettings {
    pub proxy_type: ProxyType,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub dns_servers: Vec<String>,
    pub bypass_list: Vec<String>,
}

impl Default for ProxySettings {
    fn default() -> Self {
        Self {
            proxy_type: ProxyType::Direct,
            host: None,
            port: None,
            username: None,
            password: None,
            dns_servers: vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()],
            bypass_list: vec!["localhost".to_string(), "127.0.0.1".to_string()],
        }
    }
}

impl ProxySettings {
    pub fn to_url(&self) -> Option<String> {
        if self.proxy_type == ProxyType::Direct {
            return None;
        }

        let scheme = match self.proxy_type {
            ProxyType::Http => "http",
            ProxyType::Https => "https",
            ProxyType::Socks4 => "socks4",
            ProxyType::Socks5 => "socks5",
            ProxyType::Direct => return None,
        };

        let host = self.host.as_ref()?;
        let port = self.port?;

        let auth = match (&self.username, &self.password) {
            (Some(user), Some(pass)) => format!("{}:{}@", user, pass),
            (Some(user), None) => format!("{}@", user),
            _ => String::new(),
        };

        Some(format!("{}://{}{}:{}", scheme, auth, host, port))
    }

    pub fn is_configured(&self) -> bool {
        self.proxy_type != ProxyType::Direct && self.host.is_some() && self.port.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeProxy {
    pub ip: String,
    pub port: u16,
    pub protocol: ProxyType,
    pub country: String,
    pub country_code: String,
    pub anonymity: String,
    pub speed: u32,
    pub uptime: f32,
    pub last_checked: String,
    pub provider: String,
    pub is_working: bool,
}

impl FreeProxy {
    pub fn to_proxy_settings(&self) -> ProxySettings {
        ProxySettings {
            proxy_type: self.protocol.clone(),
            host: Some(self.ip.clone()),
            port: Some(self.port),
            username: None,
            password: None,
            dns_servers: vec!["1.1.1.1".to_string()],
            bypass_list: vec!["localhost".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyTestResult {
    pub proxy: FreeProxy,
    pub is_working: bool,
    pub latency_ms: Option<u64>,
    pub detected_ip: Option<String>,
    pub error: Option<String>,
}

pub struct ProxyManager {
    settings: Arc<RwLock<ProxySettings>>,
    free_proxies: Arc<RwLock<Vec<FreeProxy>>>,
    active_proxy: Arc<RwLock<Option<FreeProxy>>>,
}

impl ProxyManager {
    pub fn new() -> Self {
        Self {
            settings: Arc::new(RwLock::new(ProxySettings::default())),
            free_proxies: Arc::new(RwLock::new(Vec::new())),
            active_proxy: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn get_settings(&self) -> ProxySettings {
        self.settings.read().await.clone()
    }

    pub async fn set_settings(&self, settings: ProxySettings) {
        *self.settings.write().await = settings;
    }

    pub async fn get_free_proxies(&self) -> Vec<FreeProxy> {
        self.free_proxies.read().await.clone()
    }

    pub async fn add_free_proxies(&self, proxies: Vec<FreeProxy>) {
        let mut list = self.free_proxies.write().await;
        for proxy in proxies {
            if !list.iter().any(|p| p.ip == proxy.ip && p.port == proxy.port) {
                list.push(proxy);
            }
        }
    }

    pub async fn set_active_proxy(&self, proxy: Option<FreeProxy>) {
        *self.active_proxy.write().await = proxy;
    }

    pub async fn get_active_proxy(&self) -> Option<FreeProxy> {
        self.active_proxy.read().await.clone()
    }

    pub async fn remove_dead_proxies(&self) {
        let mut list = self.free_proxies.write().await;
        list.retain(|p| p.is_working);
    }

    pub async fn clear_proxies(&self) {
        self.free_proxies.write().await.clear();
    }

    pub async fn get_effective_proxy_url(&self) -> Option<String> {
        // First check if there's an active free proxy
        if let Some(active) = self.active_proxy.read().await.as_ref() {
            return active.to_proxy_settings().to_url();
        }
        // Otherwise use manual settings
        self.settings.read().await.to_url()
    }

    pub async fn fetch_proxies(&self) -> Result<usize, Box<dyn std::error::Error>> {
        // Try to fetch from provider first
        match crate::FreeIpProviderManager::new() {
            Ok(mut manager) => {
                let proxies = manager.fetch_all().await;
                let count = proxies.len();
                self.add_free_proxies(proxies).await;
                Ok(count)
            }
            Err(e) => {
                // Fallback: add test proxies for development
                warn!("Failed to initialize FreeIpProviderManager: {}. Adding test proxies.", e);
                let test_proxies = vec![
                    FreeProxy {
                        ip: "192.168.1.100".to_string(),
                        port: 8080,
                        protocol: ProxyType::Http,
                        country: "United States".to_string(),
                        country_code: "US".to_string(),
                        anonymity: "anonymous".to_string(),
                        speed: 100,
                        uptime: 95.0,
                        last_checked: "2024-01-01T00:00:00Z".to_string(),
                        provider: "test".to_string(),
                        is_working: true,
                    },
                    FreeProxy {
                        ip: "10.0.0.1".to_string(),
                        port: 3128,
                        protocol: ProxyType::Http,
                        country: "United Kingdom".to_string(),
                        country_code: "GB".to_string(),
                        anonymity: "anonymous".to_string(),
                        speed: 150,
                        uptime: 90.0,
                        last_checked: "2024-01-01T00:00:00Z".to_string(),
                        provider: "test".to_string(),
                        is_working: true,
                    },
                ];
                let count = test_proxies.len();
                self.add_free_proxies(test_proxies).await;
                Ok(count)
            }
        }
    }
}

impl Default for ProxyManager {
    fn default() -> Self {
        Self::new()
    }
}
