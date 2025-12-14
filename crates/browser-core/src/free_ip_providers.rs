use anyhow::{Result, anyhow};
use serde::Deserialize;
use async_trait::async_trait;
use std::time::Duration;
use std::collections::HashMap;
use rand::seq::SliceRandom;

use crate::http_client::HttpClient;
use crate::proxy::{FreeProxy, ProxyType};
use crate::scraper_util;

#[derive(Debug, Clone)]
pub enum ProxyFilter {
    All,
    ByCountry(Vec<String>),
    ByType(Vec<ProxyType>),
    WorkingOnly,
}

#[async_trait]
pub trait ProxyProvider: Send + Sync {
    fn name(&self) -> &str;
    fn rate_limit(&self) -> Duration;
    async fn fetch_proxies(&self) -> Result<Vec<FreeProxy>>;
    async fn fetch_proxies_filtered(&self, filter: ProxyFilter) -> Result<Vec<FreeProxy>> {
        let proxies = self.fetch_proxies().await?;
        Ok(self.apply_filter(proxies, filter))
    }
    
    fn apply_filter(&self, proxies: Vec<FreeProxy>, filter: ProxyFilter) -> Vec<FreeProxy> {
        match filter {
            ProxyFilter::All => proxies,
            ProxyFilter::ByCountry(countries) => {
                proxies.into_iter()
                    .filter(|p| countries.contains(&p.country) || countries.contains(&p.country_code))
                    .collect()
            }
            ProxyFilter::ByType(types) => {
                proxies.into_iter()
                    .filter(|p| types.contains(&p.protocol))
                    .collect()
            }
            ProxyFilter::WorkingOnly => {
                proxies.into_iter()
                    .filter(|p| p.is_working)
                    .collect()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum FreeIpProvider {
    ProxyScrape,
    GeoNode,
    PubProxy,
    FreeProxyList,
    ProxyNova,
    SpysOne,
}

impl FreeIpProvider {
    pub fn all() -> Vec<Self> {
        vec![
            Self::ProxyScrape,
            Self::GeoNode,
            Self::PubProxy,
            Self::FreeProxyList,
            Self::ProxyNova,
            Self::SpysOne,
        ]
    }

    pub fn name(&self) -> &str {
        match self {
            Self::ProxyScrape => "ProxyScrape",
            Self::GeoNode => "GeoNode",
            Self::PubProxy => "PubProxy",
            Self::FreeProxyList => "FreeProxyList",
            Self::ProxyNova => "ProxyNova",
            Self::SpysOne => "SpysOne",
        }
    }

    pub fn rate_limit(&self) -> Duration {
        match self {
            Self::ProxyScrape => Duration::from_secs(1),
            Self::GeoNode => Duration::from_secs(2),
            Self::PubProxy => Duration::from_secs(1),
            Self::FreeProxyList => Duration::from_secs(5), // Web scraping needs more delay
            Self::ProxyNova => Duration::from_secs(5),
            Self::SpysOne => Duration::from_secs(5),
        }
    }

    pub fn is_api_based(&self) -> bool {
        match self {
            Self::ProxyScrape | Self::GeoNode | Self::PubProxy => true,
            Self::FreeProxyList | Self::ProxyNova | Self::SpysOne => false,
        }
    }
}

pub struct FreeIpProviderManager {
    http_client: HttpClient,
    proxy_pool: Vec<FreeProxy>,
    last_update: HashMap<String, chrono::DateTime<chrono::Utc>>,
    update_interval: Duration,
    rate_limiters: HashMap<String, std::time::Instant>,
}

impl FreeIpProviderManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            http_client: HttpClient::new()?,
            proxy_pool: Vec::new(),
            last_update: HashMap::new(),
            update_interval: Duration::from_secs(300), // 5 minutes default
            rate_limiters: HashMap::new(),
        })
    }

    pub fn with_update_interval(mut self, interval: Duration) -> Self {
        self.update_interval = interval;
        self
    }

    pub async fn fetch_from_provider(&mut self, provider: &FreeIpProvider) -> Result<Vec<FreeProxy>> {
        // Check rate limiting
        if let Some(last_fetch) = self.rate_limiters.get(&provider.name().to_string()) {
            let elapsed = last_fetch.elapsed();
            if elapsed < provider.rate_limit() {
                tokio::time::sleep(provider.rate_limit() - elapsed).await;
            }
        }

        let result = match provider {
            FreeIpProvider::ProxyScrape => self.fetch_proxyscrape().await,
            FreeIpProvider::GeoNode => self.fetch_geonode().await,
            FreeIpProvider::PubProxy => self.fetch_pubproxy().await,
            FreeIpProvider::FreeProxyList => {
                let scraper = scraper_util::ProxyScraper::new(HttpClient::new()?);
                scraper.scrape_free_proxy_list().await
                    .map_err(|e| anyhow!("Failed to scrape free-proxy-list.net: {}", e))
            }
            FreeIpProvider::ProxyNova => {
                let scraper = scraper_util::ProxyScraper::new(HttpClient::new()?);
                scraper.scrape_proxy_nova().await
                    .map_err(|e| anyhow!("Failed to scrape proxy-nova.com: {}", e))
            }
            FreeIpProvider::SpysOne => {
                let scraper = scraper_util::ProxyScraper::new(HttpClient::new()?);
                scraper.scrape_spys_one().await
                    .map_err(|e| anyhow!("Failed to scrape spys.one: {}", e))
            }
        };

        // Update rate limiter
        self.rate_limiters.insert(provider.name().to_string(), std::time::Instant::now());

        result
    }

    pub async fn fetch_all(&mut self) -> Vec<FreeProxy> {
        let mut all_proxies = Vec::new();
        
        for provider in FreeIpProvider::all() {
            match self.fetch_from_provider(&provider).await {
                Ok(proxies) => {
                    tracing::info!("Fetched {} proxies from {}", proxies.len(), provider.name());
                    all_proxies.extend(proxies);
                }
                Err(e) => {
                    tracing::error!("Failed to fetch from {}: {}", provider.name(), e);
                }
            }
        }
        
        // Remove duplicates based on IP:port
        all_proxies.sort_by(|a, b| a.ip.cmp(&b.ip).then(a.port.cmp(&b.port)));
        all_proxies.dedup_by(|a, b| a.ip == b.ip && a.port == b.port);
        
        tracing::info!("Total unique proxies fetched: {}", all_proxies.len());
        all_proxies
    }

    pub async fn update_proxy_pool(&mut self) -> Result<usize> {
        let proxies = self.fetch_all().await;
        let count = proxies.len();
        
        // Test a sample of proxies to mark working ones
        let working_proxies = self.test_proxy_sample(proxies).await;
        
        self.proxy_pool = working_proxies;
        
        // Update last fetch time for all providers
        let now = chrono::Utc::now();
        for provider in FreeIpProvider::all() {
            self.last_update.insert(provider.name().to_string(), now);
        }
        
        tracing::info!("Updated proxy pool with {} working proxies", self.proxy_pool.len());
        Ok(count)
    }

    async fn test_proxy_sample(&self, proxies: Vec<FreeProxy>) -> Vec<FreeProxy> {
        const SAMPLE_SIZE: usize = 50;
        let sample_size = proxies.len().min(SAMPLE_SIZE);
        
        if proxies.is_empty() {
            return proxies;
        }

        // Generate random sample indices before any async operations
        // Use a block scope to ensure rng is dropped before await
        let sample_indices: Vec<usize> = {
            let mut rng = rand::thread_rng();
            let mut indices: Vec<usize> = (0..proxies.len()).collect();
            indices.shuffle(&mut rng);
            indices.truncate(sample_size);
            indices
        };
        
        // Now we can safely use async operations since rng is dropped
        let mut tested_proxies = Vec::new();
        
        for &idx in &sample_indices {
            let proxy = &proxies[idx];
            let test_result = self.test_proxy(proxy).await;
            
            let mut proxy = proxy.clone();
            proxy.is_working = test_result.is_working;
            tested_proxies.push(proxy);
        }
        
        // Include untested proxies as non-working
        for (i, proxy) in proxies.into_iter().enumerate() {
            if !sample_indices.contains(&i) {
                let mut proxy = proxy;
                proxy.is_working = false;
                tested_proxies.push(proxy);
            }
        }
        
        tested_proxies
    }

    pub fn get_proxy_pool(&self) -> &[FreeProxy] {
        &self.proxy_pool
    }

    pub fn get_working_proxies(&self) -> Vec<&FreeProxy> {
        self.proxy_pool.iter()
            .filter(|p| p.is_working)
            .collect()
    }

    pub fn get_random_working_proxy(&self) -> Option<&FreeProxy> {
        let working = self.get_working_proxies();
        if working.is_empty() {
            None
        } else {
            Some(&working[rand::random::<usize>() % working.len()])
        }
    }

    pub fn needs_update(&self) -> bool {
        let oldest_update = self.last_update.values().min();
        match oldest_update {
            Some(oldest) => {
                let elapsed = chrono::Utc::now().signed_duration_since(*oldest);
                elapsed.to_std().unwrap_or(Duration::MAX) > self.update_interval
            }
            None => true, // Never updated
        }
    }

    pub fn start_auto_update_task(
        manager: std::sync::Arc<tokio::sync::RwLock<FreeIpProviderManager>>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // Check every 5 minutes
            
            loop {
                interval.tick().await;
                
                let needs_update = {
                    let mgr = manager.read().await;
                    mgr.needs_update()
                };
                
                if needs_update {
                    let mut mgr = manager.write().await;
                    if let Err(e) = mgr.update_proxy_pool().await {
                        tracing::error!("Failed to auto-update proxy pool: {}", e);
                    }
                }
            }
        })
    }

    async fn fetch_proxyscrape(&self) -> Result<Vec<FreeProxy>> {
        let url = "https://api.proxyscrape.com/v2/?request=displayproxies&protocol=http&timeout=10000&country=all&ssl=all&anonymity=all";
        let response = self.http_client.get(url).await?;
        
        let mut proxies = Vec::new();
        for line in response.lines() {
            let parts: Vec<&str> = line.trim().split(':').collect();
            if parts.len() == 2 {
                if let (Ok(port), ip) = (parts[1].parse::<u16>(), parts[0].to_string()) {
                    proxies.push(FreeProxy {
                        ip,
                        port,
                        protocol: ProxyType::Http,
                        country: "Unknown".to_string(),
                        country_code: "XX".to_string(),
                        anonymity: "unknown".to_string(),
                        speed: 0,
                        uptime: 0.0,
                        last_checked: chrono::Utc::now().to_rfc3339(),
                        provider: "ProxyScrape".to_string(),
                        is_working: false,
                    });
                }
            }
        }
        
        Ok(proxies)
    }

    async fn fetch_geonode(&self) -> Result<Vec<FreeProxy>> {
        #[derive(Deserialize)]
        struct GeoNodeResponse {
            data: Vec<GeoNodeProxy>,
        }

        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct GeoNodeProxy {
            ip: String,
            port: String,
            protocols: Vec<String>,
            country: String,
            #[serde(rename = "anonymityLevel")]
            anonymity_level: Option<String>,
            speed: Option<u32>,
            uptime: Option<f32>,
            #[serde(rename = "lastChecked")]
            last_checked: Option<i64>,
        }

        let url = "https://proxylist.geonode.com/api/proxy-list?limit=100&page=1&sort_by=lastChecked&sort_type=desc";
        let response: GeoNodeResponse = self.http_client.get_json(url).await?;

        let proxies = response.data.into_iter().filter_map(|p| {
            let port = p.port.parse::<u16>().ok()?;
            let protocol = if p.protocols.contains(&"socks5".to_string()) {
                ProxyType::Socks5
            } else if p.protocols.contains(&"socks4".to_string()) {
                ProxyType::Socks4
            } else {
                ProxyType::Http
            };

            Some(FreeProxy {
                ip: p.ip,
                port,
                protocol,
                country: p.country.clone(),
                country_code: p.country,
                anonymity: p.anonymity_level.unwrap_or_else(|| "unknown".to_string()),
                speed: p.speed.unwrap_or(0),
                uptime: p.uptime.unwrap_or(0.0),
                last_checked: chrono::Utc::now().to_rfc3339(),
                provider: "GeoNode".to_string(),
                is_working: false,
            })
        }).collect();

        Ok(proxies)
    }

    async fn fetch_pubproxy(&self) -> Result<Vec<FreeProxy>> {
        #[derive(Deserialize)]
        struct PubProxyResponse {
            data: Vec<PubProxyProxy>,
        }

        #[derive(Deserialize)]
        struct PubProxyProxy {
            ip: String,
            port: String,
            #[serde(rename = "type")]
            proxy_type: String,
            country: String,
            #[serde(rename = "proxy_level")]
            proxy_level: Option<String>,
            speed: Option<String>,
        }

        let url = "http://pubproxy.com/api/proxy?limit=20&format=json&type=http";
        let response: PubProxyResponse = self.http_client.get_json(url).await?;

        let proxies = response.data.into_iter().filter_map(|p| {
            let port = p.port.parse::<u16>().ok()?;
            let protocol = match p.proxy_type.to_lowercase().as_str() {
                "socks5" => ProxyType::Socks5,
                "socks4" => ProxyType::Socks4,
                _ => ProxyType::Http,
            };

            Some(FreeProxy {
                ip: p.ip,
                port,
                protocol,
                country: p.country.clone(),
                country_code: p.country,
                anonymity: p.proxy_level.unwrap_or_else(|| "unknown".to_string()),
                speed: p.speed.and_then(|s| s.parse().ok()).unwrap_or(0),
                uptime: 0.0,
                last_checked: chrono::Utc::now().to_rfc3339(),
                provider: "PubProxy".to_string(),
                is_working: false,
            })
        }).collect();

        Ok(proxies)
    }

    pub async fn test_proxy(&self, proxy: &FreeProxy) -> crate::proxy::ProxyTestResult {
        let settings = proxy.to_proxy_settings();
        
        let start = std::time::Instant::now();
        let result = match HttpClient::with_proxy(&settings) {
            Ok(client) => {
                match client.get("https://api.ipify.org?format=json").await {
                    Ok(response) => {
                        let latency = start.elapsed().as_millis() as u64;
                        #[derive(Deserialize)]
                        struct IpResponse { ip: String }
                        
                        let ip = serde_json::from_str::<IpResponse>(&response)
                            .map(|r| r.ip)
                            .ok();
                        
                        crate::proxy::ProxyTestResult {
                            proxy: proxy.clone(),
                            is_working: true,
                            latency_ms: Some(latency),
                            detected_ip: ip,
                            error: None,
                        }
                    }
                    Err(e) => crate::proxy::ProxyTestResult {
                        proxy: proxy.clone(),
                        is_working: false,
                        latency_ms: None,
                        detected_ip: None,
                        error: Some(e.to_string()),
                    }
                }
            }
            Err(e) => crate::proxy::ProxyTestResult {
                proxy: proxy.clone(),
                is_working: false,
                latency_ms: None,
                detected_ip: None,
                error: Some(e.to_string()),
            }
        };

        result
    }
}

impl Default for FreeIpProviderManager {
    fn default() -> Self {
        Self::new().expect("Failed to create FreeIpProviderManager")
    }
}
