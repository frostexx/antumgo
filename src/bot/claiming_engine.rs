use std::{
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};
use tokio::{sync::broadcast, time::sleep};
use tracing::{error, info, warn};
use pi_network::{PiClient, WalletManager};
use crate::{models::types::*, utils::retry::RetryStrategy};

pub struct ClaimingEngine {
    client: Arc<PiClient>,
    retry_config: RetryStrategy,
    network_flood_protection: Arc<AtomicBool>,
    concurrent_claims: usize,
}

impl ClaimingEngine {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            client: Arc::new(PiClient::new().await?),
            retry_config: RetryStrategy::exponential_backoff(
                Duration::from_millis(10), // Start with 10ms
                Duration::from_millis(100), // Max 100ms
                10, // Max attempts
            ),
            network_flood_protection: Arc::new(AtomicBool::new(false)),
            concurrent_claims: 50, // Aggressive concurrent claiming
        })
    }

    pub async fn start_claiming_with_sponsor(
        &self,
        wallet_seed: String,
        sponsor_seed: String,
        log_sender: broadcast::Sender<LogEntry>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        info!("🎯 Starting aggressive claiming with sponsor fee payment");

        // Create multiple concurrent claiming tasks
        let mut handles = Vec::new();
        
        for i in 0..self.concurrent_claims {
            let client = self.client.clone();
            let wallet_seed = wallet_seed.clone();
            let sponsor_seed = sponsor_seed.clone();
            let log_sender = log_sender.clone();
            let retry_config = self.retry_config.clone();
            
            let handle = tokio::spawn(async move {
                Self::claim_worker(
                    client,
                    wallet_seed,
                    sponsor_seed,
                    log_sender,
                    retry_config,
                    i,
                ).await
            });
            
            handles.push(handle);
            
            // Stagger the starts slightly to avoid initial collision
            sleep(Duration::from_millis(1)).await;
        }

        // Wait for first success or all failures
        let mut success_count = 0;
        for handle in handles {
            match handle.await {
                Ok(Ok(_)) => {
                    success_count += 1;
                    if success_count == 1 {
                        let _ = log_sender.send(LogEntry {
                            timestamp: chrono::Utc::now(),
                            level: "SUCCESS".to_string(),
                            message: format!("✅ First claim succeeded in {:?}", start_time.elapsed()),
                        });
                        break; // Stop after first success
                    }
                }
                Ok(Err(e)) => {
                    warn!("Claim worker failed: {}", e);
                }
                Err(e) => {
                    error!("Claim worker panicked: {}", e);
                }
            }
        }

        if success_count == 0 {
            let _ = log_sender.send(LogEntry {
                timestamp: chrono::Utc::now(),
                level: "ERROR".to_string(),
                message: "❌ All claiming attempts failed".to_string(),
            });
        }

        Ok(())
    }

    async fn claim_worker(
        client: Arc<PiClient>,
        wallet_seed: String,
        sponsor_seed: String,
        log_sender: broadcast::Sender<LogEntry>,
        retry_config: RetryStrategy,
        worker_id: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut attempts = 0;
        let mut delay = Duration::from_millis(1);

        loop {
            attempts += 1;
            
            let _ = log_sender.send(LogEntry {
                timestamp: chrono::Utc::now(),
                level: "INFO".to_string(),
                message: format!("🔄 Worker {} - Claim attempt {}", worker_id, attempts),
            });

            match client.claim_with_sponsor_fee(&wallet_seed, &sponsor_seed).await {
                Ok(result) => {
                    let _ = log_sender.send(LogEntry {
                        timestamp: chrono::Utc::now(),
                        level: "SUCCESS".to_string(),
                        message: format!("✅ Worker {} - Claim successful: {}", worker_id, result.transaction_id),
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
                        return Err(e.into());
                    }

                    // Adaptive delay based on error type
                    delay = match e.to_string().as_str() {
                        s if s.contains("rate limit") => Duration::from_millis(50),
                        s if s.contains("network") => Duration::from_millis(10),
                        _ => Duration::from_millis(5),
                    };

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
    }
}