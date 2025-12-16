use browser_core::{
    TlsManager, TlsConfig, TlsVersion, CertificateInfo, CertificateStatus,
    AuditManager, AuditEventType, AuditSeverity,
};
use chrono::Utc;

/// Test TLS manager creation with defaults
#[tokio::test]
async fn test_tls_manager_defaults() {
    let manager = TlsManager::new();
    let config = manager.get_config().await;
    
    assert_eq!(config.min_version, TlsVersion::Tls13);
    assert!(config.require_valid_certs);
    assert!(config.enable_hsts);
    assert!(config.enable_certificate_transparency);
}

/// Test certificate pinning
#[tokio::test]
async fn test_certificate_pinning() {
    let manager = TlsManager::new();
    
    // Add certificate pin
    manager.add_certificate_pin(
        "secure.example.com",
        vec!["sha256/abc123".to_string(), "sha256/def456".to_string()],
        true,
        Some(365),
    ).await;
    
    // Verify pin matches
    let result = manager.verify_certificate_pin("secure.example.com", "sha256/abc123").await;
    assert!(result.is_ok());
    assert!(result.unwrap());
    
    // Verify wrong pin fails
    let result = manager.verify_certificate_pin("secure.example.com", "sha256/wrong").await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
    
    // Verify subdomain matches (include_subdomains = true)
    let result = manager.verify_certificate_pin("api.secure.example.com", "sha256/def456").await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

/// Test certificate pin removal
#[tokio::test]
async fn test_certificate_pin_removal() {
    let manager = TlsManager::new();
    
    manager.add_certificate_pin("test.com", vec!["pin1".to_string()], false, None).await;
    
    let pins = manager.get_certificate_pins().await;
    assert_eq!(pins.len(), 1);
    
    manager.remove_certificate_pin("test.com").await;
    
    let pins = manager.get_certificate_pins().await;
    assert_eq!(pins.len(), 0);
}

/// Test HSTS management
#[tokio::test]
async fn test_hsts_entries() {
    let manager = TlsManager::new();
    
    // Add HSTS entry
    manager.add_hsts_entry("hsts.example.com", 31536000, true).await;
    
    // Check HSTS is required
    assert!(manager.is_hsts_required("hsts.example.com").await);
    assert!(manager.is_hsts_required("sub.hsts.example.com").await);
    assert!(!manager.is_hsts_required("other.com").await);
}

/// Test HSTS without subdomains
#[tokio::test]
async fn test_hsts_no_subdomains() {
    let manager = TlsManager::new();
    
    // Add HSTS entry without subdomains
    manager.add_hsts_entry("exact.com", 3600, false).await;
    
    assert!(manager.is_hsts_required("exact.com").await);
    assert!(!manager.is_hsts_required("sub.exact.com").await);
}

/// Test security headers generation
#[tokio::test]
async fn test_security_headers() {
    let manager = TlsManager::new();
    let headers = manager.get_security_headers().await;
    
    // Verify required headers are present
    let header_names: Vec<&str> = headers.iter().map(|(k, _)| k.as_str()).collect();
    
    assert!(header_names.contains(&"Strict-Transport-Security"));
    assert!(header_names.contains(&"X-Content-Type-Options"));
    assert!(header_names.contains(&"X-Frame-Options"));
    assert!(header_names.contains(&"X-XSS-Protection"));
    assert!(header_names.contains(&"Referrer-Policy"));
}

/// Test certificate validation - valid certificate
#[tokio::test]
async fn test_certificate_validation_valid() {
    let manager = TlsManager::new();
    
    let cert = CertificateInfo {
        subject: "CN=example.com".to_string(),
        issuer: "CN=Test CA".to_string(),
        serial_number: "123456".to_string(),
        not_before: Utc::now() - chrono::Duration::days(30),
        not_after: Utc::now() + chrono::Duration::days(335),
        fingerprint_sha256: "abc123".to_string(),
        status: CertificateStatus::Valid,
        is_pinned: false,
    };
    
    let result = manager.validate_certificate("example.com", &cert).await;
    assert!(result.is_ok());
}

/// Test certificate validation - expired
#[tokio::test]
async fn test_certificate_validation_expired() {
    let manager = TlsManager::new();
    
    let cert = CertificateInfo {
        subject: "CN=expired.com".to_string(),
        issuer: "CN=Test CA".to_string(),
        serial_number: "123456".to_string(),
        not_before: Utc::now() - chrono::Duration::days(400),
        not_after: Utc::now() - chrono::Duration::days(35),
        fingerprint_sha256: "abc123".to_string(),
        status: CertificateStatus::Expired,
        is_pinned: false,
    };
    
    let result = manager.validate_certificate("expired.com", &cert).await;
    assert!(result.is_err());
}

/// Test TLS config update
#[tokio::test]
async fn test_tls_config_update() {
    let manager = TlsManager::new();
    
    let new_config = TlsConfig {
        min_version: TlsVersion::Tls12,
        require_valid_certs: false,
        enable_certificate_transparency: false,
        enable_ocsp_stapling: true,
        enable_hsts: false,
        hsts_max_age_seconds: 0,
        allowed_cipher_suites: vec!["TLS_AES_256_GCM_SHA384".to_string()],
    };
    
    manager.set_config(new_config).await;
    
    let config = manager.get_config().await;
    assert_eq!(config.min_version, TlsVersion::Tls12);
    assert!(!config.require_valid_certs);
    assert!(!config.enable_hsts);
}

/// Test audit manager event logging
#[tokio::test]
async fn test_audit_event_logging() {
    let manager = AuditManager::new();
    
    manager.log(
        AuditEventType::Login,
        AuditSeverity::Info,
        "auth_service",
        "User successfully logged in",
        true,
    ).await;
    
    manager.log(
        AuditEventType::LoginFailed,
        AuditSeverity::Warning,
        "auth_service",
        "Invalid password attempt",
        false,
    ).await;
    
    let events = manager.get_recent_events(10).await;
    assert_eq!(events.len(), 2);
}

/// Test audit event filtering by type
#[tokio::test]
async fn test_audit_filter_by_type() {
    let manager = AuditManager::new();
    
    manager.log(AuditEventType::Login, AuditSeverity::Info, "auth", "Login 1", true).await;
    manager.log(AuditEventType::Login, AuditSeverity::Info, "auth", "Login 2", true).await;
    manager.log(AuditEventType::ProxyConnected, AuditSeverity::Info, "proxy", "Connected", true).await;
    manager.log(AuditEventType::NavigationCompleted, AuditSeverity::Info, "nav", "Page loaded", true).await;
    
    let login_events = manager.get_events_by_type(AuditEventType::Login).await;
    assert_eq!(login_events.len(), 2);
    
    let proxy_events = manager.get_events_by_type(AuditEventType::ProxyConnected).await;
    assert_eq!(proxy_events.len(), 1);
}

/// Test audit event filtering by severity
#[tokio::test]
async fn test_audit_filter_by_severity() {
    let manager = AuditManager::new();
    
    manager.log(AuditEventType::Login, AuditSeverity::Info, "auth", "Info event", true).await;
    manager.log(AuditEventType::LoginFailed, AuditSeverity::Warning, "auth", "Warning event", false).await;
    manager.log(AuditEventType::SecuritySettingChanged, AuditSeverity::Error, "sec", "Error event", false).await;
    manager.log(AuditEventType::TlsConnectionFailed, AuditSeverity::Critical, "tls", "Critical event", false).await;
    
    let warnings_and_above = manager.get_events_by_severity(AuditSeverity::Warning).await;
    assert_eq!(warnings_and_above.len(), 3);
    
    let errors_and_above = manager.get_events_by_severity(AuditSeverity::Error).await;
    assert_eq!(errors_and_above.len(), 2);
}

/// Test audit failed events
#[tokio::test]
async fn test_audit_failed_events() {
    let manager = AuditManager::new();
    
    manager.log(AuditEventType::Login, AuditSeverity::Info, "auth", "Success", true).await;
    manager.log(AuditEventType::LoginFailed, AuditSeverity::Warning, "auth", "Failed 1", false).await;
    manager.log(AuditEventType::ProxyConnected, AuditSeverity::Info, "proxy", "Success", true).await;
    manager.log(AuditEventType::ProxyValidationFailed, AuditSeverity::Error, "proxy", "Failed 2", false).await;
    
    let failed = manager.get_failed_events().await;
    assert_eq!(failed.len(), 2);
}

/// Test audit summary
#[tokio::test]
async fn test_audit_summary() {
    let manager = AuditManager::new();
    
    for i in 0..20 {
        let success = i % 3 != 0;
        let severity = if i % 5 == 0 { AuditSeverity::Critical } else { AuditSeverity::Info };
        manager.log(AuditEventType::NavigationCompleted, severity, "test", &format!("Event {}", i), success).await;
    }
    
    let summary = manager.get_summary().await;
    assert_eq!(summary.total_events, 20);
    assert!(summary.failed_events > 0);
    assert!(summary.critical_events > 0);
}

/// Test audit security events
#[tokio::test]
async fn test_audit_security_events() {
    let manager = AuditManager::new();
    
    manager.log(AuditEventType::CertificateValidation, AuditSeverity::Info, "tls", "Cert valid", true).await;
    manager.log(AuditEventType::SecuritySettingChanged, AuditSeverity::Warning, "sec", "Setting changed", true).await;
    manager.log(AuditEventType::NavigationCompleted, AuditSeverity::Info, "nav", "Not security", true).await;
    
    let security_events = manager.get_security_events().await;
    assert_eq!(security_events.len(), 2);
}

/// Test audit JSON export
#[tokio::test]
async fn test_audit_json_export() {
    let manager = AuditManager::new();
    
    manager.log(AuditEventType::Login, AuditSeverity::Info, "auth", "Test event", true).await;
    
    let json = manager.export_to_json().await;
    assert!(json.is_ok());
    let json_str = json.unwrap();
    assert!(json_str.contains("Login"));
    assert!(json_str.contains("Test event"));
}

/// Test audit event count by type
#[tokio::test]
async fn test_audit_count_by_type() {
    let manager = AuditManager::new();
    
    manager.log(AuditEventType::Login, AuditSeverity::Info, "auth", "1", true).await;
    manager.log(AuditEventType::Login, AuditSeverity::Info, "auth", "2", true).await;
    manager.log(AuditEventType::Login, AuditSeverity::Info, "auth", "3", true).await;
    manager.log(AuditEventType::Logout, AuditSeverity::Info, "auth", "1", true).await;
    
    let counts = manager.get_count_by_type().await;
    assert_eq!(counts.get("Login"), Some(&3));
    assert_eq!(counts.get("Logout"), Some(&1));
}

/// Test audit clear events
#[tokio::test]
async fn test_audit_clear_events() {
    let manager = AuditManager::new();
    
    manager.log(AuditEventType::Login, AuditSeverity::Info, "auth", "Event", true).await;
    assert_eq!(manager.get_event_count().await, 1);
    
    manager.clear_events().await;
    assert_eq!(manager.get_event_count().await, 0);
}
