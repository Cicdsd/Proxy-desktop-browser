use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

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

    // Create TabIPManager with in-memory storage (no database)
    let tab_manager = Arc::new(Mutex::new(
        TabIPManager::new(ip_generator.clone())
    ));
    let server = ApiServer::new(tab_manager, Arc::new(ip_generator));

    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);

    server.run(port).await
}
