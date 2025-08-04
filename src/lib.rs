//! Pi Sweeper Bot Ultimate
//! 
//! A high-performance, multi-threaded Pi Network sweeper bot designed to outperform
//! all competitors through advanced concurrency, optimal fee strategies, and
//! intelligent network protection.

pub mod api;
pub mod bot;
pub mod models;
pub mod utils;

pub use api::*;
pub use bot::*;
pub use models::*;
pub use utils::*;

use std::sync::{atomic::AtomicBool, Arc};
use dashmap::DashMap;
use parking_lot::RwLock;
use tokio::sync::broadcast;
use chrono::{DateTime, Utc};

/// Main application state shared across all components
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

/// Bot configuration for optimal performance
#[derive(Debug, Clone)]
pub struct BotConfig {
    pub max_concurrent_claims: usize,
    pub max_concurrent_transfers: usize,
    pub aggressive_mode: bool,
    pub fee_optimization_enabled: bool,
    pub network_protection_enabled: bool,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            max_concurrent_claims: 100,
            max_concurrent_transfers: 50,
            aggressive_mode: true,
            fee_optimization_enabled: true,
            network_protection_enabled: true,
        }
    }
}