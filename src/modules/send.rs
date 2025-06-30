use crate::utils::errors::SolanaError;
use axum::{routing::post, Json, Router};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, system_instruction};
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

    // Parse public keys AFTER validation
    let from_pubkey = from
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid sender address".to_string()))?;

    let to_pubkey = to
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid recipient address".to_string()))?;

    // Additional validation - check if addresses are valid Solana addresses
    if from_pubkey == to_pubkey {
        return Err(SolanaError::InvalidInput(
            "Sender and recipient cannot be the same".to_string(),
        ));
    }

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

    info!("Response: 200");

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

    // Parse public keys AFTER validation
    let destination_pubkey = destination
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid destination address".to_string()))?;

    let mint_pubkey = mint
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid mint address".to_string()))?;

    let owner_pubkey = owner
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid owner address".to_string()))?;

    // Get associated token accounts (simplified - in reality you'd derive these)
    let source =
        spl_associated_token_account::get_associated_token_address(&owner_pubkey, &mint_pubkey);
    let dest = spl_associated_token_account::get_associated_token_address(
        &destination_pubkey,
        &mint_pubkey,
    );

    // Create transfer instruction
    let instruction = transfer(&spl_token::id(), &source, &dest, &owner_pubkey, &[], amount)
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

    info!("Response: 200");

    Ok(Json(json_response))
}
