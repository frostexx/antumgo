use axum::{
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{sync::broadcast, time::sleep};
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing::{error, info, warn};

mod api;
mod bot;
mod models;
mod utils;

use bot::{claiming_engine::ClaimingEngine, transfer_engine::TransferEngine};
use models::types::*;
use utils::{rate_limiter::RateLimiter, retry::RetryConfig};

#[derive(Clone)]
pub struct AppState {
    pub active_sessions: Arc<DashMap<String, WalletSession>>,
    pub claiming_engine: Arc<ClaimingEngine>,
    pub transfer_engine: Arc<TransferEngine>,
    pub rate_limiter: Arc<RateLimiter>,
    pub log_sender: broadcast::Sender<LogEntry>,
    pub is_claiming_active: Arc<AtomicBool>,
    pub is_transfer_active: Arc<AtomicBool>,
    pub server_time: Arc<RwLock<DateTime<Utc>>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::init();
    
    let (log_sender, _) = broadcast::channel(1000);
    
    let state = AppState {
        active_sessions: Arc::new(DashMap::new()),
        claiming_engine: Arc::new(ClaimingEngine::new().await?),
        transfer_engine: Arc::new(TransferEngine::new().await?),
        rate_limiter: Arc::new(RateLimiter::new()),
        log_sender: log_sender.clone(),
        is_claiming_active: Arc::new(AtomicBool::new(false)),
        is_transfer_active: Arc::new(AtomicBool::new(false)),
        server_time: Arc::new(RwLock::new(Utc::now())),
    };

    // Start time updater
    let time_state = state.server_time.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        loop {
            interval.tick().await;
            *time_state.write() = Utc::now();
        }
    });

    let app = Router::new()
        .route("/", get(serve_frontend))
        .route("/api/login", post(api::handlers::login))
        .route("/api/balance", get(api::handlers::get_balance))
        .route("/api/withdraw", post(api::handlers::withdraw))
        .route("/api/claim", post(api::handlers::claim))
        .route("/api/transactions", get(api::handlers::get_transactions))
        .route("/api/logs", get(api::handlers::get_logs))
        .route("/api/time", get(api::handlers::get_server_time))
        .route("/api/status", get(api::handlers::get_status))
        .nest_service("/static", ServeDir::new("frontend/static"))
        .layer(CorsLayer::permissive())
        .with_state(state);

    info!("🚀 Pi Sweeper Bot Ultimate starting on 0.0.0.0:3000");
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn serve_frontend() -> Html<&'static str> {
    Html(include_str!("../frontend/static/index.html"))
}