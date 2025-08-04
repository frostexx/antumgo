use axum::{
    extract::State,
    http::StatusCode,
    response::sse::{Event, Sse},
    Json,
};
use chrono::Utc;
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use uuid::Uuid;

use crate::{models::types::*, AppState};

#[derive(Deserialize)]
pub struct LoginRequest {
    seed_phrase: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    success: bool,
    session_id: String,
    address: Option<String>,
}

#[derive(Deserialize)]
pub struct WithdrawRequest {
    to_address: String,
    amount: u64,
    sponsor_seed: Option<String>,
}

#[derive(Deserialize)]
pub struct ClaimRequest {
    sponsor_seed: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Validate seed phrase (basic validation)
    if request.seed_phrase.split_whitespace().count() < 12 {
        return Ok(Json(LoginResponse {
            success: false,
            session_id: String::new(),
            address: None,
        }));
    }

    // Try to derive address to validate seed phrase
    let address = match derive_pi_address(&request.seed_phrase) {
        Ok(addr) => Some(addr),
        Err(_) => {
            return Ok(Json(LoginResponse {
                success: false,
                session_id: String::new(),
                address: None,
            }));
        }
    };

    let session_id = Uuid::new_v4().to_string();
    let session = WalletSession {
        id: session_id.clone(),
        seed_phrase: request.seed_phrase,
        created_at: Utc::now(),
        last_activity: Utc::now(),
    };

    state.active_sessions.insert(session_id.clone(), session);

    Ok(Json(LoginResponse {
        success: true,
        session_id,
        address,
    }))
}

pub async fn get_balance(
    State(state): State<AppState>,
) -> Result<Json<pi_network::BalanceInfo>, StatusCode> {
    // Get the first active session (in production, extract from auth header)
    if let Some(session) = state.active_sessions.iter().next() {
        let client = pi_network::PiClient::new().await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        match client.get_balance(&session.seed_phrase).await {
            Ok(balance) => Ok(Json(balance)),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn withdraw(
    State(state): State<AppState>,
    Json(request): Json<WithdrawRequest>,
) -> Result<Json<TransactionResponse>, StatusCode> {
    // Get active session (in production, extract from auth header)
    let session = state.active_sessions.iter().next()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let wallet_seed = session.seed_phrase.clone();
    
    // Start concurrent claiming and transfer
    let claiming_engine = state.claiming_engine.clone();
    let transfer_engine = state.transfer_engine.clone();
    let log_sender = state.log_sender.clone();

    // Start claiming if sponsor seed is provided
    if let Some(sponsor_seed) = request.sponsor_seed {
        let claiming_engine = claiming_engine.clone();
        let wallet_seed = wallet_seed.clone();
        let log_sender = log_sender.clone();
        tokio::spawn(async move {
            let _ = claiming_engine
                .start_claiming_with_sponsor(wallet_seed, sponsor_seed, log_sender)
                .await;
        });
    }

    // Start transfer immediately (independent of claiming)
    let transfer_request = pi_network::TransferRequest {
        to_address: request.to_address,
        amount: request.amount,
        fee: 4800000, // Optimized fee (50% higher than competitor's 3.2M)
        memo: Some("Pi Sweeper Bot Ultimate".to_string()),
    };

    let transfer_engine = transfer_engine.clone();
    let log_sender = log_sender.clone();
    tokio::spawn(async move {
        let _ = transfer_engine
            .start_instant_transfer(wallet_seed, transfer_request, log_sender)
            .await;
    });

    // Don't wait for completion, return immediately
    Ok(Json(TransactionResponse {
        transaction_id: Uuid::new_v4().to_string(),
        status: "initiated".to_string(),
        message: "High-speed withdrawal initiated with concurrent claiming".to_string(),
    }))
}

pub async fn claim(
    State(state): State<AppState>,
    Json(request): Json<ClaimRequest>,
) -> Result<Json<TransactionResponse>, StatusCode> {
    let session = state.active_sessions.iter().next()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let claiming_engine = state.claiming_engine.clone();
    let log_sender = state.log_sender.clone();
    let wallet_seed = session.seed_phrase.clone();

    // Start aggressive claiming
    tokio::spawn(async move {
        let _ = claiming_engine
            .start_claiming_with_sponsor(wallet_seed, request.sponsor_seed, log_sender)
            .await;
    });

    Ok(Json(TransactionResponse {
        transaction_id: Uuid::new_v4().to_string(),
        status: "initiated".to_string(),
        message: "Aggressive claiming initiated with sponsor fee payment".to_string(),
    }))
}

pub async fn get_transactions(
    State(state): State<AppState>,
) -> Result<Json<Vec<TransactionInfo>>, StatusCode> {
    // Get active session
    if let Some(session) = state.active_sessions.iter().next() {
        let client = pi_network::PiClient::new().await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        match client.get_transaction_history(&session.seed_phrase, Some(5)).await {
            Ok(transactions) => {
                let tx_info: Vec<TransactionInfo> = transactions
                    .into_iter()
                    .map(|tx| TransactionInfo {
                        id: tx.id,
                        transaction_type: tx.transaction_type,
                        amount: tx.amount,
                        timestamp: tx.timestamp,
                        status: tx.status,
                        to_address: tx.to_address,
                    })
                    .collect();
                
                Ok(Json(tx_info))
            }
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn get_logs(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = BroadcastStream::new(state.log_sender.subscribe())
        .map(|result| {
            match result {
                Ok(log_entry) => {
                    let json = serde_json::to_string(&log_entry).unwrap_or_default();
                    Ok(Event::default().data(json))
                }
                Err(_) => Ok(Event::default().data("error")),
            }
        });

    Sse::new(stream)
}

pub async fn get_server_time(State(state): State<AppState>) -> Json<ServerTimeResponse> {
    Json(ServerTimeResponse {
        server_time: *state.server_time.read(),
    })
}

pub async fn get_status(State(state): State<AppState>) -> Json<StatusResponse> {
    use std::sync::atomic::Ordering;
    
    Json(StatusResponse {
        claiming_active: state.is_claiming_active.load(Ordering::SeqCst),
        transfer_active: state.is_transfer_active.load(Ordering::SeqCst),
        active_sessions: state.active_sessions.len(),
        server_time: *state.server_time.read(),
    })
}

// Helper function to derive Pi address from seed phrase
fn derive_pi_address(seed_phrase: &str) -> Result<String, Box<dyn std::error::Error>> {
    use tiny_bip39::{Mnemonic, Seed};
    use ed25519_dalek::{PublicKey, SecretKey};
    use sha2::{Digest, Sha256};
    
    let mnemonic = Mnemonic::from_phrase(seed_phrase, tiny_bip39::Language::English)?;
    let seed = Seed::new(&mnemonic, "");
    let secret_key = SecretKey::from_bytes(&seed.as_bytes()[..32])?;
    let public_key = PublicKey::from(&secret_key);
    
    let mut hasher = Sha256::new();
    hasher.update(public_key.as_bytes());
    let hash = hasher.finalize();
    
    Ok(format!("G{}", bs58::encode(&hash[..25]).into_string()))
}