//! Tests for Network Request Interception
//! 
//! This module tests:
//! - Request interception and logging
//! - WebSocket proxy handling
//! - Modification rules
//! - URL blocking

use browser_core::local_proxy::{
    NetworkInterceptor, InterceptedRequest, ModificationRule, RequestModifications,
    WebSocketInterception,
};
use std::collections::HashMap;
use chrono::Utc;

// ============================================================================
// Test Helper Functions
// ============================================================================

/// Create a basic test request with default values
fn create_test_request(id: &str, url: &str) -> InterceptedRequest {
    create_test_request_with_headers(id, url, HashMap::new())
}

/// Create a test request with custom headers
fn create_test_request_with_headers(id: &str, url: &str, headers: HashMap<String, String>) -> InterceptedRequest {
    InterceptedRequest {
        id: id.to_string(),
        method: "GET".to_string(),
        url: url.to_string(),
        headers,
        body: None,
        timestamp: Utc::now(),
        response_status: None,
        response_headers: None,
        blocked: false,
        modified: false,
    }
}

/// Create a test request with response status
fn create_test_request_with_status(id: &str, url: &str, status: u16) -> InterceptedRequest {
    InterceptedRequest {
        id: id.to_string(),
        method: "GET".to_string(),
        url: url.to_string(),
        headers: HashMap::new(),
        body: None,
        timestamp: Utc::now(),
        response_status: Some(status),
        response_headers: None,
        blocked: false,
        modified: false,
    }
}

/// Create a modification rule that adds headers
fn create_add_header_rule(id: &str, name: &str, pattern: &str, headers: HashMap<String, String>) -> ModificationRule {
    ModificationRule {
        id: id.to_string(),
        name: name.to_string(),
        url_pattern: pattern.to_string(),
        enabled: true,
        modifications: RequestModifications {
            add_headers: headers,
            remove_headers: vec![],
            modify_headers: HashMap::new(),
            redirect_url: None,
        },
    }
}

/// Create a modification rule that removes headers
fn create_remove_header_rule(id: &str, name: &str, pattern: &str, headers_to_remove: Vec<String>) -> ModificationRule {
    ModificationRule {
        id: id.to_string(),
        name: name.to_string(),
        url_pattern: pattern.to_string(),
        enabled: true,
        modifications: RequestModifications {
            add_headers: HashMap::new(),
            remove_headers: headers_to_remove,
            modify_headers: HashMap::new(),
            redirect_url: None,
        },
    }
}

// ============================================================================
// NetworkInterceptor Tests
// ============================================================================

#[tokio::test]
async fn test_network_interceptor_creation() {
    let interceptor = NetworkInterceptor::new();
    
    let requests = interceptor.get_intercepted_requests().await;
    assert!(requests.is_empty());
    
    let rules = interceptor.get_rules().await;
    assert!(rules.is_empty());
}

#[tokio::test]
async fn test_network_interceptor_log_request() {
    let interceptor = NetworkInterceptor::new();
    
    let request = create_test_request_with_status("req-1", "https://example.com/api/data", 200);
    
    interceptor.log_request(request).await;
    
    let requests = interceptor.get_intercepted_requests().await;
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].url, "https://example.com/api/data");
}

#[tokio::test]
async fn test_network_interceptor_multiple_requests() {
    let interceptor = NetworkInterceptor::new();
    
    for i in 0..5 {
        let request = create_test_request_with_status(
            &format!("req-{}", i),
            &format!("https://example.com/page{}", i),
            200,
        );
        interceptor.log_request(request).await;
    }
    
    let requests = interceptor.get_intercepted_requests().await;
    assert_eq!(requests.len(), 5);
}

#[tokio::test]
async fn test_network_interceptor_clear_requests() {
    let interceptor = NetworkInterceptor::new();
    
    let request = create_test_request("req-1", "https://example.com");
    
    interceptor.log_request(request).await;
    assert_eq!(interceptor.get_intercepted_requests().await.len(), 1);
    
    interceptor.clear_requests().await;
    assert!(interceptor.get_intercepted_requests().await.is_empty());
}
// ============================================================================
// Modification Rules Tests
// ============================================================================

#[tokio::test]
async fn test_add_modification_rule() {
    let interceptor = NetworkInterceptor::new();
    
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer token123".to_string());
    let rule = create_add_header_rule("rule-1", "Add Auth Header", "api.example.com", headers);
    
    interceptor.add_rule(rule).await;
    
    let rules = interceptor.get_rules().await;
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].name, "Add Auth Header");
}

#[tokio::test]
async fn test_remove_modification_rule() {
    let interceptor = NetworkInterceptor::new();
    
    let rule1 = create_add_header_rule("rule-1", "Rule 1", "*.example.com", HashMap::new());
    let rule2 = create_add_header_rule("rule-2", "Rule 2", "*.test.com", HashMap::new());
    
    interceptor.add_rule(rule1).await;
    interceptor.add_rule(rule2).await;
    
    assert_eq!(interceptor.get_rules().await.len(), 2);
    
    interceptor.remove_rule("rule-1").await;
    
    let rules = interceptor.get_rules().await;
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].id, "rule-2");
}

#[tokio::test]
async fn test_apply_modifications_add_headers() {
    let interceptor = NetworkInterceptor::new();
    
    let mut headers = HashMap::new();
    headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());
    let rule = create_add_header_rule("rule-1", "Add Custom Header", "api.example.com", headers);
    
    interceptor.add_rule(rule).await;
    
    let request = create_test_request("req-1", "https://api.example.com/users");
    
    let modified = interceptor.apply_modifications(request).await;
    
    assert!(modified.modified);
    assert_eq!(
        modified.headers.get("X-Custom-Header"),
        Some(&"custom-value".to_string())
    );
}

#[tokio::test]
async fn test_apply_modifications_remove_headers() {
    let interceptor = NetworkInterceptor::new();
    
    let rule = create_remove_header_rule(
        "rule-1",
        "Remove Tracking",
        "example.com",
        vec!["X-Tracking-Id".to_string()],
    );
    
    interceptor.add_rule(rule).await;
    
    let mut headers = HashMap::new();
    headers.insert("X-Tracking-Id".to_string(), "track-123".to_string());
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    
    let request = create_test_request_with_headers("req-1", "https://example.com/page", headers);
    
    let modified = interceptor.apply_modifications(request).await;
    
    assert!(modified.modified);
    assert!(modified.headers.get("X-Tracking-Id").is_none());
    assert!(modified.headers.get("Content-Type").is_some());
}

#[tokio::test]
async fn test_disabled_rule_not_applied() {
    let interceptor = NetworkInterceptor::new();
    
    let mut rule = create_add_header_rule(
        "rule-1",
        "Disabled Rule",
        "example.com",
        {
            let mut h = HashMap::new();
            h.insert("X-Should-Not-Exist".to_string(), "value".to_string());
            h
        },
    );
    rule.enabled = false; // Disable the rule
    
    interceptor.add_rule(rule).await;
    
    let request = create_test_request("req-1", "https://example.com/page");
    
    let modified = interceptor.apply_modifications(request).await;
    
    assert!(!modified.modified);
    assert!(modified.headers.get("X-Should-Not-Exist").is_none());
}

// ============================================================================
// URL Blocking Tests
// ============================================================================

#[tokio::test]
async fn test_block_pattern() {
    let interceptor = NetworkInterceptor::new();
    
    interceptor.block_pattern("ads.example.com".to_string()).await;
    interceptor.block_pattern("tracking.".to_string()).await;
    
    assert!(interceptor.should_block("https://ads.example.com/banner").await);
    assert!(interceptor.should_block("https://tracking.analytics.com/pixel").await);
    assert!(!interceptor.should_block("https://example.com/page").await);
}

#[tokio::test]
async fn test_multiple_block_patterns() {
    let interceptor = NetworkInterceptor::new();
    
    let patterns = vec![
        "ads.",
        "tracking.",
        "analytics.",
        ".doubleclick.",
        "facebook.com/tr",
    ];
    
    for pattern in patterns {
        interceptor.block_pattern(pattern.to_string()).await;
    }
    
    // Should be blocked
    assert!(interceptor.should_block("https://ads.example.com/ad.js").await);
    assert!(interceptor.should_block("https://www.doubleclick.net/pixel").await);
    
    // Should not be blocked
    assert!(!interceptor.should_block("https://example.com/page").await);
    assert!(!interceptor.should_block("https://safe-site.com/content").await);
}

// ============================================================================
// WebSocket Interception Tests
// ============================================================================

#[tokio::test]
async fn test_register_websocket() {
    let interceptor = NetworkInterceptor::new();
    
    interceptor.register_websocket(
        "ws-1".to_string(),
        "wss://example.com/socket".to_string(),
    ).await;
    
    let connections = interceptor.get_websocket_connections().await;
    assert_eq!(connections.len(), 1);
    
    let conn = connections.get("ws-1").unwrap();
    assert_eq!(conn.url, "wss://example.com/socket");
    assert_eq!(conn.message_count, 0);
    assert!(conn.ended_at.is_none());
}

#[tokio::test]
async fn test_increment_websocket_count() {
    let interceptor = NetworkInterceptor::new();
    
    interceptor.register_websocket(
        "ws-1".to_string(),
        "wss://example.com/socket".to_string(),
    ).await;
    
    for _ in 0..5 {
        interceptor.increment_websocket_count("ws-1").await;
    }
    
    let connections = interceptor.get_websocket_connections().await;
    let conn = connections.get("ws-1").unwrap();
    assert_eq!(conn.message_count, 5);
}

#[tokio::test]
async fn test_close_websocket() {
    let interceptor = NetworkInterceptor::new();
    
    interceptor.register_websocket(
        "ws-1".to_string(),
        "wss://example.com/socket".to_string(),
    ).await;
    
    interceptor.close_websocket("ws-1").await;
    
    let connections = interceptor.get_websocket_connections().await;
    let conn = connections.get("ws-1").unwrap();
    assert!(conn.ended_at.is_some());
}

#[tokio::test]
async fn test_multiple_websocket_connections() {
    let interceptor = NetworkInterceptor::new();
    
    for i in 0..3 {
        interceptor.register_websocket(
            format!("ws-{}", i),
            format!("wss://example.com/socket{}", i),
        ).await;
    }
    
    let connections = interceptor.get_websocket_connections().await;
    assert_eq!(connections.len(), 3);
}

// ============================================================================
// InterceptedRequest Tests
// ============================================================================

#[test]
fn test_intercepted_request_creation() {
    let request = InterceptedRequest {
        id: "req-123".to_string(),
        method: "POST".to_string(),
        url: "https://api.example.com/data".to_string(),
        headers: {
            let mut h = HashMap::new();
            h.insert("Content-Type".to_string(), "application/json".to_string());
            h
        },
        body: Some(b"{\"key\":\"value\"}".to_vec()),
        timestamp: Utc::now(),
        response_status: Some(201),
        response_headers: Some({
            let mut h = HashMap::new();
            h.insert("Location".to_string(), "/data/123".to_string());
            h
        }),
        blocked: false,
        modified: false,
    };
    
    assert_eq!(request.method, "POST");
    assert_eq!(request.response_status, Some(201));
    assert!(request.body.is_some());
}

#[test]
fn test_intercepted_request_serialization() {
    let request = InterceptedRequest {
        id: "req-1".to_string(),
        method: "GET".to_string(),
        url: "https://example.com".to_string(),
        headers: HashMap::new(),
        body: None,
        timestamp: Utc::now(),
        response_status: Some(200),
        response_headers: None,
        blocked: false,
        modified: true,
    };
    
    let json = serde_json::to_string(&request).unwrap();
    let parsed: InterceptedRequest = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.id, request.id);
    assert_eq!(parsed.method, request.method);
    assert_eq!(parsed.modified, request.modified);
}

// ============================================================================
// ModificationRule Tests
// ============================================================================

#[test]
fn test_modification_rule_serialization() {
    let rule = ModificationRule {
        id: "rule-1".to_string(),
        name: "Test Rule".to_string(),
        url_pattern: "*.example.com".to_string(),
        enabled: true,
        modifications: RequestModifications {
            add_headers: {
                let mut h = HashMap::new();
                h.insert("X-Test".to_string(), "value".to_string());
                h
            },
            remove_headers: vec!["Cookie".to_string()],
            modify_headers: HashMap::new(),
            redirect_url: Some("https://redirect.example.com".to_string()),
        },
    };
    
    let json = serde_json::to_string(&rule).unwrap();
    let parsed: ModificationRule = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.id, rule.id);
    assert_eq!(parsed.url_pattern, rule.url_pattern);
    assert!(parsed.modifications.redirect_url.is_some());
}

// ============================================================================
// WebSocketInterception Tests
// ============================================================================

#[test]
fn test_websocket_interception_structure() {
    let interception = WebSocketInterception {
        url: "wss://example.com/ws".to_string(),
        message_count: 42,
        started_at: Utc::now(),
        ended_at: Some(Utc::now()),
    };
    
    assert_eq!(interception.url, "wss://example.com/ws");
    assert_eq!(interception.message_count, 42);
    assert!(interception.ended_at.is_some());
}
