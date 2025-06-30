use crate::utils::errors::SolanaError;
use axum::{routing::post, Json, Router};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, system_instruction};
use spl_token::instruction::transfer;

#[derive(Deserialize)]
pub struct SendSolRequest {
    pub from: String,
    pub to: String,
    pub lamports: u64,
}

#[derive(Deserialize)]
pub struct SendTokenRequest {
    pub destination: String,
    pub mint: String,
    pub owner: String,
    pub amount: u64,
}

#[derive(Serialize)]
pub struct SendSolResponse {
    pub program_id: String,
    pub accounts: Vec<String>,
    pub instruction_data: Vec<u8>,
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
    pub instruction_data: Vec<u8>,
}

pub fn routes() -> Router {
    Router::new()
        .route("/send/sol", post(send_sol))
        .route("/send/token", post(send_token))
}

async fn send_sol(
    Json(payload): Json<SendSolRequest>,
) -> Result<Json<serde_json::Value>, SolanaError> {
    // Validate required fields are not empty FIRST
    if payload.from.trim().is_empty() || payload.to.trim().is_empty() || payload.lamports == 0 {
        return Err(SolanaError::MissingFields);
    }

    // Parse public keys AFTER validation
    let from_pubkey = payload
        .from
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid sender address".to_string()))?;

    let to_pubkey = payload
        .to
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid recipient address".to_string()))?;

    // Additional validation - check if addresses are valid Solana addresses
    if from_pubkey == to_pubkey {
        return Err(SolanaError::InvalidInput(
            "Sender and recipient cannot be the same".to_string(),
        ));
    }

    // Create transfer instruction
    let instruction = system_instruction::transfer(&from_pubkey, &to_pubkey, payload.lamports);

    let response = SendSolResponse {
        program_id: instruction.program_id.to_string(),
        accounts: instruction
            .accounts
            .iter()
            .map(|acc| acc.pubkey.to_string())
            .collect(),
        instruction_data: instruction.data,
    };

    Ok(Json(serde_json::json!({
        "success": true,
        "data": response
    })))
}

async fn send_token(
    Json(payload): Json<SendTokenRequest>,
) -> Result<Json<serde_json::Value>, SolanaError> {
    // Validate required fields are not empty FIRST
    if payload.destination.trim().is_empty()
        || payload.mint.trim().is_empty()
        || payload.owner.trim().is_empty()
        || payload.amount == 0
    {
        return Err(SolanaError::MissingFields);
    }

    // Parse public keys AFTER validation
    let destination = payload
        .destination
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid destination address".to_string()))?;

    let mint = payload
        .mint
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid mint address".to_string()))?;

    let owner = payload
        .owner
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid owner address".to_string()))?;

    // Get associated token accounts (simplified - in reality you'd derive these)
    let source = spl_associated_token_account::get_associated_token_address(&owner, &mint);
    let dest = spl_associated_token_account::get_associated_token_address(&destination, &mint);

    // Create transfer instruction
    let instruction = transfer(
        &spl_token::id(),
        &source,
        &dest,
        &owner,
        &[],
        payload.amount,
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
        instruction_data: instruction.data,
    };

    Ok(Json(serde_json::json!({
        "success": true,
        "data": response
    })))
}
