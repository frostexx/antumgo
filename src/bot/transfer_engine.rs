use std::{
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};
use tokio::{sync::broadcast, time::sleep};
use tracing::{error, info, warn};
use pi_network::{PiClient, TransferRequest};
use crate::{models::types::*, utils::retry::RetryStrategy};

pub struct TransferEngine {
    client: Arc<PiClient>,
    retry_config: RetryStrategy,
    is_active: Arc<AtomicBool>,
    concurrent_transfers: usize,
}

impl TransferEngine {
    pub async fn new() -> Result<Self, BotError> {
        Ok(Self {
            client: Arc::new(PiClient::new().await.map_err(|e| BotError::Network(e.to_string()))?),
            retry_config: RetryStrategy::exponential_backoff(
                Duration::from_millis(5), // Even faster for transfers
                Duration::from_millis(50),
                20, // More attempts for transfers
            ),
            is_active: Arc::new(AtomicBool::new(false)),
            concurrent_transfers: 25, // Aggressive concurrent transfers
        })
    }

    pub async fn start_instant_transfer(
        &self,
        wallet_seed: String,
        transfer_request: TransferRequest,
        log_sender: broadcast::Sender<LogEntry>,
    ) -> Result<(), BotError> {
        use std::sync::atomic::Ordering;
        
        self.is_active.store(true, Ordering::SeqCst);
        let start_time = Instant::now();
        
        info!("🚀 Starting instant transfer with optimized fees");

        // Calculate optimal fee (higher than competitors)
        let optimized_fee = self.calculate_optimal_fee(&transfer_request).await?;
        let mut transfer_req = transfer_request;
        transfer_req.fee = optimized_fee;

        let _ = log_sender.send(LogEntry {
            timestamp: chrono::Utc::now(),
            level: "INFO".to_string(),
            message: format!("💰 Using optimized fee: {} PI (higher than standard)", optimized_fee),
        });

        // Launch multiple concurrent transfer attempts
        let mut handles = Vec::new();
        
        for i in 0..self.concurrent_transfers {
            let client = self.client.clone();
            let wallet_seed = wallet_seed.clone();
            let transfer_req = transfer_req.clone();
            let log_sender = log_sender.clone();
            let retry_config = self.retry_config.clone();
            let is_active = self.is_active.clone();
            
            let handle = tokio::spawn(async move {
                Self::transfer_worker(
                    client,
                    wallet_seed,
                    transfer_req,
                    log_sender,
                    retry_config,
                    is_active,
                    i,
                ).await
            });
            
            handles.push(handle);
        }

        // Wait for first success
        let mut success_count = 0;
        for handle in handles {
            match handle.await {
                Ok(Ok(_)) => {
                    success_count += 1;
                    if success_count == 1 {
                        self.is_active.store(false, Ordering::SeqCst);
                        let _ = log_sender.send(LogEntry {
                            timestamp: chrono::Utc::now(),
                            level: "SUCCESS".to_string(),
                            message: format!("✅ Transfer completed in {:?}", start_time.elapsed()),
                        });
                        break;
                    }
                }
                Ok(Err(e)) => {
                    warn!("Transfer worker failed: {}", e);
                }
                Err(e) => {
                    error!("Transfer worker panicked: {}", e);
                }
            }
        }

        if success_count == 0 {
            self.is_active.store(false, Ordering::SeqCst);
            let _ = log_sender.send(LogEntry {
                timestamp: chrono::Utc::now(),
                level: "ERROR".to_string(),
                message: "❌ All transfer attempts failed".to_string(),
            });
        }

        Ok(())
    }

    async fn calculate_optimal_fee(&self, _request: &TransferRequest) -> Result<u64, BotError> {
        // Use higher fees than competitors to ensure priority
        let base_fee = 3200000; // Competitor's fee
        let priority_multiplier = 1.5; // 50% higher for priority
        let network_congestion_bonus = 1000000; // Extra for network congestion
        
        Ok(((base_fee as f64) * priority_multiplier) as u64 + network_congestion_bonus)
    }

    async fn transfer_worker(
        client: Arc<PiClient>,
        wallet_seed: String,
        transfer_request: TransferRequest,
        log_sender: broadcast::Sender<LogEntry>,
        retry_config: RetryStrategy,
        is_active: Arc<AtomicBool>,
        worker_id: usize,
    ) -> Result<(), BotError> {
        use std::sync::atomic::Ordering;
        
        let mut attempts = 0;

        while is_active.load(Ordering::SeqCst) {
            attempts += 1;
            
            let _ = log_sender.send(LogEntry {
                timestamp: chrono::Utc::now(),
                level: "INFO".to_string(),
                message: format!("🔄 Worker {} - Transfer attempt {}", worker_id, attempts),
            });

            match client.transfer(&wallet_seed, &transfer_request).await {
                Ok(result) => {
                    let _ = log_sender.send(LogEntry {
                        timestamp: chrono::Utc::now(),
                        level: "SUCCESS".to_string(),
                        message: format!("✅ Worker {} - Transfer successful: {}", worker_id, result.transaction_id),
                    });
                    return Ok(());
                }
                Err(e) => {
                    if attempts >= retry_config.max_attempts {
                        let _ = log_sender.send(LogEntry {
                            timestamp: chrono::Utc::now(),
                            level: "ERROR".to_string(),
                            message: format!("❌ Worker {} - Max attempts reached: {}", worker_id, e),
                        });
                        return Err(BotError::Api(e.to_string()));
                    }

                    // Ultra-fast retry for transfers
                    let delay = Duration::from_millis(2);

                    let _ = log_sender.send(LogEntry {
                        timestamp: chrono::Utc::now(),
                        level: "WARN".to_string(),
                        message: format!("⚠️ Worker {} - Attempt {} failed, retrying in {:?}: {}", 
                                       worker_id, attempts, delay, e),
                    });

                    sleep(delay).await;
                }
            }
        }

        Ok(())
    }
}