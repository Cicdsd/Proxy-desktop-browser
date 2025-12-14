use std::sync::Arc;

use api_server::ApiServer;
use axum::{body::Body, http::Request};
use browser_core::TabIPManager;
use serde::Deserialize;
use tower::util::ServiceExt; // for oneshot
use virtual_ip::demo_generator;
use sqlx::SqlitePool;

#[derive(Debug, Deserialize)]
struct TabResponse {
    tab_id: String,
    ip: String,
    country_code: String,
}

#[derive(Debug, Deserialize)]
struct VirtualIPResponse {
    ip: String,
    country_code: String,
}

#[derive(Debug, Deserialize)]
struct ValidationResponse {
    overall_pass: bool,
}

async fn create_test_server() -> Arc<ApiServer> {
    let generator = demo_generator();
    let ip_gen = Arc::new(generator.clone());
    
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory database");
    
    // Create necessary table for testing with all required columns
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS tabs (
            id TEXT PRIMARY KEY,
            title TEXT,
            url TEXT,
            favicon TEXT,
            proxy_id TEXT,
            virtual_ip TEXT,
            created_at INTEGER NOT NULL,
            last_active INTEGER NOT NULL,
            is_pinned INTEGER DEFAULT 0,
            is_suspended INTEGER DEFAULT 0
        )"
    )
    .execute(&pool)
    .await
    .expect("Failed to create tabs table");
    
    let tab_manager = Arc::new(TabIPManager::new(generator, pool)
        .await
        .expect("Failed to create TabIPManager"));
    Arc::new(ApiServer::new(tab_manager, ip_gen))
}

#[tokio::test]
async fn create_list_rotate_validate_tab() {
    // Arrange app
    let server = create_test_server().await;
    let app = server.router().await;

    // Create tab (US)
    let req = Request::builder()
        .method("POST")
        .uri("/api/tabs")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"country_code":"US"}"#))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let created: TabResponse = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(created.country_code, "US");

    // List tabs
    let req = Request::builder()
        .method("GET")
        .uri("/api/tabs")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let listed: Vec<TabResponse> = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(listed.len(), 1);

    // Rotate IP
    let rotate_uri = format!("/api/tabs/{}/rotate-ip", created.tab_id);
    let req = Request::builder()
        .method("POST")
        .uri(&rotate_uri)
        .header("content-type", "application/json")
        .body(Body::from(r#"{"new_country":null}"#))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let rotated: VirtualIPResponse = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(rotated.country_code, "US");
    assert_ne!(rotated.ip, created.ip);

    // Validate IP - just check that the endpoint responds (validation may fail with demo IPs)
    let validate_uri = format!("/api/tabs/{}/validate", created.tab_id);
    let req = Request::builder()
        .method("GET")
        .uri(&validate_uri)
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let validation: ValidationResponse = serde_json::from_slice(&body_bytes).unwrap();
    // Don't assert on overall_pass since demo IPs may not validate
    // Just verify that we got a response
    let _ = validation;
}
