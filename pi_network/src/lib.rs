use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;

#[derive(Error, Debug)]
pub enum PiError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("API error: {0}")]
    Api(String),
    #[error("Rate limit exceeded")]
    RateLimit,
    #[error("Insufficient balance")]
    InsufficientBalance,
}

#[derive(Clone)]
pub struct PiClient {
    client: Client,
    base_url: String,
    max_retries: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransferRequest {
    pub to_address: String,
    pub amount: u64,
    pub fee: u64,
    pub memo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferResult {
    pub transaction_id: String,
    pub status: String,
    pub fee_paid: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaimResult {
    pub transaction_id: String,
    pub amount_claimed: u64,
    pub fee_paid: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceInfo {
    pub available: u64,
    pub locked: Vec<LockedBalance>,
    pub total: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LockedBalance {
    pub amount: u64,
    pub unlock_time: chrono::DateTime<chrono::Utc>,
    pub id: String,
}

impl PiClient {
    pub async fn new() -> Result<Self, PiError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .pool_max_idle_per_host(50) // High connection pool
            .pool_idle_timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            base_url: "https://api.minepi.com".to_string(), // Placeholder
            max_retries: 10,
        })
    }

    pub async fn claim_with_sponsor_fee(
        &self,
        _wallet_seed: &str,
        _sponsor_seed: &str,
    ) -> Result<ClaimResult, PiError> {
        // Implementation for claiming with sponsor paying the fee
        let mut attempts = 0;
        
        loop {
            attempts += 1;
            
            match self.attempt_claim_with_sponsor(_wallet_seed, _sponsor_seed).await {
                Ok(result) => return Ok(result),
                Err(e) if attempts >= self.max_retries => return Err(e),
                Err(PiError::RateLimit) => {
                    sleep(Duration::from_millis(10)).await;
                    continue;
                }
                Err(_) => {
                    sleep(Duration::from_millis(5)).await;
                    continue;
                }
            }
        }
    }

    async fn attempt_claim_with_sponsor(
        &self,
        _wallet_seed: &str,
        _sponsor_seed: &str,
    ) -> Result<ClaimResult, PiError> {
        // This would implement the actual claiming logic
        // where the sponsor pays the transaction fee
        
        // Simulate network call
        sleep(Duration::from_millis(1)).await;
        
        // For now, return a mock result
        Ok(ClaimResult {
            transaction_id: format!("claim_{}", uuid::Uuid::new_v4()),
            amount_claimed: 1000000, // 1 PI
            fee_paid: 100000, // Paid by sponsor
        })
    }

    pub async fn transfer(
        &self,
        _wallet_seed: &str,
        request: &TransferRequest,
    ) -> Result<TransferResult, PiError> {
        let mut attempts = 0;
        
        loop {
            attempts += 1;
            
            match self.attempt_transfer(_wallet_seed, request).await {
                Ok(result) => return Ok(result),
                Err(e) if attempts >= self.max_retries => return Err(e),
                Err(PiError::RateLimit) => {
                    sleep(Duration::from_millis(5)).await;
                    continue;
                }
                Err(_) => {
                    sleep(Duration::from_millis(2)).await;
                    continue;
                }
            }
        }
    }

    async fn attempt_transfer(
        &self,
        _wallet_seed: &str,
        request: &TransferRequest,
    ) -> Result<TransferResult, PiError> {
        // This would implement the actual transfer logic
        
        // Simulate network call with optimized fee
        sleep(Duration::from_millis(1)).await;
        
        // For now, return a mock result
        Ok(TransferResult {
            transaction_id: format!("transfer_{}", uuid::Uuid::new_v4()),
            status: "confirmed".to_string(),
            fee_paid: request.fee,
        })
    }

    pub async fn get_balance(&self, _wallet_seed: &str) -> Result<BalanceInfo, PiError> {
        // Implementation for getting wallet balance
        sleep(Duration::from_millis(1)).await;
        
        Ok(BalanceInfo {
            available: 5000000, // 5 PI
            locked: vec![
                LockedBalance {
                    amount: 10000000, // 10 PI
                    unlock_time: chrono::Utc::now() + chrono::Duration::hours(1),
                    id: "locked_1".to_string(),
                }
            ],
            total: 15000000, // 15 PI
        })
    }
}