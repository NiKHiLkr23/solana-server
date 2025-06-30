use crate::utils::errors::SolanaError;
use axum::{routing::post, Json, Router};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, system_instruction};
use spl_associated_token_account::get_associated_token_address;
use spl_token::instruction::transfer;
use tracing::info;

#[derive(Deserialize, Serialize)]
pub struct SendSolRequest {
    pub from: Option<String>,
    pub to: Option<String>,
    pub lamports: Option<u64>,
}

#[derive(Deserialize, Serialize)]
pub struct SendTokenRequest {
    pub destination: Option<String>,
    pub mint: Option<String>,
    pub owner: Option<String>,
    pub amount: Option<u64>,
}

#[derive(Serialize)]
pub struct SendSolResponse {
    pub program_id: String,
    pub accounts: Vec<String>,
    pub instruction_data: String,
}

#[derive(Serialize)]
pub struct AccountMetaTokenResponse {
    pub pubkey: String,
    #[serde(rename = "isSigner")]
    pub is_signer: bool,
}

#[derive(Serialize)]
pub struct SendTokenResponse {
    pub program_id: String,
    pub accounts: Vec<AccountMetaTokenResponse>,
    pub instruction_data: String,
}

pub fn routes() -> Router {
    Router::new()
        .route("/send/sol", post(send_sol))
        .route("/send/token", post(send_token))
}

async fn send_sol(
    Json(payload): Json<SendSolRequest>,
) -> Result<Json<serde_json::Value>, SolanaError> {
    info!(
        "POST /send/sol - Request: {}",
        serde_json::to_string(&payload).unwrap_or_default()
    );

    // Validate required fields are present and not empty
    let from = payload
        .from
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .ok_or(SolanaError::MissingFields)?;

    let to = payload
        .to
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .ok_or(SolanaError::MissingFields)?;

    let lamports = payload
        .lamports
        .filter(|&l| l > 0)
        .ok_or(SolanaError::MissingFields)?;

    // Validate lamports amount (reasonable limits)
    const MIN_LAMPORTS: u64 = 1; // Minimum 1 lamport
    const MAX_LAMPORTS: u64 = 100_000_000_000; // Maximum 100 SOL (100 * 10^9 lamports)

    if lamports < MIN_LAMPORTS {
        return Err(SolanaError::InvalidInput(
            "Amount must be at least 1 lamport".to_string(),
        ));
    }

    if lamports > MAX_LAMPORTS {
        return Err(SolanaError::InvalidInput(
            "Amount exceeds maximum limit (100 SOL)".to_string(),
        ));
    }

    // Parse public keys AFTER validation
    let from_pubkey = from
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid sender address".to_string()))?;

    let to_pubkey = to
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid recipient address".to_string()))?;

    // Validate that sender and recipient are different
    if from_pubkey == to_pubkey {
        return Err(SolanaError::InvalidInput(
            "Sender and recipient cannot be the same".to_string(),
        ));
    }

    info!(
        "Creating SOL transfer: {} lamports ({} SOL) from {} to {}",
        lamports,
        lamports as f64 / 1_000_000_000.0,
        from_pubkey,
        to_pubkey
    );

    // Create transfer instruction
    let instruction = system_instruction::transfer(&from_pubkey, &to_pubkey, lamports);

    let response = SendSolResponse {
        program_id: instruction.program_id.to_string(),
        accounts: instruction
            .accounts
            .iter()
            .map(|acc| acc.pubkey.to_string())
            .collect(),
        instruction_data: general_purpose::STANDARD.encode(&instruction.data),
    };

    let json_response = serde_json::json!({
        "success": true,
        "data": response
    });

    info!("Response: 200 - SOL transfer instruction created successfully");

    Ok(Json(json_response))
}

async fn send_token(
    Json(payload): Json<SendTokenRequest>,
) -> Result<Json<serde_json::Value>, SolanaError> {
    info!(
        "POST /send/token - Request: {}",
        serde_json::to_string(&payload).unwrap_or_default()
    );

    // Validate required fields are present and not empty
    let destination = payload
        .destination
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .ok_or(SolanaError::MissingFields)?;

    let mint = payload
        .mint
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .ok_or(SolanaError::MissingFields)?;

    let owner = payload
        .owner
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .ok_or(SolanaError::MissingFields)?;

    let amount = payload
        .amount
        .filter(|&a| a > 0)
        .ok_or(SolanaError::MissingFields)?;

    // Validate amount range (prevent overflow and ensure reasonable limits)
    if amount > u64::MAX / 2 {
        return Err(SolanaError::InvalidInput(
            "Token amount is too large".to_string(),
        ));
    }

    // Parse public keys AFTER validation
    let destination_pubkey = destination
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid destination wallet address".to_string()))?;

    let mint_pubkey = mint
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid mint address".to_string()))?;

    let owner_pubkey = owner
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid owner address".to_string()))?;

    // Validate that owner and destination are different
    if owner_pubkey == destination_pubkey {
        return Err(SolanaError::InvalidInput(
            "Token owner and destination cannot be the same".to_string(),
        ));
    }

    // Derive Associated Token Accounts for both owner and destination
    let source_ata = get_associated_token_address(&owner_pubkey, &mint_pubkey);
    let destination_ata = get_associated_token_address(&destination_pubkey, &mint_pubkey);

    info!(
        "Creating token transfer: {} tokens of mint {} from owner {} (ATA: {}) to destination {} (ATA: {})",
        amount, mint_pubkey, owner_pubkey, source_ata, destination_pubkey, destination_ata
    );

    // Create transfer instruction using derived ATAs
    let instruction = transfer(
        &spl_token::id(),
        &source_ata,
        &destination_ata,
        &owner_pubkey,
        &[],
        amount,
    )
    .map_err(|e| SolanaError::TokenError(e.to_string()))?;

    let accounts: Vec<AccountMetaTokenResponse> = instruction
        .accounts
        .iter()
        .map(|acc| AccountMetaTokenResponse {
            pubkey: acc.pubkey.to_string(),
            is_signer: acc.is_signer,
        })
        .collect();

    let response = SendTokenResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: general_purpose::STANDARD.encode(&instruction.data),
    };

    let json_response = serde_json::json!({
        "success": true,
        "data": response
    });

    info!("Response: 200 - Token transfer instruction created successfully");

    Ok(Json(json_response))
}
