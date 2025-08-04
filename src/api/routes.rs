use axum::{
    routing::{get, post},
    Router,
};
use crate::{api::handlers, AppState};

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/api/login", post(handlers::login))
        .route("/api/balance", get(handlers::get_balance))
        .route("/api/withdraw", post(handlers::withdraw))
        .route("/api/claim", post(handlers::claim))
        .route("/api/transactions", get(handlers::get_transactions))
        .route("/api/logs", get(handlers::get_logs))
        .route("/api/time", get(handlers::get_server_time))
        .route("/api/status", get(handlers::get_status))
}