//! Request Module
//!
//! Provides a unified API for making HTTP requests with:
//! - Multiple HTTP methods (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS)
//! - Request builder pattern
//! - Timeout configuration
//! - Header management
//! - Body handling (JSON, form data, raw bytes)
//! - Response parsing
//! - Error handling with detailed error types

use anyhow::{anyhow, Result};
use reqwest::{Client, Method};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;

use std::time::Duration;
use tracing::{debug, info};

use crate::proxy::ProxySettings;

/// Request error types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RequestErrorKind {
    /// Network connectivity error
    Network,
    /// DNS resolution failure
    DnsResolution,
    /// Request timeout
    Timeout,
    /// Invalid URL format
    InvalidUrl,
    /// Server returned an error status
    ServerError(u16),
    /// Failed to parse response
    ParseError,
    /// Request was cancelled
    Cancelled,
    /// Proxy connection error
    ProxyError,
    /// SSL/TLS error
    TlsError,
    /// Other errors
    Other,
}

/// Detailed request error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestError {
    /// The type of error
    pub kind: RequestErrorKind,
    /// Human-readable error message
    pub message: String,
    /// The URL that was being accessed
    pub url: Option<String>,
    /// HTTP status code if available
    pub status_code: Option<u16>,
}

impl RequestError {
    /// Create a new RequestError
    pub fn new(kind: RequestErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            url: None,
            status_code: None,
        }
    }

    /// Add URL to the error
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Add status code to the error
    pub fn with_status(mut self, status: u16) -> Self {
        self.status_code = Some(status);
        self
    }
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for RequestError {}

/// HTTP request method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl From<HttpMethod> for Method {
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::Get => Method::GET,
            HttpMethod::Post => Method::POST,
            HttpMethod::Put => Method::PUT,
            HttpMethod::Delete => Method::DELETE,
            HttpMethod::Patch => Method::PATCH,
            HttpMethod::Head => Method::HEAD,
            HttpMethod::Options => Method::OPTIONS,
        }
    }
}

/// Request body types
#[derive(Debug, Clone)]
pub enum RequestBody {
    /// No body
    None,
    /// JSON body
    Json(serde_json::Value),
    /// Form data
    Form(HashMap<String, String>),
    /// Raw bytes
    Bytes(Vec<u8>),
    /// Plain text
    Text(String),
}

/// Configuration for a request
#[derive(Debug, Clone)]
pub struct RequestConfig {
    /// Request timeout
    pub timeout: Duration,
    /// Follow redirects
    pub follow_redirects: bool,
    /// Maximum number of redirects
    pub max_redirects: u32,
    /// Verify SSL certificates
    pub verify_ssl: bool,
    /// User agent string
    pub user_agent: Option<String>,
}

impl Default for RequestConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            follow_redirects: true,
            max_redirects: 10,
            verify_ssl: true,
            user_agent: Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string()),
        }
    }
}

/// Response wrapper with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestResponse {
    /// HTTP status code
    pub status: u16,
    /// Status text
    pub status_text: String,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body as string
    pub body: String,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Final URL after redirects
    pub final_url: String,
}

impl RequestResponse {
    /// Check if the response was successful (2xx status)
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    /// Parse body as JSON
    pub fn json<T: DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_str(&self.body)
            .map_err(|e| anyhow!("Failed to parse JSON: {}", e))
    }
}

/// Request builder for constructing HTTP requests
#[derive(Debug, Clone)]
pub struct RequestBuilder {
    /// The request URL
    pub url: String,
    /// The HTTP method
    pub method: HttpMethod,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Request body
    pub body: RequestBody,
    /// Request configuration
    pub config: RequestConfig,
    /// Optional proxy settings
    pub proxy: Option<ProxySettings>,
}

impl RequestBuilder {
    /// Create a new GET request builder
    pub fn get(url: impl Into<String>) -> Self {
        Self::new(HttpMethod::Get, url)
    }

    /// Create a new POST request builder
    pub fn post(url: impl Into<String>) -> Self {
        Self::new(HttpMethod::Post, url)
    }

    /// Create a new PUT request builder
    pub fn put(url: impl Into<String>) -> Self {
        Self::new(HttpMethod::Put, url)
    }

    /// Create a new DELETE request builder
    pub fn delete(url: impl Into<String>) -> Self {
        Self::new(HttpMethod::Delete, url)
    }

    /// Create a new PATCH request builder
    pub fn patch(url: impl Into<String>) -> Self {
        Self::new(HttpMethod::Patch, url)
    }

    /// Create a new HEAD request builder
    pub fn head(url: impl Into<String>) -> Self {
        Self::new(HttpMethod::Head, url)
    }

    /// Create a new OPTIONS request builder
    pub fn options(url: impl Into<String>) -> Self {
        Self::new(HttpMethod::Options, url)
    }

    /// Create a new request builder with the specified method
    pub fn new(method: HttpMethod, url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method,
            headers: HashMap::new(),
            body: RequestBody::None,
            config: RequestConfig::default(),
            proxy: None,
        }
    }

    /// Set a header
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Set multiple headers
    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers.extend(headers);
        self
    }

    /// Set JSON body
    pub fn json<T: Serialize>(mut self, body: &T) -> Result<Self> {
        let json_value = serde_json::to_value(body)?;
        self.body = RequestBody::Json(json_value);
        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
        Ok(self)
    }

    /// Set form body
    pub fn form(mut self, data: HashMap<String, String>) -> Self {
        self.body = RequestBody::Form(data);
        self.headers.insert("Content-Type".to_string(), "application/x-www-form-urlencoded".to_string());
        self
    }

    /// Set raw body
    pub fn body_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.body = RequestBody::Bytes(bytes);
        self
    }

    /// Set text body
    pub fn body_text(mut self, text: impl Into<String>) -> Self {
        self.body = RequestBody::Text(text.into());
        self.headers.insert("Content-Type".to_string(), "text/plain".to_string());
        self
    }

    /// Set timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set whether to follow redirects
    pub fn follow_redirects(mut self, follow: bool) -> Self {
        self.config.follow_redirects = follow;
        self
    }

    /// Set proxy settings
    pub fn proxy(mut self, proxy: ProxySettings) -> Self {
        self.proxy = Some(proxy);
        self
    }

    /// Set user agent
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.config.user_agent = Some(user_agent.into());
        self
    }

    /// Send the request
    pub async fn send(self) -> Result<RequestResponse> {
        let start_time = std::time::Instant::now();
        
        debug!("Sending {} request to: {}", format!("{:?}", self.method), self.url);

        // Build the client
        let mut client_builder = Client::builder()
            .timeout(self.config.timeout)
            .redirect(if self.config.follow_redirects {
                reqwest::redirect::Policy::limited(self.config.max_redirects as usize)
            } else {
                reqwest::redirect::Policy::none()
            });

        // Add proxy if configured
        if let Some(proxy_settings) = &self.proxy {
            if let Some(proxy_url) = proxy_settings.to_url() {
                let proxy = reqwest::Proxy::all(&proxy_url)
                    .map_err(|e| anyhow!("Invalid proxy URL: {}", e))?;
                client_builder = client_builder.proxy(proxy);
            }
        }

        // Set user agent
        if let Some(ua) = &self.config.user_agent {
            client_builder = client_builder.user_agent(ua);
        }

        let client = client_builder.build()
            .map_err(|e| anyhow!("Failed to build HTTP client: {}", e))?;

        // Build the request
        let method: Method = self.method.into();
        let mut request_builder = client.request(method, &self.url);

        // Add headers
        for (name, value) in &self.headers {
            request_builder = request_builder.header(name, value);
        }

        // Add body
        request_builder = match self.body {
            RequestBody::None => request_builder,
            RequestBody::Json(json) => request_builder.json(&json),
            RequestBody::Form(form) => request_builder.form(&form),
            RequestBody::Bytes(bytes) => request_builder.body(bytes),
            RequestBody::Text(text) => request_builder.body(text),
        };

        // Send the request
        let response = request_builder.send().await
            .map_err(|e| {
                let error_msg = e.to_string().to_lowercase();
                let kind = if e.is_timeout() {
                    RequestErrorKind::Timeout
                } else if error_msg.contains("dns") 
                    || error_msg.contains("resolve") 
                    || error_msg.contains("no such host") 
                    || error_msg.contains("name or service not known")
                    || error_msg.contains("getaddrinfo") {
                    // DNS resolution failure - provide helpful error message
                    tracing::warn!("DNS resolution failed for URL: {}", &self.url);
                    RequestErrorKind::DnsResolution
                } else if e.is_connect() {
                    RequestErrorKind::Network
                } else {
                    RequestErrorKind::Other
                };
                anyhow!(RequestError::new(kind, e.to_string()).with_url(&self.url))
            })?;

        let status = response.status().as_u16();
        let status_text = response.status().canonical_reason().unwrap_or("Unknown").to_string();
        let final_url = response.url().to_string();
        
        // Extract headers
        let mut headers = HashMap::new();
        for (name, value) in response.headers() {
            if let Ok(v) = value.to_str() {
                headers.insert(name.to_string(), v.to_string());
            }
        }

        // Read body
        let body = response.text().await
            .map_err(|e| anyhow!("Failed to read response body: {}", e))?;

        let response_time_ms = start_time.elapsed().as_millis() as u64;

        info!("Request completed: {} {} - {} in {}ms", 
            format!("{:?}", self.method), self.url, status, response_time_ms);

        Ok(RequestResponse {
            status,
            status_text,
            headers,
            body,
            response_time_ms,
            final_url,
        })
    }
}

/// Request manager for handling multiple requests
#[allow(dead_code)]
pub struct RequestManager {
    client: Client,
    default_config: RequestConfig,
    default_proxy: Option<ProxySettings>,
}

impl RequestManager {
    /// Create a new request manager
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            client,
            default_config: RequestConfig::default(),
            default_proxy: None,
        })
    }

    /// Create with proxy settings
    pub fn with_proxy(proxy: ProxySettings) -> Result<Self> {
        let mut manager = Self::new()?;
        manager.default_proxy = Some(proxy);
        Ok(manager)
    }

    /// Set default configuration
    pub fn set_default_config(&mut self, config: RequestConfig) {
        self.default_config = config;
    }

    /// Set default proxy
    pub fn set_default_proxy(&mut self, proxy: Option<ProxySettings>) {
        self.default_proxy = proxy;
    }

    /// Create a GET request builder
    pub fn get(&self, url: impl Into<String>) -> RequestBuilder {
        let mut builder = RequestBuilder::get(url);
        if let Some(proxy) = &self.default_proxy {
            builder = builder.proxy(proxy.clone());
        }
        builder.timeout(self.default_config.timeout)
    }

    /// Create a POST request builder
    pub fn post(&self, url: impl Into<String>) -> RequestBuilder {
        let mut builder = RequestBuilder::post(url);
        if let Some(proxy) = &self.default_proxy {
            builder = builder.proxy(proxy.clone());
        }
        builder.timeout(self.default_config.timeout)
    }

    /// Create a PUT request builder
    pub fn put(&self, url: impl Into<String>) -> RequestBuilder {
        let mut builder = RequestBuilder::put(url);
        if let Some(proxy) = &self.default_proxy {
            builder = builder.proxy(proxy.clone());
        }
        builder.timeout(self.default_config.timeout)
    }

    /// Create a DELETE request builder
    pub fn delete(&self, url: impl Into<String>) -> RequestBuilder {
        let mut builder = RequestBuilder::delete(url);
        if let Some(proxy) = &self.default_proxy {
            builder = builder.proxy(proxy.clone());
        }
        builder.timeout(self.default_config.timeout)
    }

    /// Simple GET request
    pub async fn simple_get(&self, url: &str) -> Result<String> {
        let response = self.get(url).send().await?;
        if response.is_success() {
            Ok(response.body)
        } else {
            Err(anyhow!("Request failed with status: {}", response.status))
        }
    }

    /// Simple GET request returning JSON
    pub async fn simple_get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response = self.get(url).send().await?;
        if response.is_success() {
            response.json()
        } else {
            Err(anyhow!("Request failed with status: {}", response.status))
        }
    }

    /// Simple POST request with JSON body
    pub async fn simple_post_json<T: Serialize, R: DeserializeOwned>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<R> {
        let response = self.post(url).json(body)?.send().await?;
        if response.is_success() {
            response.json()
        } else {
            Err(anyhow!("Request failed with status: {}", response.status))
        }
    }
}

impl Default for RequestManager {
    fn default() -> Self {
        Self::new().expect("Failed to create RequestManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_builder_creation() {
        let builder = RequestBuilder::get("https://example.com");
        assert_eq!(builder.method, HttpMethod::Get);
        assert_eq!(builder.url, "https://example.com");
    }

    #[test]
    fn test_request_builder_with_headers() {
        let builder = RequestBuilder::get("https://example.com")
            .header("Authorization", "Bearer token")
            .header("Accept", "application/json");
        
        assert_eq!(builder.headers.get("Authorization").unwrap(), "Bearer token");
        assert_eq!(builder.headers.get("Accept").unwrap(), "application/json");
    }

    #[test]
    fn test_request_config_default() {
        let config = RequestConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert!(config.follow_redirects);
        assert_eq!(config.max_redirects, 10);
        assert!(config.verify_ssl);
    }

    #[test]
    fn test_request_response_is_success() {
        let response = RequestResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: "{}".to_string(),
            response_time_ms: 100,
            final_url: "https://example.com".to_string(),
        };
        assert!(response.is_success());

        let error_response = RequestResponse {
            status: 404,
            status_text: "Not Found".to_string(),
            headers: HashMap::new(),
            body: "{}".to_string(),
            response_time_ms: 100,
            final_url: "https://example.com".to_string(),
        };
        assert!(!error_response.is_success());
    }

    #[test]
    fn test_http_method_conversion() {
        assert_eq!(Method::from(HttpMethod::Get), Method::GET);
        assert_eq!(Method::from(HttpMethod::Post), Method::POST);
        assert_eq!(Method::from(HttpMethod::Put), Method::PUT);
        assert_eq!(Method::from(HttpMethod::Delete), Method::DELETE);
    }
}
