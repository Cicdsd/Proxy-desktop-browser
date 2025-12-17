use browser_core::{
    MemoryManager, MemoryPressure, ResourceType,
    ErrorRecoveryManager, ErrorCategory, ErrorSeverity,
    PerformanceManager,
    LogManager, LogLevel, LogConfig,
};
use std::time::Duration;
use anyhow;

/// Test memory manager allocation tracking
#[tokio::test]
async fn test_memory_manager_allocations() {
    let manager = MemoryManager::new(100); // 100MB limit
    
    // Register multiple allocations
    manager.register_allocation("tab1", "Browser Tab 1", ResourceType::TabMemory, 10 * 1024 * 1024).await;
    manager.register_allocation("cache1", "Image Cache", ResourceType::CacheEntry, 5 * 1024 * 1024).await;
    manager.register_allocation("script1", "JS Context", ResourceType::ScriptContext, 2 * 1024 * 1024).await;
    
    // Verify total usage
    let usage = manager.get_current_usage();
    assert_eq!(usage, 17 * 1024 * 1024);
    
    // Verify stats
    let stats = manager.get_stats().await;
    assert_eq!(stats.allocation_count, 3);
    
    // Unregister one
    manager.unregister_allocation("cache1").await;
    assert_eq!(manager.get_current_usage(), 12 * 1024 * 1024);
}

/// Test memory pressure levels
#[tokio::test]
async fn test_memory_pressure_levels() {
    let manager = MemoryManager::new(10); // 10MB limit
    
    // Low pressure
    assert_eq!(manager.get_memory_pressure(), MemoryPressure::Low);
    
    // Add 5MB (50%) - Medium
    manager.register_allocation("test1", "Test", ResourceType::Other, 5 * 1024 * 1024).await;
    assert_eq!(manager.get_memory_pressure(), MemoryPressure::Medium);
    
    // Add 3MB more (80%) - High  
    manager.register_allocation("test2", "Test", ResourceType::Other, 3 * 1024 * 1024).await;
    assert_eq!(manager.get_memory_pressure(), MemoryPressure::High);
    
    // Add 2MB more (100%) - Critical
    manager.register_allocation("test3", "Test", ResourceType::Other, 2 * 1024 * 1024).await;
    assert_eq!(manager.get_memory_pressure(), MemoryPressure::Critical);
}

/// Test resource counting by type
#[tokio::test]
async fn test_resource_count_by_type() {
    let manager = MemoryManager::new(100);
    
    manager.register_allocation("tab1", "Tab 1", ResourceType::TabMemory, 1024).await;
    manager.register_allocation("tab2", "Tab 2", ResourceType::TabMemory, 1024).await;
    manager.register_allocation("cache1", "Cache 1", ResourceType::CacheEntry, 1024).await;
    manager.register_allocation("net1", "Network", ResourceType::NetworkBuffer, 1024).await;
    
    let counts = manager.get_resource_count_by_type().await;
    assert_eq!(counts.get(&ResourceType::TabMemory), Some(&2));
    assert_eq!(counts.get(&ResourceType::CacheEntry), Some(&1));
    assert_eq!(counts.get(&ResourceType::NetworkBuffer), Some(&1));
}

/// Test error recovery manager
#[tokio::test]
async fn test_error_recovery_recording() {
    let manager = ErrorRecoveryManager::new();
    
    // Record various errors
    let id1 = manager.record_error(
        "Connection timeout",
        ErrorCategory::Network,
        ErrorSeverity::Error,
        Some("http_client"),
        Some("Request to api.example.com"),
    ).await;
    
    let id2 = manager.record_error(
        "Query failed",
        ErrorCategory::Database,
        ErrorSeverity::Warning,
        Some("storage"),
        None,
    ).await;
    
    assert!(!id1.is_empty());
    assert!(!id2.is_empty());
    assert_ne!(id1, id2);
    
    // Check recent errors
    let recent = manager.get_recent_errors(10).await;
    assert_eq!(recent.len(), 2);
    
    // Check by category
    let network_errors = manager.get_errors_by_category(ErrorCategory::Network).await;
    assert_eq!(network_errors.len(), 1);
    assert_eq!(network_errors[0].message, "Connection timeout");
}

/// Test error recovery with retry
#[tokio::test]
async fn test_error_recovery_retry_success() {
    let manager = ErrorRecoveryManager::new();
    let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    
    let counter_clone = counter.clone();
    let result = manager.execute_with_recovery(
        "test_operation",
        ErrorCategory::Network,
        || {
            let c = counter_clone.clone();
            async move {
                let attempt = c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if attempt < 2 {
                    Err(anyhow::anyhow!("Simulated failure attempt {}", attempt))
                } else {
                    Ok("Success!")
                }
            }
        },
    ).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Success!");
    assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 3);
}

/// Test error recovery max retries exceeded
#[tokio::test]
async fn test_error_recovery_max_retries() {
    let manager = ErrorRecoveryManager::new();
    
    let result: Result<(), _> = manager.execute_with_recovery(
        "always_fails",
        ErrorCategory::Network,
        || async {
            Err(anyhow::anyhow!("Always fails"))
        },
    ).await;
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Max retries exceeded"));
}

/// Test health score calculation
#[tokio::test]
async fn test_health_score() {
    let manager = ErrorRecoveryManager::new();
    
    // Perfect health with no errors
    assert_eq!(manager.get_health_score().await, 100);
    
    // Add some errors
    for i in 0..15 {
        manager.record_error(
            &format!("Error {}", i),
            ErrorCategory::Network,
            ErrorSeverity::Warning,
            None,
            None,
        ).await;
    }
    
    // Health should be reduced
    let score = manager.get_health_score().await;
    assert!(score < 100);
    assert_eq!(score, 80); // 11-50 errors = 80
}

/// Test performance manager timing
#[tokio::test]
async fn test_performance_timing() {
    let manager = PerformanceManager::new();
    
    // Record some operations
    manager.record_operation("db_query", Duration::from_millis(50), true).await;
    manager.record_operation("db_query", Duration::from_millis(100), true).await;
    manager.record_operation("db_query", Duration::from_millis(150), true).await;
    
    let metrics = manager.get_operation_metrics("db_query").await.unwrap();
    assert_eq!(metrics.total_calls, 3);
    assert_eq!(metrics.min_duration_ms, 50);
    assert_eq!(metrics.max_duration_ms, 150);
    assert_eq!(metrics.avg_duration_ms, 100.0);
    assert_eq!(metrics.error_count, 0);
}

/// Test performance manager with failures
#[tokio::test]
async fn test_performance_with_failures() {
    let manager = PerformanceManager::new();
    
    manager.record_operation("api_call", Duration::from_millis(100), true).await;
    manager.record_operation("api_call", Duration::from_millis(100), false).await;
    manager.record_operation("api_call", Duration::from_millis(100), true).await;
    manager.record_operation("api_call", Duration::from_millis(100), false).await;
    
    let rate = manager.get_overall_success_rate().await;
    assert_eq!(rate, 50.0);
}

/// Test slowest operations tracking
#[tokio::test]
async fn test_slowest_operations() {
    let manager = PerformanceManager::new();
    
    manager.record_operation("fast_op", Duration::from_millis(10), true).await;
    manager.record_operation("medium_op", Duration::from_millis(100), true).await;
    manager.record_operation("slow_op", Duration::from_millis(500), true).await;
    
    let slowest = manager.get_slowest_operations(2).await;
    assert_eq!(slowest.len(), 2);
    assert_eq!(slowest[0].name, "slow_op");
    assert_eq!(slowest[1].name, "medium_op");
}

/// Test performance summary
#[tokio::test]
async fn test_performance_summary() {
    let manager = PerformanceManager::new();
    
    for i in 0..10 {
        manager.record_operation(
            &format!("op_{}", i % 3),
            Duration::from_millis(50 + i as u64 * 10),
            i % 4 != 0,
        ).await;
    }
    
    let summary = manager.get_summary().await;
    assert_eq!(summary.total_operations, 10);
    assert_eq!(summary.unique_operations, 3);
    assert!(summary.success_rate > 0.0);
}

/// Test log manager creation and levels
#[tokio::test]
async fn test_log_manager_levels() {
    let manager = LogManager::new();
    
    assert_eq!(manager.get_level().await, LogLevel::Info);
    
    manager.set_level(LogLevel::Debug).await;
    assert_eq!(manager.get_level().await, LogLevel::Debug);
    
    manager.set_level(LogLevel::Error).await;
    assert_eq!(manager.get_level().await, LogLevel::Error);
}

/// Test log config
#[tokio::test]
async fn test_log_config() {
    let config = LogConfig {
        level: LogLevel::Debug,
        log_dir: std::path::PathBuf::from("./test_logs"),
        file_prefix: "test".to_string(),
        rotation: browser_core::LogRotation::Daily,
        max_files: 10,
        include_source_location: true,
        include_thread_ids: false,
        json_format: false,
    };
    
    let manager = LogManager::with_config(config);
    assert_eq!(manager.get_level().await, LogLevel::Debug);
}
