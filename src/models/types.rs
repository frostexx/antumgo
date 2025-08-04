use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSession {
    pub id: String,
    pub seed_phrase: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub transaction_id: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub amount: u64,
    pub timestamp: DateTime<Utc>,
    pub status: String,
    pub to_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerTimeResponse {
    pub server_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub claiming_active: bool,
    pub transfer_active: bool,
    pub active_sessions: usize,
    pub server_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedFeeConfig {
    pub base_fee: u64,
    pub priority_multiplier: f64,
    pub network_congestion_bonus: u64,
    pub max_fee: u64,
}

impl Default for OptimizedFeeConfig {
    fn default() -> Self {
        Self {
            base_fee: 3200000, // Competitor's base fee
            priority_multiplier: 2.0, // 100% higher for priority
            network_congestion_bonus: 2000000, // 2 PI bonus
            max_fee: 10000000, // 10 PI max
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub active_connections: usize,
    pub retry_attempts: u64,
    pub success_rate: f64,
    pub average_response_time: u64, // milliseconds
}