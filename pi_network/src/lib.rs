use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use ed25519_dalek::{Keypair, PublicKey, SecretKey}; // Remove unused Signature
use bip39::{Mnemonic, Language}; // Remove Seed - it's not exported in bip39 v1.0
use sha2::{Digest, Sha256};
use bs58;

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
    #[error("Invalid seed phrase")]
    InvalidSeedPhrase,
    #[error("Cryptographic error: {0}")]
    Crypto(String),
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
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(50)
            .pool_idle_timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            base_url: "https://api.mainnet.minepi.com".to_string(), // Real Pi mainnet API
            max_retries: 10,
        })
    }

    // Convert seed phrase to Pi Network address using correct bip39 API
    fn seed_to_address(&self, seed_phrase: &str) -> Result<String, PiError> {
        // Parse mnemonic using bip39 v1.0 API
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, seed_phrase)
            .map_err(|_| PiError::InvalidSeedPhrase)?;
        
        // Generate seed from mnemonic (bip39 v1.0 uses to_seed method)
        let seed = mnemonic.to_seed("");
        
        // Create secret key from first 32 bytes of seed
        let secret_key_bytes: [u8; 32] = seed[..32].try_into()
            .map_err(|_| PiError::Crypto("Invalid seed length".to_string()))?;
        
        let secret_key = SecretKey::from_bytes(&secret_key_bytes)
            .map_err(|e| PiError::Crypto(e.to_string()))?;
        
        let public_key = PublicKey::from(&secret_key);
        
        // Pi Network uses a specific address format
        let mut hasher = Sha256::new();
        hasher.update(public_key.as_bytes());
        let hash = hasher.finalize();
        
        // Pi addresses typically start with "G" (similar to Stellar)
        Ok(format!("G{}", bs58::encode(&hash[..25]).into_string()))
    }

    pub async fn get_balance(&self, wallet_seed: &str) -> Result<BalanceInfo, PiError> {
        let address = self.seed_to_address(wallet_seed)?;
        
        let url = format!("{}/v1/accounts/{}/balance", self.base_url, address);
        
        for attempt in 1..=self.max_retries {
            match self.client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let balance_data: serde_json::Value = response.json().await?;
                        
                        // Parse real Pi Network balance response
                        let available = balance_data["available"]
                            .as_str()
                            .and_then(|s| s.parse::<u64>().ok())
                            .unwrap_or(0);
                        
                        // Fix: Add explicit type annotation for locked_balances
                        let locked_balances: Vec<LockedBalance> = balance_data["locked"]
                            .as_array()
                            .map(|arr| {
                                arr.iter().filter_map(|item| {
                                    Some(LockedBalance {
                                        amount: item["amount"].as_str()?.parse().ok()?,
                                        unlock_time: chrono::DateTime::parse_from_rfc3339(
                                            item["unlock_time"].as_str()?
                                        ).ok()?.with_timezone(&chrono::Utc),
                                        id: item["id"].as_str()?.to_string(),
                                    })
                                }).collect()
                            })
                            .unwrap_or_default();
                        
                        let total = available + locked_balances.iter().map(|l| l.amount).sum::<u64>();
                        
                        return Ok(BalanceInfo {
                            available,
                            locked: locked_balances,
                            total,
                        });
                    } else if response.status().as_u16() == 429 {
                        // Rate limited, wait and retry
                        tokio::time::sleep(Duration::from_millis(100 * attempt as u64)).await;
                        continue;
                    } else {
                        return Err(PiError::Api(format!("HTTP {}: {}", 
                            response.status(), 
                            response.text().await.unwrap_or_default()
                        )));
                    }
                }
                Err(e) => {
                    if attempt == self.max_retries {
                        return Err(PiError::Network(e));
                    }
                    tokio::time::sleep(Duration::from_millis(50 * attempt as u64)).await;
                }
            }
        }
        
        Err(PiError::Api("Max retries exceeded".to_string()))
    }

    pub async fn claim_with_sponsor_fee(
        &self,
        wallet_seed: &str,
        sponsor_seed: &str,
    ) -> Result<ClaimResult, PiError> {
        let wallet_address = self.seed_to_address(wallet_seed)?;
        let sponsor_address = self.seed_to_address(sponsor_seed)?;
        
        let claim_payload = serde_json::json!({
            "type": "claim",
            "wallet_address": wallet_address,
            "sponsor_address": sponsor_address,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        let url = format!("{}/v1/transactions/claim", self.base_url);
        
        for attempt in 1..=self.max_retries {
            match self.client
                .post(&url)
                .json(&claim_payload)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        let result: serde_json::Value = response.json().await?;
                        
                        return Ok(ClaimResult {
                            transaction_id: result["transaction_id"]
                                .as_str()
                                .unwrap_or("unknown")
                                .to_string(),
                            amount_claimed: result["amount_claimed"]
                                .as_str()
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(0),
                            fee_paid: result["fee_paid"]
                                .as_str()
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(0),
                        });
                    } else if response.status().as_u16() == 429 {
                        tokio::time::sleep(Duration::from_millis(100 * attempt as u64)).await;
                        continue;
                    } else {
                        return Err(PiError::Api(format!("Claim failed: HTTP {}", response.status())));
                    }
                }
                Err(e) => {
                    if attempt == self.max_retries {
                        return Err(PiError::Network(e));
                    }
                    tokio::time::sleep(Duration::from_millis(50 * attempt as u64)).await;
                }
            }
        }
        
        Err(PiError::Api("Claim max retries exceeded".to_string()))
    }

    pub async fn transfer(
        &self,
        wallet_seed: &str,
        request: &TransferRequest,
    ) -> Result<TransferResult, PiError> {
        let from_address = self.seed_to_address(wallet_seed)?;
        
        let transfer_payload = serde_json::json!({
            "type": "transfer",
            "from_address": from_address,
            "to_address": request.to_address,
            "amount": request.amount.to_string(),
            "fee": request.fee.to_string(),
            "memo": request.memo,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        let url = format!("{}/v1/transactions/transfer", self.base_url);
        
        for attempt in 1..=self.max_retries {
            match self.client
                .post(&url)
                .json(&transfer_payload)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        let result: serde_json::Value = response.json().await?;
                        
                        return Ok(TransferResult {
                            transaction_id: result["transaction_id"]
                                .as_str()
                                .unwrap_or("unknown")
                                .to_string(),
                            status: result["status"]
                                .as_str()
                                .unwrap_or("pending")
                                .to_string(),
                            fee_paid: result["fee_paid"]
                                .as_str()
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(request.fee),
                        });
                    } else if response.status().as_u16() == 429 {
                        tokio::time::sleep(Duration::from_millis(100 * attempt as u64)).await;
                        continue;
                    } else if response.status().as_u16() == 400 {
                        let error_text = response.text().await.unwrap_or_default();
                        if error_text.contains("insufficient") {
                            return Err(PiError::InsufficientBalance);
                        }
                        return Err(PiError::Api(format!("Transfer failed: {}", error_text)));
                    } else {
                        return Err(PiError::Api(format!("Transfer failed: HTTP {}", response.status())));
                    }
                }
                Err(e) => {
                    if attempt == self.max_retries {
                        return Err(PiError::Network(e));
                    }
                    tokio::time::sleep(Duration::from_millis(50 * attempt as u64)).await;
                }
            }
        }
        
        Err(PiError::Api("Transfer max retries exceeded".to_string()))
    }

    pub async fn get_transaction_history(&self, wallet_seed: &str, limit: Option<u32>) -> Result<Vec<Transaction>, PiError> {
        let address = self.seed_to_address(wallet_seed)?;
        let limit = limit.unwrap_or(10);
        
        let url = format!("{}/v1/accounts/{}/transactions?limit={}", self.base_url, address, limit);
        
        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            let transactions: serde_json::Value = response.json().await?;
            
            let mut result = Vec::new();
            if let Some(tx_array) = transactions["transactions"].as_array() {
                for tx in tx_array {
                    if let Some(transaction) = self.parse_transaction(tx) {
                        result.push(transaction);
                    }
                }
            }
            
            Ok(result)
        } else {
            Err(PiError::Api(format!("Failed to get transaction history: HTTP {}", response.status())))
        }
    }

    fn parse_transaction(&self, tx: &serde_json::Value) -> Option<Transaction> {
        Some(Transaction {
            id: tx["id"].as_str()?.to_string(),
            transaction_type: tx["type"].as_str()?.to_string(),
            amount: tx["amount"].as_str()?.parse().ok()?,
            timestamp: chrono::DateTime::parse_from_rfc3339(tx["timestamp"].as_str()?)
                .ok()?
                .with_timezone(&chrono::Utc),
            status: tx["status"].as_str()?.to_string(),
            to_address: tx["to_address"].as_str().map(|s| s.to_string()),
            from_address: tx["from_address"].as_str().map(|s| s.to_string()),
            fee: tx["fee"].as_str()?.parse().ok()?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub amount: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub status: String,
    pub to_address: Option<String>,
    pub from_address: Option<String>,
    pub fee: u64,
}