use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use browser_core::TabIPManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, error};
use virtual_ip::{Country, IPGenerator, IPValidator, VirtualIP};

#[derive(Clone)]
pub struct ApiServer {
    tab_manager: Arc<TabIPManager>,
    ip_generator: Arc<IPGenerator>,
}

impl ApiServer {
    pub fn new(tab_manager: Arc<TabIPManager>, ip_generator: Arc<IPGenerator>) -> Self {
        Self {
            tab_manager,
            ip_generator,
        }
    }

    pub async fn router(self: Arc<Self>) -> Router {
        Router::new()
            // Tab endpoints
            .route("/api/tabs", post(create_tab_handler).get(list_tabs_handler))
            .route(
                "/api/tabs/:id",
                get(get_tab_handler).delete(close_tab_handler),
            )
            .route("/api/tabs/:id/rotate-ip", post(rotate_ip_handler))
            .route("/api/tabs/:id/validate", get(validate_ip_handler))
            // Country endpoints
            .route("/api/countries", get(list_countries_handler))
            .with_state(self)
    }

    pub async fn run(self, port: u16) -> Result<()> {
        tracing_subscriber::fmt::init();
        let app = Arc::new(self).router().await;
        let addr = format!("127.0.0.1:{port}");
        info!("API server listening on http://{addr}");
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

// ========= Handlers =========

#[derive(Deserialize)]
struct CreateTabRequest {
    country_code: String,
}

async fn create_tab_handler(
    State(state): State<Arc<ApiServer>>,
    Json(payload): Json<CreateTabRequest>,
) -> Result<Json<TabResponse>, StatusCode> {
    let tab = state.tab_manager
        .create_tab(&payload.country_code)
        .await
        .map_err(|e| {
            error!("Failed to create tab for country '{}': {}", payload.country_code, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(TabResponse::from(tab)))
}

async fn list_tabs_handler(
    State(state): State<Arc<ApiServer>>,
) -> Result<Json<Vec<TabResponse>>, StatusCode> {
    let tabs = state.tab_manager.list_tabs().await;
    Ok(Json(tabs.into_iter().map(TabResponse::from).collect()))
}

async fn get_tab_handler(
    State(state): State<Arc<ApiServer>>,
    Path(id): Path<String>,
) -> Result<Json<TabResponse>, StatusCode> {
    let tab = state.tab_manager
        .get_tab(&id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(TabResponse::from(tab)))
}

async fn close_tab_handler(
    State(state): State<Arc<ApiServer>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state.tab_manager
        .close_tab(&id)
        .await
        .map_err(|e| {
            error!("Failed to close tab '{}': {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct RotateIPRequest {
    new_country: Option<String>,
}

async fn rotate_ip_handler(
    State(state): State<Arc<ApiServer>>,
    Path(id): Path<String>,
    Json(payload): Json<RotateIPRequest>,
) -> Result<Json<VirtualIPResponse>, StatusCode> {
    let ip = state.tab_manager
        .rotate_ip(&id, payload.new_country.as_deref())
        .await
        .map_err(|e| {
            error!("Failed to rotate IP for tab '{}': {:?}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(VirtualIPResponse::from(ip)))
}

async fn validate_ip_handler(
    State(state): State<Arc<ApiServer>>,
    Path(id): Path<String>,
) -> Result<Json<ValidationResponse>, StatusCode> {
    let tab = state.tab_manager.get_tab(&id).await.ok_or(StatusCode::NOT_FOUND)?;

    let validator = IPValidator::new();
    let report = validator
        .validate_comprehensive(&tab.virtual_ip)
        .await
        .map_err(|e| {
            error!("Failed to validate IP for tab '{}': {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ValidationResponse::from(report)))
}

async fn list_countries_handler(
    State(state): State<Arc<ApiServer>>,
) -> Result<Json<Vec<CountryResponse>>, StatusCode> {
    let countries: Vec<Country> = state.ip_generator.list_countries();
    Ok(Json(
        countries
            .into_iter()
            .map(CountryResponse::from)
            .collect(),
    ))
}

// ========= DTOs =========

#[derive(Serialize, Deserialize)]
pub struct TabResponse {
    pub tab_id: String,
    pub ip: String,
    pub country_code: String,
    pub country_name: String,
    pub city: String,
    pub timezone: String,
    pub isp: String,
    pub status: String,
}

impl From<browser_core::TabProfile> for TabResponse {
    fn from(tab: browser_core::TabProfile) -> Self {
        Self {
            tab_id: tab.tab_id,
            ip: tab.virtual_ip.ip.to_string(),
            country_code: tab.virtual_ip.country_code,
            country_name: tab.virtual_ip.country,
            city: tab.virtual_ip.city,
            timezone: tab.virtual_ip.timezone,
            isp: tab.virtual_ip.isp,
            status: format!("{:?}", tab.status),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct VirtualIPResponse {
    pub ip: String,
    pub country_code: String,
    pub country_name: String,
    pub city: String,
    pub region: String,
    pub timezone: String,
    pub language: String,
    pub currency: String,
    pub isp: String,
}

impl From<VirtualIP> for VirtualIPResponse {
    fn from(ip: VirtualIP) -> Self {
        Self {
            ip: ip.ip.to_string(),
            country_code: ip.country_code,
            country_name: ip.country,
            city: ip.city,
            region: ip.region,
            timezone: ip.timezone,
            language: ip.language,
            currency: ip.currency,
            isp: ip.isp,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CountryResponse {
    pub code: String,
    pub name: String,
    pub flag: String,
    pub timezone: String,
    pub language: String,
    pub currency: String,
    pub is_top: bool,
}

impl From<Country> for CountryResponse {
    fn from(c: Country) -> Self {
        Self {
            code: c.code,
            name: c.name,
            flag: c.flag,
            timezone: c.timezone,
            language: c.language,
            currency: c.currency,
            is_top: c.is_top,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ValidationResponse {
    pub ip_matches: bool,
    pub webrtc_secure: bool,
    pub dns_secure: bool,
    pub overall_pass: bool,
}

impl From<virtual_ip::ValidationReport> for ValidationResponse {
    fn from(r: virtual_ip::ValidationReport) -> Self {
        Self {
            ip_matches: r.ip_matches,
            webrtc_secure: !r.webrtc_leaks,
            dns_secure: r.dns_secure,
            overall_pass: r.overall_pass,
        }
    }
}
