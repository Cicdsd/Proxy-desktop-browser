use anyhow::{anyhow, Result};
use base64::engine::Engine;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::proxy::ProxySettings;

// ============================================================================
// Shared Utility Functions
// ============================================================================

/// Build a CONNECT request for HTTP proxy tunneling
fn build_connect_request(host: &str, port: u16, proxy: &ProxySettings) -> String {
    let mut request = format!(
        "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\n",
        host, port, host, port
    );

    // Add proxy authentication if configured
    if let (Some(username), Some(password)) = (&proxy.username, &proxy.password) {
        let credentials = format!("{}:{}", username, password);
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
        request.push_str(&format!("Proxy-Authorization: Basic {}\r\n", encoded));
    }

    request.push_str("\r\n");
    request
}

/// Send CONNECT request and verify the response
async fn send_connect_request(
    stream: &mut TcpStream,
    host: &str,
    port: u16,
    proxy: &ProxySettings,
) -> Result<()> {
    let request = build_connect_request(host, port, proxy);
    stream.write_all(request.as_bytes()).await?;

    // Read proxy response
    let mut response_buf = vec![0u8; 1024];
    let n = stream.read(&mut response_buf).await?;
    let response = String::from_utf8_lossy(&response_buf[..n]);

    // Check if connection was established
    if !response.contains("200") {
        return Err(anyhow!("Proxy CONNECT failed: {}", response));
    }

    Ok(())
}

/// Extract proxy address from ProxySettings
fn get_proxy_address(proxy: &ProxySettings) -> Result<String> {
    let host = proxy.host.as_ref()
        .ok_or_else(|| anyhow!("Proxy host not set"))?;
    let port = proxy.port
        .ok_or_else(|| anyhow!("Proxy port not set"))?;
    Ok(format!("{}:{}", host, port))
}

/// Connect to a proxy server
async fn connect_to_proxy(proxy: &ProxySettings) -> Result<TcpStream> {
    let proxy_addr = get_proxy_address(proxy)?;
    TcpStream::connect(&proxy_addr)
        .await
        .map_err(|e| anyhow!("Failed to connect to proxy {} - {}", proxy_addr, e))
}

/// Forward data from reader to writer until EOF or error
async fn forward_data<R, W>(mut reader: R, mut writer: W)
where
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    let mut buffer = vec![0u8; 8192];
    loop {
        match reader.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => {
                if writer.write_all(&buffer[..n]).await.is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

/// Bidirectional data forwarding between two streams
async fn forward_bidirectional(client_stream: TcpStream, target_stream: TcpStream) {
    let (client_read, client_write) = client_stream.into_split();
    let (target_read, target_write) = target_stream.into_split();

    // Run both directions concurrently until one completes
    tokio::select! {
        _ = forward_data(client_read, target_write) => {}
        _ = forward_data(target_read, client_write) => {}
    }
}

// ============================================================================
// Local Proxy Server
// ============================================================================

/// Local proxy server for routing tab traffic through upstream proxies
pub struct LocalProxyServer {
    /// Local address to bind to
    bind_addr: SocketAddr,
    /// Upstream proxy configuration
    upstream_proxy: Option<ProxySettings>,
    /// Active connections
    connections: Arc<RwLock<HashMap<String, ProxyConnection>>>,
    /// Running state
    is_running: Arc<RwLock<bool>>,
}

/// Represents an active proxy connection
#[derive(Debug, Clone)]
pub struct ProxyConnection {
    pub id: String,
    pub client_addr: String,
    pub target_host: String,
    pub target_port: u16,
    pub upstream_proxy: Option<ProxySettings>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl LocalProxyServer {
    /// Create a new local proxy server
    pub fn new(bind_port: u16, upstream_proxy: Option<ProxySettings>) -> Result<Self> {
        let bind_addr = format!("127.0.0.1:{}", bind_port)
            .parse()
            .map_err(|e| anyhow!("Invalid bind address: {}", e))?;

        Ok(Self {
            bind_addr,
            upstream_proxy,
            connections: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
        })
    }

    /// Start the local proxy server
    pub async fn start(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(anyhow!("Proxy server is already running"));
        }

        let listener = TcpListener::bind(&self.bind_addr)
            .await
            .map_err(|e| anyhow!("Failed to bind to {}: {}", self.bind_addr, e))?;

        info!("Local proxy server listening on {}", self.bind_addr);
        *is_running = true;
        drop(is_running);

        let connections = self.connections.clone();
        let upstream_proxy = self.upstream_proxy.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            while *is_running.read().await {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        debug!("New connection from {}", addr);
                        let conn_id = Uuid::new_v4().to_string();
                        let connections_clone = connections.clone();
                        let upstream_proxy_clone = upstream_proxy.clone();

                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(
                                stream,
                                addr.to_string(),
                                conn_id.clone(),
                                upstream_proxy_clone,
                                connections_clone,
                            )
                            .await
                            {
                                error!("Error handling connection {}: {}", conn_id, e);
                            }
                        });
                    }
                    Err(e) => {
                        if *is_running.read().await {
                            error!("Error accepting connection: {}", e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the local proxy server
    pub async fn stop(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;

        // Clear all connections
        let mut connections = self.connections.write().await;
        connections.clear();

        info!("Local proxy server stopped");
        Ok(())
    }

    /// Get the proxy URL for configuring WebView
    pub fn get_proxy_url(&self) -> String {
        format!("http://{}", self.bind_addr)
    }

    /// Handle an incoming proxy connection
    async fn handle_connection(
        mut client_stream: TcpStream,
        client_addr: String,
        conn_id: String,
        upstream_proxy: Option<ProxySettings>,
        connections: Arc<RwLock<HashMap<String, ProxyConnection>>>,
    ) -> Result<()> {
        // Read HTTP CONNECT request
        let mut buffer = vec![0u8; 4096];
        let n = client_stream.read(&mut buffer).await?;
        let request = String::from_utf8_lossy(&buffer[..n]);

        // Parse CONNECT request
        let (target_host, target_port) = Self::parse_connect_request(&request)?;

        // Record the connection
        {
            let mut conns = connections.write().await;
            conns.insert(
                conn_id.clone(),
                ProxyConnection {
                    id: conn_id.clone(),
                    client_addr: client_addr.clone(),
                    target_host: target_host.clone(),
                    target_port,
                    upstream_proxy: upstream_proxy.clone(),
                    created_at: chrono::Utc::now(),
                },
            );
        }

        // Send 200 Connection established response
        let response = "HTTP/1.1 200 Connection Established\r\n\r\n";
        client_stream.write_all(response.as_bytes()).await?;

        // Connect to upstream proxy if configured, otherwise direct connection
        let target_stream = if let Some(ref proxy) = upstream_proxy {
            Self::connect_through_upstream(proxy, &target_host, target_port).await?
        } else {
            Self::connect_direct(&target_host, target_port).await?
        };

        // Forward data bidirectionally
        forward_bidirectional(client_stream, target_stream).await;

        // Remove connection from active list
        {
            let mut conns = connections.write().await;
            conns.remove(&conn_id);
        }

        debug!("Connection {} closed", conn_id);
        Ok(())
    }

    /// Parse HTTP CONNECT request to extract target host and port
    fn parse_connect_request(request: &str) -> Result<(String, u16)> {
        let lines: Vec<&str> = request.lines().collect();
        if lines.is_empty() {
            return Err(anyhow!("Empty request"));
        }

        let parts: Vec<&str> = lines[0].split_whitespace().collect();
        if parts.len() < 2 || parts[0] != "CONNECT" {
            return Err(anyhow!("Invalid CONNECT request"));
        }

        let target = parts[1];
        let target_parts: Vec<&str> = target.split(':').collect();
        if target_parts.len() != 2 {
            return Err(anyhow!("Invalid target format"));
        }

        let host = target_parts[0].to_string();
        let port = target_parts[1]
            .parse::<u16>()
            .map_err(|_| anyhow!("Invalid port"))?;

        Ok((host, port))
    }

    /// Connect directly to target host
    async fn connect_direct(host: &str, port: u16) -> Result<TcpStream> {
        let target_addr = format!("{}:{}", host, port);
        TcpStream::connect(target_addr)
            .await
            .map_err(|e| anyhow!("Failed to connect to {}:{} - {}", host, port, e))
    }

    /// Connect through upstream proxy
    async fn connect_through_upstream(
        proxy: &ProxySettings,
        target_host: &str,
        target_port: u16,
    ) -> Result<TcpStream> {
        let mut proxy_stream = connect_to_proxy(proxy).await?;

        // Send CONNECT request and verify response
        send_connect_request(&mut proxy_stream, target_host, target_port, proxy).await?;

        Ok(proxy_stream)
    }

    /// Get active connections
    pub async fn get_active_connections(&self) -> Vec<ProxyConnection> {
        self.connections.read().await.values().cloned().collect()
    }

    /// Check if the proxy server is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }
}

// ============================================================================
// Local Proxy Manager
// ============================================================================

/// Manager for multiple local proxy servers (one per tab)
pub struct LocalProxyManager {
    /// Active proxy servers by tab ID
    proxy_servers: Arc<RwLock<HashMap<String, Arc<LocalProxyServer>>>>,
    /// Port range for local proxies
    port_range: std::ops::Range<u16>,
    /// Currently used ports
    used_ports: Arc<RwLock<std::collections::HashSet<u16>>>,
}

impl LocalProxyManager {
    /// Create a new local proxy manager
    pub fn new(port_range: std::ops::Range<u16>) -> Self {
        Self {
            proxy_servers: Arc::new(RwLock::new(HashMap::new())),
            port_range,
            used_ports: Arc::new(RwLock::new(std::collections::HashSet::new())),
        }
    }

    /// Create a proxy server for a specific tab
    pub async fn create_proxy_for_tab(
        &self,
        tab_id: &str,
        upstream_proxy: Option<ProxySettings>,
    ) -> Result<String> {
        // Find an available port
        let port = self.find_available_port().await?;

        // Create and start the proxy server
        let proxy_server = Arc::new(LocalProxyServer::new(port, upstream_proxy)?);
        proxy_server.start().await?;

        // Register the proxy server
        {
            let mut servers = self.proxy_servers.write().await;
            servers.insert(tab_id.to_string(), proxy_server.clone());
        }

        {
            let mut used_ports = self.used_ports.write().await;
            used_ports.insert(port);
        }

        let proxy_url = proxy_server.get_proxy_url();
        info!("Created proxy for tab {} on {}", tab_id, proxy_url);

        Ok(proxy_url)
    }

    /// Remove proxy server for a tab
    pub async fn remove_proxy_for_tab(&self, tab_id: &str) -> Result<()> {
        let proxy_server = {
            let mut servers = self.proxy_servers.write().await;
            servers.remove(tab_id)
        };

        if let Some(server) = proxy_server {
            server.stop().await?;

            // Extract port from bind address and mark as available
            let addr_str = server.get_proxy_url();
            if let Some(port_str) = addr_str.split(':').nth(1) {
                if let Ok(port) = port_str.parse::<u16>() {
                    let mut used_ports = self.used_ports.write().await;
                    used_ports.remove(&port);
                }
            }

            info!("Removed proxy for tab {}", tab_id);
        }

        Ok(())
    }

    /// Get proxy URL for a tab
    pub async fn get_proxy_url_for_tab(&self, tab_id: &str) -> Option<String> {
        let servers = self.proxy_servers.read().await;
        servers.get(tab_id).map(|server| server.get_proxy_url())
    }

    /// Find an available port in the configured range
    async fn find_available_port(&self) -> Result<u16> {
        let used_ports = self.used_ports.read().await;

        for port in self.port_range.clone() {
            if !used_ports.contains(&port) {
                return Ok(port);
            }
        }

        Err(anyhow!("No available ports in range {:?}", self.port_range))
    }

    /// Get all active proxy servers
    pub async fn get_active_proxies(&self) -> HashMap<String, String> {
        let servers = self.proxy_servers.read().await;
        servers
            .iter()
            .map(|(tab_id, server)| (tab_id.clone(), server.get_proxy_url()))
            .collect()
    }

    /// Stop all proxy servers
    pub async fn stop_all(&self) -> Result<()> {
        let servers: Vec<_> = {
            let servers = self.proxy_servers.read().await;
            servers.values().cloned().collect()
        };

        for server in servers {
            server.stop().await?;
        }

        {
            let mut servers = self.proxy_servers.write().await;
            servers.clear();
        }

        {
            let mut used_ports = self.used_ports.write().await;
            used_ports.clear();
        }

        info!("Stopped all proxy servers");
        Ok(())
    }
}

// ============================================================================
// WebSocket Proxy Support
// ============================================================================

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

/// WebSocket proxy handler for proxying WebSocket connections
pub struct WebSocketProxyHandler {
    upstream_proxy: Option<ProxySettings>,
}

impl WebSocketProxyHandler {
    pub fn new(upstream_proxy: Option<ProxySettings>) -> Self {
        Self { upstream_proxy }
    }

    /// Handle a WebSocket upgrade request and proxy the connection
    pub async fn handle_upgrade(&self, client_stream: TcpStream, target_url: &str) -> Result<()> {
        info!("Handling WebSocket upgrade for: {}", target_url);

        // Parse the target URL
        let url =
            url::Url::parse(target_url).map_err(|e| anyhow!("Invalid WebSocket URL: {}", e))?;

        // Connect to the target WebSocket server
        let (ws_stream, _response) = if let Some(ref proxy) = self.upstream_proxy {
            self.connect_through_proxy(&url, proxy).await?
        } else {
            self.connect_direct(&url).await?
        };

        // Accept the client WebSocket connection
        let client_ws = tokio_tungstenite::accept_async(client_stream)
            .await
            .map_err(|e| anyhow!("Failed to accept WebSocket connection: {}", e))?;

        // Proxy messages between client and target
        self.proxy_websocket(client_ws, ws_stream).await
    }

    /// Connect directly to a WebSocket server
    async fn connect_direct(
        &self,
        url: &url::Url,
    ) -> Result<(
        WebSocketStream<MaybeTlsStream<TcpStream>>,
        tokio_tungstenite::tungstenite::http::Response<Option<Vec<u8>>>,
    )> {
        tokio_tungstenite::connect_async(url.as_str())
            .await
            .map_err(|e| anyhow!("Failed to connect to WebSocket server: {}", e))
    }

    /// Establish a CONNECT tunnel through the proxy
    async fn establish_proxy_tunnel(
        &self,
        url: &url::Url,
        proxy: &ProxySettings,
    ) -> Result<TcpStream> {
        let mut proxy_stream = connect_to_proxy(proxy).await?;

        let target_host = url
            .host_str()
            .ok_or_else(|| anyhow!("No host in WebSocket URL"))?;
        let target_port = url
            .port()
            .unwrap_or(if url.scheme() == "wss" { 443 } else { 80 });

        // Send CONNECT request and verify response
        send_connect_request(&mut proxy_stream, target_host, target_port, proxy).await?;

        Ok(proxy_stream)
    }

    /// Perform TLS handshake on a stream
    async fn perform_tls_handshake(
        &self,
        stream: TcpStream,
        host: &str,
    ) -> Result<tokio_native_tls::TlsStream<TcpStream>> {
        let connector = tokio_native_tls::TlsConnector::from(
            native_tls::TlsConnector::new().map_err(|e| anyhow!("TLS error: {}", e))?,
        );
        connector
            .connect(host, stream)
            .await
            .map_err(|e| anyhow!("TLS handshake failed: {}", e))
    }

    /// Connect through a proxy to a WebSocket server
    async fn connect_through_proxy(
        &self,
        url: &url::Url,
        proxy: &ProxySettings,
    ) -> Result<(
        WebSocketStream<MaybeTlsStream<TcpStream>>,
        tokio_tungstenite::tungstenite::http::Response<Option<Vec<u8>>>,
    )> {
        let proxy_stream = self.establish_proxy_tunnel(url, proxy).await?;

        let target_host = url
            .host_str()
            .ok_or_else(|| anyhow!("No host in WebSocket URL"))?;

        // Upgrade to WebSocket (with TLS if needed)
        let ws_stream = if url.scheme() == "wss" {
            let tls_stream = self
                .perform_tls_handshake(proxy_stream, target_host)
                .await?;
            tokio_tungstenite::client_async(url.as_str(), MaybeTlsStream::NativeTls(tls_stream))
                .await
                .map_err(|e| anyhow!("WebSocket handshake failed: {}", e))?
        } else {
            tokio_tungstenite::client_async(url.as_str(), MaybeTlsStream::Plain(proxy_stream))
                .await
                .map_err(|e| anyhow!("WebSocket handshake failed: {}", e))?
        };

        Ok(ws_stream)
    }

    /// Forward WebSocket messages from reader to writer until close or error
    async fn forward_websocket_messages<R, W>(mut reader: R, mut writer: W)
    where
        R: futures_util::Stream<
                Item = Result<
                    tokio_tungstenite::tungstenite::Message,
                    tokio_tungstenite::tungstenite::Error,
                >,
            > + Unpin,
        W: futures_util::Sink<
                tokio_tungstenite::tungstenite::Message,
                Error = tokio_tungstenite::tungstenite::Error,
            > + Unpin,
    {
        while let Some(msg_result) = reader.next().await {
            match msg_result {
                Ok(msg) if msg.is_close() => break,
                Ok(msg) => {
                    if writer.send(msg).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    }

    /// Proxy WebSocket messages between client and target
    async fn proxy_websocket<S1, S2>(
        &self,
        client: WebSocketStream<S1>,
        target: WebSocketStream<S2>,
    ) -> Result<()>
    where
        S1: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
        S2: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    {
        let (client_write, client_read) = client.split();
        let (target_write, target_read) = target.split();

        let client_to_target = Self::forward_websocket_messages(client_read, target_write);
        let target_to_client = Self::forward_websocket_messages(target_read, client_write);

        // Run both forwarding tasks concurrently until one completes
        tokio::select! {
            _ = client_to_target => {}
            _ = target_to_client => {}
        }

        info!("WebSocket proxy connection closed");
        Ok(())
    }
}

// ============================================================================
// Network Interceptor
// ============================================================================

/// WebSocket interception result
#[derive(Debug, Clone)]
pub struct WebSocketInterception {
    pub url: String,
    pub message_count: usize,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Network request interceptor for monitoring and modifying requests
pub struct NetworkInterceptor {
    /// Intercepted requests log
    intercepted_requests: Arc<RwLock<Vec<InterceptedRequest>>>,
    /// WebSocket connections
    websocket_connections: Arc<RwLock<HashMap<String, WebSocketInterception>>>,
    /// Request modification rules
    modification_rules: Arc<RwLock<Vec<ModificationRule>>>,
    /// Blocked URL patterns
    blocked_patterns: Arc<RwLock<Vec<String>>>,
}

/// An intercepted HTTP request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InterceptedRequest {
    pub id: String,
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub response_status: Option<u16>,
    pub response_headers: Option<HashMap<String, String>>,
    pub blocked: bool,
    pub modified: bool,
}

/// A rule for modifying requests
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModificationRule {
    pub id: String,
    pub name: String,
    pub url_pattern: String,
    pub enabled: bool,
    pub modifications: RequestModifications,
}

/// Modifications to apply to a request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RequestModifications {
    pub add_headers: HashMap<String, String>,
    pub remove_headers: Vec<String>,
    pub modify_headers: HashMap<String, String>,
    pub redirect_url: Option<String>,
}

impl NetworkInterceptor {
    pub fn new() -> Self {
        Self {
            intercepted_requests: Arc::new(RwLock::new(Vec::new())),
            websocket_connections: Arc::new(RwLock::new(HashMap::new())),
            modification_rules: Arc::new(RwLock::new(Vec::new())),
            blocked_patterns: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Log an intercepted request
    pub async fn log_request(&self, request: InterceptedRequest) {
        let mut requests = self.intercepted_requests.write().await;
        requests.push(request);

        // Keep only the last 1000 requests
        if requests.len() > 1000 {
            requests.remove(0);
        }
    }

    /// Get all intercepted requests
    pub async fn get_intercepted_requests(&self) -> Vec<InterceptedRequest> {
        self.intercepted_requests.read().await.clone()
    }

    /// Clear intercepted requests
    pub async fn clear_requests(&self) {
        self.intercepted_requests.write().await.clear();
    }

    /// Add a modification rule
    pub async fn add_rule(&self, rule: ModificationRule) {
        self.modification_rules.write().await.push(rule);
    }

    /// Remove a modification rule
    pub async fn remove_rule(&self, rule_id: &str) {
        let mut rules = self.modification_rules.write().await;
        rules.retain(|r| r.id != rule_id);
    }

    /// Get all modification rules
    pub async fn get_rules(&self) -> Vec<ModificationRule> {
        self.modification_rules.read().await.clone()
    }

    /// Add a blocked URL pattern
    pub async fn block_pattern(&self, pattern: String) {
        self.blocked_patterns.write().await.push(pattern);
    }

    /// Check if a URL should be blocked
    pub async fn should_block(&self, url: &str) -> bool {
        let patterns = self.blocked_patterns.read().await;
        patterns.iter().any(|pattern| url.contains(pattern))
    }

    /// Apply a single modification rule to a request
    fn apply_rule_to_request(request: &mut InterceptedRequest, rule: &ModificationRule) {
        // Add headers
        for (key, value) in &rule.modifications.add_headers {
            request.headers.insert(key.clone(), value.clone());
        }

        // Remove headers
        for key in &rule.modifications.remove_headers {
            request.headers.remove(key);
        }

        // Modify existing headers
        for (key, value) in &rule.modifications.modify_headers {
            if request.headers.contains_key(key) {
                request.headers.insert(key.clone(), value.clone());
            }
        }

        request.modified = true;
    }

    /// Check if a rule matches the request
    fn rule_matches_request(request: &InterceptedRequest, rule: &ModificationRule) -> bool {
        rule.enabled && request.url.contains(&rule.url_pattern)
    }

    /// Apply modification rules to a request
    pub async fn apply_modifications(&self, mut request: InterceptedRequest) -> InterceptedRequest {
        let rules = self.modification_rules.read().await;

        for rule in rules.iter() {
            if Self::rule_matches_request(&request, rule) {
                Self::apply_rule_to_request(&mut request, rule);
            }
        }

        request
    }

    /// Register a WebSocket connection
    pub async fn register_websocket(&self, id: String, url: String) {
        let mut connections = self.websocket_connections.write().await;
        connections.insert(
            id,
            WebSocketInterception {
                url,
                message_count: 0,
                started_at: chrono::Utc::now(),
                ended_at: None,
            },
        );
    }

    /// Update WebSocket message count
    pub async fn increment_websocket_count(&self, id: &str) {
        let mut connections = self.websocket_connections.write().await;
        if let Some(conn) = connections.get_mut(id) {
            conn.message_count += 1;
        }
    }

    /// Close a WebSocket connection
    pub async fn close_websocket(&self, id: &str) {
        let mut connections = self.websocket_connections.write().await;
        if let Some(conn) = connections.get_mut(id) {
            conn.ended_at = Some(chrono::Utc::now());
        }
    }

    /// Get all WebSocket connections
    pub async fn get_websocket_connections(&self) -> HashMap<String, WebSocketInterception> {
        self.websocket_connections.read().await.clone()
    }
}

impl Default for NetworkInterceptor {
    fn default() -> Self {
        Self::new()
    }
}
