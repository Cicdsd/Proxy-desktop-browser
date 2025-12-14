use browser_core::proxy_rotation::{ProxyRotationManager, ProxyRotationStrategy, ProxyMetrics};
use browser_core::proxy::{FreeProxy, ProxyType};
use browser_core::free_ip_providers::FreeIpProviderManager;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Duration;

fn create_test_proxy(ip: &str, country: &str) -> FreeProxy {
    // Safely extract first 2 characters for country code
    let country_code = country
        .get(..2)
        .unwrap_or("XX")
        .to_uppercase();
    
    FreeProxy {
        ip: ip.to_string(),
        port: 8080,
        proxy_type: ProxyType::Http,
        country: country.to_string(),
        country_code,
        anonymity: "elite".to_string(),
        speed: 100.0,
        uptime: 99.9,
        last_checked: chrono::Utc::now(),
        provider: "test_provider".to_string(),
        is_working: true,
    }
}

#[tokio::test]
async fn test_rotation_strategies_exist() {
    // Test that all rotation strategies can be created
    let strategies = vec![
        ProxyRotationStrategy::PerRequest(10),
        ProxyRotationStrategy::PerDuration(Duration::minutes(5)),
        ProxyRotationStrategy::PerSession,
        ProxyRotationStrategy::Random { probability: 0.5 },
        ProxyRotationStrategy::Sticky { duration: Duration::hours(1) },
        ProxyRotationStrategy::Geographic { country_codes: vec!["US".to_string(), "GB".to_string()] },
        ProxyRotationStrategy::PerformanceBased,
        ProxyRotationStrategy::RoundRobin,
        ProxyRotationStrategy::DomainBased,
        ProxyRotationStrategy::Manual,
    ];
    
    assert_eq!(strategies.len(), 10);
}

#[tokio::test]
async fn test_proxy_metrics_initialization() {
    let metrics = ProxyMetrics {
        response_time_ms: 100.0,
        success_rate: 95.0,
        last_success: Some(chrono::Utc::now()),
        consecutive_failures: 0,
        total_requests: 100,
        failed_requests: 5,
    };
    
    assert_eq!(metrics.response_time_ms, 100.0);
    assert_eq!(metrics.success_rate, 95.0);
    assert_eq!(metrics.consecutive_failures, 0);
    assert_eq!(metrics.total_requests, 100);
    assert_eq!(metrics.failed_requests, 5);
}

#[tokio::test]
async fn test_proxy_metrics_success_rate_calculation() {
    let metrics = ProxyMetrics {
        response_time_ms: 150.0,
        success_rate: 0.0, // Will be calculated
        last_success: None,
        consecutive_failures: 2,
        total_requests: 50,
        failed_requests: 10,
    };
    
    // Success rate = (total - failed) / total * 100
    let calculated_rate = ((metrics.total_requests - metrics.failed_requests) as f64 
        / metrics.total_requests as f64) * 100.0;
    
    assert_eq!(calculated_rate, 80.0);
}

#[tokio::test]
async fn test_free_proxy_creation() {
    let proxy = create_test_proxy("192.168.1.1", "United States");
    
    assert_eq!(proxy.ip, "192.168.1.1");
    assert_eq!(proxy.port, 8080);
    assert_eq!(proxy.country, "United States");
    assert!(proxy.is_working);
}

#[tokio::test]
async fn test_proxy_type_variants() {
    // Test all proxy type variants
    let http = ProxyType::Http;
    let https = ProxyType::Https;
    let socks4 = ProxyType::Socks4;
    let socks5 = ProxyType::Socks5;
    let direct = ProxyType::Direct;
    
    // Ensure they are different
    assert!(matches!(http, ProxyType::Http));
    assert!(matches!(https, ProxyType::Https));
    assert!(matches!(socks4, ProxyType::Socks4));
    assert!(matches!(socks5, ProxyType::Socks5));
    assert!(matches!(direct, ProxyType::Direct));
}

#[tokio::test]
async fn test_rotation_strategy_per_request() {
    let strategy = ProxyRotationStrategy::PerRequest(5);
    
    match strategy {
        ProxyRotationStrategy::PerRequest(count) => {
            assert_eq!(count, 5);
        }
        _ => panic!("Expected PerRequest strategy"),
    }
}

#[tokio::test]
async fn test_rotation_strategy_per_duration() {
    let duration = Duration::minutes(10);
    let strategy = ProxyRotationStrategy::PerDuration(duration);
    
    match strategy {
        ProxyRotationStrategy::PerDuration(d) => {
            assert_eq!(d.num_minutes(), 10);
        }
        _ => panic!("Expected PerDuration strategy"),
    }
}

#[tokio::test]
async fn test_rotation_strategy_geographic() {
    let countries = vec!["US".to_string(), "GB".to_string(), "DE".to_string()];
    let strategy = ProxyRotationStrategy::Geographic { 
        country_codes: countries.clone() 
    };
    
    match strategy {
        ProxyRotationStrategy::Geographic { country_codes } => {
            assert_eq!(country_codes.len(), 3);
            assert!(country_codes.contains(&"US".to_string()));
            assert!(country_codes.contains(&"GB".to_string()));
            assert!(country_codes.contains(&"DE".to_string()));
        }
        _ => panic!("Expected Geographic strategy"),
    }
}

#[tokio::test]
async fn test_rotation_strategy_random_probability() {
    let strategy = ProxyRotationStrategy::Random { probability: 0.3 };
    
    match strategy {
        ProxyRotationStrategy::Random { probability } => {
            assert_eq!(probability, 0.3);
            assert!(probability >= 0.0 && probability <= 1.0);
        }
        _ => panic!("Expected Random strategy"),
    }
}

#[tokio::test]
async fn test_proxy_metrics_serialization() {
    let metrics = ProxyMetrics {
        response_time_ms: 200.0,
        success_rate: 88.5,
        last_success: Some(chrono::Utc::now()),
        consecutive_failures: 1,
        total_requests: 200,
        failed_requests: 23,
    };
    
    // Test JSON serialization
    let json = serde_json::to_string(&metrics).expect("Failed to serialize");
    assert!(json.contains("response_time_ms"));
    assert!(json.contains("success_rate"));
    
    // Test deserialization
    let deserialized: ProxyMetrics = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized.response_time_ms, 200.0);
    assert_eq!(deserialized.success_rate, 88.5);
}
