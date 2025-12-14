use std::env;
use std::sync::Arc;

use api_server::ApiServer;
use browser_core::TabIPManager;
use virtual_ip::{
    demo_generator,
    load_countries_from_file,
    load_ip_ranges,
    load_ip_ranges_from_file,
    CountryDatabase,
    IPGenerator,
};
use sqlx::SqlitePool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load country and IP range data (file-based override via env).
    let country_path = env::var("COUNTRIES_PATH").ok();
    let ip_ranges_path = env::var("IP_RANGES_PATH").ok();

    let countries = country_path
        .as_deref()
        .map(std::path::Path::new)
        .map(load_countries_from_file)
        .unwrap_or_else(CountryDatabase::load_all_countries);

    let ranges = ip_ranges_path
        .as_deref()
        .map(std::path::Path::new)
        .map(load_ip_ranges_from_file)
        .unwrap_or_else(load_ip_ranges);
    let ip_generator: IPGenerator = if countries.is_empty() || ranges.is_empty() {
        demo_generator()
    } else {
        IPGenerator::new(countries, ranges)
    };

    // Create database pool
    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./data/browser.db?mode=rwc".to_string());
    
    let db_pool = SqlitePool::connect(&db_url).await?;
    
    // Run migrations
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS tabs (
            tab_id TEXT PRIMARY KEY,
            country_code TEXT NOT NULL,
            ip TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            last_active INTEGER NOT NULL
        )"
    )
    .execute(&db_pool)
    .await?;
    
    let tab_manager = Arc::new(TabIPManager::new(ip_generator.clone(), db_pool).await?);
    let server = ApiServer::new(tab_manager, Arc::new(ip_generator));

    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);

    server.run(port).await
}
