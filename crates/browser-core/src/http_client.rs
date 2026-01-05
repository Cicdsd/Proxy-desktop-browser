use anyhow::{anyhow, Result};
use reqwest::{Client, Proxy};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;
use governor::{Quota, RateLimiter, state::{NotKeyed, InMemoryState}, clock::DefaultClock};
use std::num::NonZeroU32;

use crate::proxy::ProxySettings;

pub struct HttpClient {
    client: Client,
    enhanced_client: Option<ClientWithMiddleware>,
    rate_limiter: Option<Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>,
}

impl HttpClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        
        // Also create enhanced client with middleware
        let retry_policy = ExponentialBackoff::builder()
            .retry_bounds(Duration::from_secs(1), Duration::from_secs(30))
            .build_with_max_retries(3);
        
        let enhanced_client = ClientBuilder::new(client.clone())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();
        
        let rate_limiter = Arc::new(RateLimiter::direct(
            Quota::per_second(NonZeroU32::new(10).expect("10 is non-zero"))
                .allow_burst(NonZeroU32::new(20).expect("20 is non-zero"))
        ));
        
        Ok(Self { 
            client,
            enhanced_client: Some(enhanced_client),
            rate_limiter: Some(rate_limiter),
        })
    }

    pub fn with_proxy(proxy_settings: &ProxySettings) -> Result<Self> {
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(30));

        if let Some(proxy_url) = proxy_settings.to_url() {
            let proxy = Proxy::all(&proxy_url)?;
            builder = builder.proxy(proxy);
        }

        let client = builder.build()?;
        
        // Create enhanced client with middleware for proxy client too
        let retry_policy = ExponentialBackoff::builder()
            .retry_bounds(Duration::from_secs(1), Duration::from_secs(30))
            .build_with_max_retries(3);
        
        let enhanced_client = ClientBuilder::new(client.clone())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();
        
        let rate_limiter = Arc::new(RateLimiter::direct(
            Quota::per_second(NonZeroU32::new(10).expect("10 is non-zero"))
                .allow_burst(NonZeroU32::new(20).expect("20 is non-zero"))
        ));
        
        Ok(Self { 
            client,
            enhanced_client: Some(enhanced_client),
            rate_limiter: Some(rate_limiter),
        })
    }

    pub async fn get(&self, url: &str) -> Result<String> {
        let response = self.client.get(url).send().await?;
        let text = response.text().await?;
        Ok(text)
    }

    /// Enhanced GET with retry and rate limiting
    pub async fn get_enhanced(&self, url: &str) -> Result<String> {
        if let (Some(enhanced_client), Some(rate_limiter)) = (&self.enhanced_client, &self.rate_limiter) {
            // Wait for rate limit
            rate_limiter.until_ready().await;
            
            debug!("Making enhanced GET request to: {}", url);
            
            let response = enhanced_client
                .get(url)
                .send()
                .await
                .map_err(|e| anyhow!("Enhanced GET request failed: {}", e))?;
            
            response.text().await
                .map_err(|e| anyhow!("Failed to read response text: {}", e))
        } else {
            // Fallback to regular client
            self.get(url).await
        }
    }

    pub async fn get_json<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T> {
        let response = self.client.get(url).send().await?;
        let json = response.json::<T>().await?;
        Ok(json)
    }

    /// Enhanced GET with retry and rate limiting, returns JSON
    pub async fn get_json_enhanced<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T> {
        if let (Some(enhanced_client), Some(rate_limiter)) = (&self.enhanced_client, &self.rate_limiter) {
            // Wait for rate limit
            rate_limiter.until_ready().await;
            
            debug!("Making enhanced GET JSON request to: {}", url);
            
            let response = enhanced_client
                .get(url)
                .send()
                .await
                .map_err(|e| anyhow!("Enhanced GET request failed: {}", e))?;
            
            response.json::<T>().await
                .map_err(|e| anyhow!("Failed to parse JSON: {}", e))
        } else {
            // Fallback to regular client
            self.get_json(url).await
        }
    }

    /// Enhanced POST with retry and rate limiting
    pub async fn post_enhanced(&self, url: &str, body: String) -> Result<String> {
        if let (Some(enhanced_client), Some(rate_limiter)) = (&self.enhanced_client, &self.rate_limiter) {
            // Wait for rate limit
            rate_limiter.until_ready().await;
            
            debug!("Making enhanced POST request to: {}", url);
            
            let response = enhanced_client
                .post(url)
                .header("Content-Type", "application/json")
                .body(body)
                .send()
                .await
                .map_err(|e| anyhow!("Enhanced POST request failed: {}", e))?;
            
            response.text().await
                .map_err(|e| anyhow!("Failed to read response text: {}", e))
        } else {
            // Fallback to regular client
            let response = self.client.post(url)
                .header("Content-Type", "application/json")
                .body(body)
                .send()
                .await?;
            response.text().await
                .map_err(|e| anyhow!("Failed to read response text: {}", e))
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create HTTP client")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicIpInfo {
    pub ip: String,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub isp: Option<String>,
    pub timezone: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
}

pub struct PublicIpDetector {
    http_client: HttpClient,
}

impl PublicIpDetector {
    pub fn new() -> Result<Self> {
        Ok(Self {
            http_client: HttpClient::new()?,
        })
    }

    pub fn with_proxy(proxy_settings: &ProxySettings) -> Result<Self> {
        Ok(Self {
            http_client: HttpClient::with_proxy(proxy_settings)?,
        })
    }

    pub async fn detect_ip(&self) -> Result<PublicIpInfo> {
        // Try multiple IP detection services
        let services: Vec<(&str, fn(&str) -> Result<PublicIpInfo>)> = vec![
            ("https://ipapi.co/json/", Self::parse_ipapi_co as fn(&str) -> Result<PublicIpInfo>),
            ("https://api.ipify.org?format=json", Self::parse_ipify as fn(&str) -> Result<PublicIpInfo>),
            ("https://ipinfo.io/json", Self::parse_ipinfo as fn(&str) -> Result<PublicIpInfo>),
        ];

        for (url, parser) in services {
            match self.http_client.get(url).await {
                Ok(response) => {
                    if let Ok(info) = parser(&response) {
                        return Ok(info);
                    }
                }
                Err(_) => continue,
            }
        }

        Err(anyhow!("Failed to detect public IP from any service"))
    }

    fn parse_ipapi_co(response: &str) -> Result<PublicIpInfo> {
        #[derive(Deserialize)]
        struct IpApiCoResponse {
            ip: String,
            country_name: Option<String>,
            country_code: Option<String>,
            city: Option<String>,
            region: Option<String>,
            org: Option<String>,
            timezone: Option<String>,
            latitude: Option<f64>,
            longitude: Option<f64>,
        }

        let data: IpApiCoResponse = serde_json::from_str(response)?;
        Ok(PublicIpInfo {
            ip: data.ip,
            country: data.country_name,
            country_code: data.country_code,
            city: data.city,
            region: data.region,
            isp: data.org,
            timezone: data.timezone,
            lat: data.latitude,
            lon: data.longitude,
        })
    }

    fn parse_ipify(response: &str) -> Result<PublicIpInfo> {
        #[derive(Deserialize)]
        struct IpifyResponse {
            ip: String,
        }

        let data: IpifyResponse = serde_json::from_str(response)?;
        Ok(PublicIpInfo {
            ip: data.ip,
            country: None,
            country_code: None,
            city: None,
            region: None,
            isp: None,
            timezone: None,
            lat: None,
            lon: None,
        })
    }

    fn parse_ipinfo(response: &str) -> Result<PublicIpInfo> {
        #[derive(Deserialize)]
        struct IpInfoResponse {
            ip: String,
            country: Option<String>,
            city: Option<String>,
            region: Option<String>,
            org: Option<String>,
            timezone: Option<String>,
            loc: Option<String>,
        }

        let data: IpInfoResponse = serde_json::from_str(response)?;
        
        let (lat, lon) = if let Some(loc) = &data.loc {
            let parts: Vec<&str> = loc.split(',').collect();
            if parts.len() == 2 {
                (parts[0].parse().ok(), parts[1].parse().ok())
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        Ok(PublicIpInfo {
            ip: data.ip,
            country: None,
            country_code: data.country,
            city: data.city,
            region: data.region,
            isp: data.org,
            timezone: data.timezone,
            lat,
            lon,
        })
    }
}

impl Default for PublicIpDetector {
    fn default() -> Self {
        Self::new().expect("Failed to create IP detector")
    }
}
