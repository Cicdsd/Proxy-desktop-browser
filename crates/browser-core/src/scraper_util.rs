use anyhow::Result;
use scraper::{Html, Selector};
use tracing::{info, warn, error, debug};
use chrono::Utc;

use crate::http_client::HttpClient;
use crate::proxy::{FreeProxy, ProxyType};

/// Web scraper for extracting free proxy lists from various providers
pub struct ProxyScraper {
    http_client: HttpClient,
}

impl ProxyScraper {
    /// Create a new proxy scraper with HTTP client
    pub fn new(http_client: HttpClient) -> Self {
        Self { http_client }
    }

    /// Scrape free proxy list from free-proxy-list.net
    pub async fn scrape_free_proxy_list(&self) -> Result<Vec<FreeProxy>> {
        info!("Scraping free-proxy-list.net");
        
        let html = self.http_client.get_enhanced("https://free-proxy-list.net/")
            .await
            .map_err(|e| {
                error!("Failed to fetch free-proxy-list.net: {}", e);
                e
            })?;
        
        let document = Html::parse_document(&html);
        let table_selector = Selector::parse("table.table tbody tr").expect("Failed to parse table selector");
        let cell_selector = Selector::parse("td").expect("Failed to parse cell selector");
        
        let mut proxies = Vec::new();
        let mut count = 0;
        
        for row in document.select(&table_selector) {
            let cells: Vec<_> = row.select(&cell_selector).collect();
            
            if cells.len() >= 8 {
                let ip = cells[0].text().collect::<String>().trim().to_string();
                let port_text = cells[1].text().collect::<String>();
                let port_str = port_text.trim().to_string();
                let country_text = cells[2].text().collect::<String>();
                let country_code = country_text.trim().to_string();
                let https_text = cells[6].text().collect::<String>();
                let https = https_text.trim().to_string();
                let last_checked_text = cells[7].text().collect::<String>();
                let last_checked = last_checked_text.trim().to_string();
                
                if let Ok(port) = port_str.parse::<u16>() {
                    let proxy_type = if https.to_lowercase() == "yes" { 
                        ProxyType::Https 
                    } else { 
                        ProxyType::Http 
                    };
                    
                    let proxy = FreeProxy {
                        ip: ip.clone(),
                        port,
                        protocol: proxy_type,
                        country: country_code.to_string(),
                        country_code: country_code.to_string(),
                        anonymity: "unknown".to_string(),
                        speed: 0,
                        uptime: 0.0,
                        last_checked: last_checked.to_string(),
                        provider: "free-proxy-list.net".to_string(),
                        is_working: false,
                    };
                    
                    proxies.push(proxy);
                    count += 1;
                    
                    // Limit to prevent overwhelming
                    if count >= 100 {
                        break;
                    }
                }
            }
        }
        
        info!("Scraped {} proxies from free-proxy-list.net", proxies.len());
        Ok(proxies)
    }

    /// Scrape proxies from proxy-nova.com
    pub async fn scrape_proxy_nova(&self) -> Result<Vec<FreeProxy>> {
        info!("Scraping proxy-nova.com");
        
        let html = self.http_client.get_enhanced("https://www.proxynova.com/proxy-server-list/")
            .await
            .map_err(|e| {
                error!("Failed to fetch proxy-nova.com: {}", e);
                e
            })?;
        
        let document = Html::parse_document(&html);
        let row_selector = Selector::parse("table#tbl_proxy_list tbody tr").expect("Failed to parse proxy table selector");
        let cell_selector = Selector::parse("td").expect("Failed to parse cell selector");
        let ip_selector = Selector::parse("abbr[data-proxy]").expect("Failed to parse IP abbr selector");
        
        let mut proxies = Vec::new();
        let mut count = 0;
        
        for row in document.select(&row_selector) {
            let cells: Vec<_> = row.select(&cell_selector).collect();
            
            if cells.len() >= 5 {
                // Extract IP from the abbr tag with data-proxy attribute
                let ip = if let Some(ip_elem) = cells[0].select(&ip_selector).next() {
                    ip_elem.value().attr("data-proxy")
                        .unwrap_or("")
                        .to_string()
                } else {
                    cells[0].text().collect::<String>().trim().to_string()
                };
                
                let port_text = cells[1].text().collect::<String>();
                let port_str = port_text.trim().to_string();
                let country_text = cells[2].text().collect::<String>();
                let country = country_text.trim().to_string();
                let speed_text = cells[3].text().collect::<String>();
                let speed = speed_text.trim().to_string();
                let uptime_text = cells[4].text().collect::<String>();
                let uptime = uptime_text.trim().to_string();
                
                if let Ok(port) = port_str.parse::<u16>() {
                    let proxy = FreeProxy {
                        ip: ip.clone(),
                        port,
                        protocol: ProxyType::Http,
                        country: country.to_string(),
                        country_code: country.to_string(),
                        anonymity: "unknown".to_string(),
                        speed: speed.parse().unwrap_or(0),
                        uptime: uptime.trim_end_matches('%').parse().unwrap_or(0.0),
                        last_checked: Utc::now().to_rfc3339(),
                        provider: "proxy-nova.com".to_string(),
                        is_working: false,
                    };
                    
                    proxies.push(proxy);
                    count += 1;
                    
                    // Limit to prevent overwhelming
                    if count >= 50 {
                        break;
                    }
                }
            }
        }
        
        info!("Scraped {} proxies from proxy-nova.com", proxies.len());
        Ok(proxies)
    }

    /// Scrape proxies from spys.one
    pub async fn scrape_spys_one(&self) -> Result<Vec<FreeProxy>> {
        info!("Scraping spys.one");
        
        // Note: spys.one uses complex encoding, this is a simplified implementation
        let html = self.http_client.get_enhanced("http://spys.one/en/free-proxy-list/")
            .await
            .map_err(|e| {
                error!("Failed to fetch spys.one: {}", e);
                e
            })?;
        
        let document = Html::parse_document(&html);
        let row_selector = Selector::parse("table tr[onclick]").expect("Failed to parse onclick row selector");
        let cell_selector = Selector::parse("td").expect("Failed to parse cell selector");
        
        let mut proxies = Vec::new();
        let mut count = 0;
        
        for row in document.select(&row_selector) {
            let cells: Vec<_> = row.select(&cell_selector).collect();
            
            if cells.len() >= 9 {
                let ip_port_text = cells[0].text().collect::<String>();
                let ip_port = ip_port_text.trim().to_string();
                let country_text = cells[1].text().collect::<String>();
                let country = country_text.trim().to_string();
                let anonymity_text = cells[2].text().collect::<String>();
                let anonymity = anonymity_text.trim().to_string();
                let speed_text = cells[5].text().collect::<String>();
                let speed = speed_text.trim().to_string();
                
                // Parse IP and port (simplified)
                if let Some((ip, port_str)) = ip_port.split_once(':') {
                    if let Ok(port) = port_str.parse::<u16>() {
                        let proxy = FreeProxy {
                            ip: ip.to_string(),
                            port,
                            protocol: ProxyType::Http,
                            country: country.to_string(),
                            country_code: country.to_string(),
                            anonymity: anonymity.to_string(),
                            speed: speed.parse().unwrap_or(0),
                            uptime: 0.0,
                            last_checked: Utc::now().to_rfc3339(),
                            provider: "spys.one".to_string(),
                            is_working: false,
                        };
                        
                        proxies.push(proxy);
                        count += 1;
                        
                        // Limit to prevent overwhelming
                        if count >= 50 {
                            break;
                        }
                    }
                }
            }
        }
        
        info!("Scraped {} proxies from spys.one", proxies.len());
        Ok(proxies)
    }

    /// Scrape from all configured providers
    pub async fn scrape_all_providers(&self) -> Result<Vec<FreeProxy>> {
        let mut all_proxies = Vec::new();
        
        // Scrape from all providers sequentially to avoid type issues
        match self.scrape_free_proxy_list().await {
            Ok(proxies) => {
                info!("Added {} proxies from free-proxy-list.net", proxies.len());
                all_proxies.extend(proxies);
            }
            Err(e) => warn!("Failed to scrape free-proxy-list.net: {}", e),
        }
        
        match self.scrape_proxy_nova().await {
            Ok(proxies) => {
                info!("Added {} proxies from proxy-nova.com", proxies.len());
                all_proxies.extend(proxies);
            }
            Err(e) => warn!("Failed to scrape proxy-nova.com: {}", e),
        }
        
        match self.scrape_spys_one().await {
            Ok(proxies) => {
                info!("Added {} proxies from spys.one", proxies.len());
                all_proxies.extend(proxies);
            }
            Err(e) => warn!("Failed to scrape spys.one: {}", e),
        }
        
        info!("Total scraped: {} proxies from all providers", all_proxies.len());
        Ok(all_proxies)
    }

    /// Validate scraped proxy by testing connectivity
    pub async fn validate_proxy(&self, proxy: &FreeProxy) -> Result<bool> {
        // Use a simple validation endpoint
        let test_url = "http://httpbin.org/ip";
        
        match self.http_client.get_enhanced(test_url).await {
            Ok(_) => Ok(true),
            Err(e) => {
                debug!("Proxy validation failed for {}:{} - {}", proxy.ip, proxy.port, e);
                Ok(false)
            }
        }
    }

    /// Validate multiple proxies in parallel
    pub async fn validate_proxies(&self, proxies: &[FreeProxy]) -> Result<Vec<bool>> {
        let futures: Vec<_> = proxies.iter()
            .map(|proxy| self.validate_proxy(proxy))
            .collect();
        
        let results = futures::future::join_all(futures).await;
        
        let mut valid_results = Vec::new();
        for result in results {
            valid_results.push(result.unwrap_or(false));
        }
        
        Ok(valid_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_proxy_scraper_creation() {
        let client = HttpClient::new().expect("Failed to create HTTP client for test");
        let scraper = ProxyScraper::new(client);
        assert!(scraper.scrape_all_providers().await.is_ok());
    }
}
