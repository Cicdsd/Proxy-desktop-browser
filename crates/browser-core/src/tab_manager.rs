use crate::fingerprint::BrowserFingerprint;
use crate::tab_isolation::{NetworkConfig, TabProfile, TabStatus};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::time::SystemTime;
use tokio::sync::RwLock;
use uuid::Uuid;
use virtual_ip::{IPGenerator, VirtualIP};
use tracing::{info, debug};

pub struct TabIPManager {
    tabs: RwLock<HashMap<String, TabProfile>>,
    ip_generator: IPGenerator,
}

impl TabIPManager {
    /// Create new TabIPManager with in-memory storage
    pub fn new(ip_generator: IPGenerator) -> Self {
        Self {
            tabs: RwLock::new(HashMap::new()),
            ip_generator,
        }
    }

    /// Create new tab with IP from specific country
    pub async fn create_tab(&self, country_code: &str) -> Result<TabProfile> {
        let tab_id = Uuid::new_v4().to_string();
        let virtual_ip = self.ip_generator.generate_for_country(country_code)?;
        let fingerprint = self.generate_matching_fingerprint(&virtual_ip)?;
        let network_config = self.create_network_config(&virtual_ip)?;

        let tab_profile = TabProfile {
            tab_id: tab_id.clone(),
            virtual_ip,
            fingerprint,
            network_config,
            storage_path: format!("./data/tabs/{}", tab_id),
            process_id: None,
            created_at: SystemTime::now(),
            last_active: SystemTime::now(),
            status: TabStatus::Creating,
        };

        // Save to in-memory cache
        self.tabs
            .write()
            .await
            .insert(tab_id.clone(), tab_profile.clone());

        info!("Created tab {} for country {}", tab_id, country_code);
        Ok(tab_profile)
    }

    /// Create tab with random country
    pub async fn create_tab_random(&self) -> Result<TabProfile> {
        let virtual_ip = self.ip_generator.generate_random()?;
        self.create_tab(&virtual_ip.country_code).await
    }

    /// Rotate IP for existing tab
    pub async fn rotate_ip(&self, tab_id: &str, new_country_code: Option<&str>) -> Result<VirtualIP> {
        let mut tabs = self.tabs.write().await;
        let tab = tabs
            .get_mut(tab_id)
            .ok_or_else(|| anyhow!("Tab not found"))?;

        let new_ip = if let Some(country) = new_country_code {
            self.ip_generator.generate_for_country(country)?
        } else {
            self.ip_generator
                .generate_for_country(&tab.virtual_ip.country_code)?
        };

        tab.virtual_ip = new_ip.clone();
        tab.fingerprint = self.generate_matching_fingerprint(&new_ip)?;
        tab.network_config = self.create_network_config(&new_ip)?;
        tab.last_active = SystemTime::now();

        info!("Rotated IP for tab {} to {}", tab_id, new_ip.ip);
        Ok(new_ip)
    }

    /// Get tab by ID
    pub async fn get_tab(&self, tab_id: &str) -> Option<TabProfile> {
        self.tabs.read().await.get(tab_id).cloned()
    }

    /// List all active tabs
    pub async fn list_tabs(&self) -> Vec<TabProfile> {
        self.tabs.read().await.values().cloned().collect()
    }

    /// Close tab
    pub async fn close_tab(&self, tab_id: &str) -> Result<()> {
        // Remove from in-memory cache
        self.tabs.write().await.remove(tab_id);
        
        info!("Tab {} closed successfully", tab_id);
        Ok(())
    }

    /// Navigate an existing tab
    pub async fn navigate(&self, tab_id: &str, _url: &str) -> Result<()> {
        let mut tabs = self.tabs.write().await;
        let tab = tabs
            .get_mut(tab_id)
            .ok_or_else(|| anyhow!("Tab not found"))?;
        tab.last_active = SystemTime::now();
        
        debug!("Tab {} navigated", tab_id);
        Ok(())
    }

    fn generate_matching_fingerprint(&self, ip: &VirtualIP) -> Result<BrowserFingerprint> {
        Ok(BrowserFingerprint {
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string(),
            accept_language: ip.language.clone(),
            timezone: ip.timezone.clone(),
            screen_resolution: (1920, 1080),
            color_depth: 24,
            hardware_concurrency: 8,
            device_memory: 8,
            platform: "Win32".to_string(),
            webgl_vendor: "Google Inc.".to_string(),
            webgl_renderer: "ANGLE (Intel HD Graphics)".to_string(),
            canvas_hash: self.generate_canvas_hash(),
            audio_hash: self.generate_audio_hash(),
            fonts: vec![
                "Arial".to_string(),
                "Helvetica".to_string(),
                "Times New Roman".to_string(),
            ],
        })
    }

    fn create_network_config(&self, ip: &VirtualIP) -> Result<NetworkConfig> {
        Ok(NetworkConfig {
            dns_servers: vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()],
            proxy_url: None,
            bypass_list: vec![],
            force_ipv4: true,
            custom_headers: HashMap::from([
                ("Accept-Language".to_string(), ip.language.clone()),
                (
                    "Accept".to_string(),
                    "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string(),
                ),
            ]),
        })
    }

    fn generate_canvas_hash(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        format!("{:x}", rng.gen::<u64>())
    }

    fn generate_audio_hash(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        format!("{:x}", rng.gen::<u64>())
    }
}
