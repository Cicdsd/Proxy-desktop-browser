use browser_core::TabIPManager;
use virtual_ip::demo_generator;
use sqlx::SqlitePool;

async fn create_test_manager() -> TabIPManager {
    let generator = demo_generator();
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
    
    TabIPManager::new(generator, pool)
        .await
        .expect("Failed to create TabIPManager")
}

#[tokio::test]
async fn test_tab_creation_lifecycle() {
    let manager = create_test_manager().await;

    let tab = manager.create_tab("US").await.unwrap();
    assert_eq!(tab.virtual_ip.country_code, "US");

    let retrieved = manager.get_tab(&tab.tab_id).await.unwrap();
    assert_eq!(retrieved.tab_id, tab.tab_id);

    manager.close_tab(&tab.tab_id).await.unwrap();
    assert!(manager.get_tab(&tab.tab_id).await.is_none());
}

#[tokio::test]
async fn test_ip_rotation() {
    let manager = create_test_manager().await;

    let tab = manager.create_tab("US").await.unwrap();
    let original_ip = tab.virtual_ip.ip;

    let new_ip = manager.rotate_ip(&tab.tab_id, None).await.unwrap();

    assert_ne!(original_ip, new_ip.ip);
    assert_eq!(new_ip.country_code, "US");
}

#[tokio::test]
async fn test_multiple_tabs_isolation() {
    let manager = create_test_manager().await;

    // Create multiple tabs with the same country - IPs should still be different
    let tab1 = manager.create_tab("US").await.unwrap();
    let tab2 = manager.create_tab("GB").await.unwrap();
    let tab3 = manager.create_tab("US").await.unwrap();  // Different tab, same country

    assert_ne!(tab1.virtual_ip.ip, tab2.virtual_ip.ip);
    // tab3 should have different ID than tab1
    assert_ne!(tab1.tab_id, tab3.tab_id);
    assert_eq!(tab1.virtual_ip.country_code, "US");
    assert_eq!(tab2.virtual_ip.country_code, "GB");
    assert_eq!(tab3.virtual_ip.country_code, "US");
}
