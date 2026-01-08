//! Tests for the request module

use browser_core::request::{
    RequestBuilder, RequestManager, RequestConfig, RequestResponse,
    RequestError, RequestErrorKind, HttpMethod, RequestBody,
};
use std::collections::HashMap;
use std::time::Duration;

#[test]
fn test_request_builder_get() {
    let builder = RequestBuilder::get("https://example.com");
    assert_eq!(builder.url, "https://example.com");
}

#[test]
fn test_request_builder_post() {
    let builder = RequestBuilder::post("https://api.example.com/data");
    assert_eq!(builder.url, "https://api.example.com/data");
}

#[test]
fn test_request_builder_with_headers() {
    let builder = RequestBuilder::get("https://example.com")
        .header("Authorization", "Bearer token123")
        .header("Accept", "application/json");
    
    assert_eq!(builder.headers.get("Authorization").unwrap(), "Bearer token123");
    assert_eq!(builder.headers.get("Accept").unwrap(), "application/json");
}

#[test]
fn test_request_builder_with_timeout() {
    let builder = RequestBuilder::get("https://example.com")
        .timeout(Duration::from_secs(60));
    
    assert_eq!(builder.config.timeout, Duration::from_secs(60));
}

#[test]
fn test_request_builder_follow_redirects() {
    let builder = RequestBuilder::get("https://example.com")
        .follow_redirects(false);
    
    assert!(!builder.config.follow_redirects);
}

#[test]
fn test_request_config_default() {
    let config = RequestConfig::default();
    
    assert_eq!(config.timeout, Duration::from_secs(30));
    assert!(config.follow_redirects);
    assert_eq!(config.max_redirects, 10);
    assert!(config.verify_ssl);
    assert!(config.user_agent.is_some());
}

#[test]
fn test_request_response_is_success() {
    let success_response = RequestResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers: HashMap::new(),
        body: r#"{"success": true}"#.to_string(),
        response_time_ms: 150,
        final_url: "https://example.com".to_string(),
    };
    assert!(success_response.is_success());

    let created_response = RequestResponse {
        status: 201,
        status_text: "Created".to_string(),
        headers: HashMap::new(),
        body: "{}".to_string(),
        response_time_ms: 200,
        final_url: "https://example.com/resource".to_string(),
    };
    assert!(created_response.is_success());

    let error_response = RequestResponse {
        status: 404,
        status_text: "Not Found".to_string(),
        headers: HashMap::new(),
        body: "{}".to_string(),
        response_time_ms: 100,
        final_url: "https://example.com/missing".to_string(),
    };
    assert!(!error_response.is_success());

    let server_error = RequestResponse {
        status: 500,
        status_text: "Internal Server Error".to_string(),
        headers: HashMap::new(),
        body: "{}".to_string(),
        response_time_ms: 50,
        final_url: "https://example.com/error".to_string(),
    };
    assert!(!server_error.is_success());
}

#[test]
fn test_request_response_json_parsing() {
    let response = RequestResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers: HashMap::new(),
        body: r#"{"name": "test", "value": 42}"#.to_string(),
        response_time_ms: 100,
        final_url: "https://example.com".to_string(),
    };

    #[derive(serde::Deserialize)]
    struct TestData {
        name: String,
        value: i32,
    }

    let data: TestData = response.json().expect("Failed to parse JSON");
    assert_eq!(data.name, "test");
    assert_eq!(data.value, 42);
}

#[test]
fn test_http_method_conversion() {
    use reqwest::Method;
    
    assert_eq!(Method::from(HttpMethod::Get), Method::GET);
    assert_eq!(Method::from(HttpMethod::Post), Method::POST);
    assert_eq!(Method::from(HttpMethod::Put), Method::PUT);
    assert_eq!(Method::from(HttpMethod::Delete), Method::DELETE);
    assert_eq!(Method::from(HttpMethod::Patch), Method::PATCH);
    assert_eq!(Method::from(HttpMethod::Head), Method::HEAD);
    assert_eq!(Method::from(HttpMethod::Options), Method::OPTIONS);
}

#[test]
fn test_request_error_creation() {
    let error = RequestError::new(RequestErrorKind::Timeout, "Request timed out")
        .with_url("https://example.com")
        .with_status(408);
    
    assert!(matches!(error.kind, RequestErrorKind::Timeout));
    assert_eq!(error.message, "Request timed out");
    assert_eq!(error.url, Some("https://example.com".to_string()));
    assert_eq!(error.status_code, Some(408));
}

#[test]
fn test_request_error_display() {
    let error = RequestError::new(RequestErrorKind::Network, "Connection refused");
    let display = format!("{}", error);
    assert!(display.contains("Network"));
    assert!(display.contains("Connection refused"));
}

#[test]
fn test_request_manager_creation() {
    let manager = RequestManager::new();
    assert!(manager.is_ok());
}

#[test]
fn test_request_builder_form_data() {
    let mut form_data = HashMap::new();
    form_data.insert("username".to_string(), "test_user".to_string());
    form_data.insert("password".to_string(), "secret".to_string());
    
    let builder = RequestBuilder::post("https://example.com/login")
        .form(form_data);
    
    assert!(matches!(builder.body, RequestBody::Form(_)));
    assert_eq!(
        builder.headers.get("Content-Type").unwrap(),
        "application/x-www-form-urlencoded"
    );
}

#[test]
fn test_request_builder_text_body() {
    let builder = RequestBuilder::post("https://example.com/text")
        .body_text("Hello, World!");
    
    assert!(matches!(builder.body, RequestBody::Text(_)));
    assert_eq!(builder.headers.get("Content-Type").unwrap(), "text/plain");
}

#[test]
fn test_request_builder_bytes_body() {
    let bytes = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]; // "Hello" in bytes
    let builder = RequestBuilder::post("https://example.com/binary")
        .body_bytes(bytes);
    
    assert!(matches!(builder.body, RequestBody::Bytes(_)));
}

#[test]
fn test_all_http_methods() {
    let get = RequestBuilder::get("https://example.com");
    assert_eq!(get.method, HttpMethod::Get);
    
    let post = RequestBuilder::post("https://example.com");
    assert_eq!(post.method, HttpMethod::Post);
    
    let put = RequestBuilder::put("https://example.com");
    assert_eq!(put.method, HttpMethod::Put);
    
    let delete = RequestBuilder::delete("https://example.com");
    assert_eq!(delete.method, HttpMethod::Delete);
    
    let patch = RequestBuilder::patch("https://example.com");
    assert_eq!(patch.method, HttpMethod::Patch);
    
    let head = RequestBuilder::head("https://example.com");
    assert_eq!(head.method, HttpMethod::Head);
    
    let options = RequestBuilder::options("https://example.com");
    assert_eq!(options.method, HttpMethod::Options);
}
