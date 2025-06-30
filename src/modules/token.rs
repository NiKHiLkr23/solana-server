use crate::utils::errors::SolanaError;
use axum::{routing::post, Json, Router};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use spl_token::instruction::{initialize_mint, mint_to};

#[derive(Deserialize)]
pub struct CreateTokenRequest {
    #[serde(rename = "mintAuthority")]
    pub mint_authority: String,
    pub mint: String,
    pub decimals: u8,
}

#[derive(Deserialize)]
pub struct MintTokenRequest {
    pub mint: String,
    pub destination: String,
    pub authority: String,
    pub amount: u64,
}

#[derive(Serialize)]
pub struct InstructionResponse {
    pub program_id: String,
    pub accounts: Vec<AccountMetaResponse>,
    pub instruction_data: String,
}

#[derive(Serialize)]
pub struct AccountMetaResponse {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}

pub fn routes() -> Router {
    Router::new()
        .route("/token/create", post(create_token))
        .route("/token/mint", post(mint_token))
}

async fn create_token(
    Json(payload): Json<CreateTokenRequest>,
) -> Result<Json<serde_json::Value>, SolanaError> {
    // Validate required fields are not empty FIRST
    if payload.mint_authority.trim().is_empty() || payload.mint.trim().is_empty() {
        return Err(SolanaError::MissingFields);
    }

    // Parse public keys AFTER validation
    let mint_authority = payload
        .mint_authority
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid mint authority public key".to_string()))?;

    let mint = payload
        .mint
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid mint public key".to_string()))?;

    // Create initialize mint instruction
    let instruction = initialize_mint(
        &spl_token::id(),
        &mint,
        &mint_authority,
        Some(&mint_authority),
        payload.decimals,
    )
    .map_err(|e| SolanaError::TokenError(e.to_string()))?;

    let accounts: Vec<AccountMetaResponse> = instruction
        .accounts
        .iter()
        .map(|acc| AccountMetaResponse {
            pubkey: acc.pubkey.to_string(),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();

    let response = InstructionResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: general_purpose::STANDARD.encode(&instruction.data),
    };

    Ok(Json(serde_json::json!({
        "success": true,
        "data": response
    })))
}

async fn mint_token(
    Json(payload): Json<MintTokenRequest>,
) -> Result<Json<serde_json::Value>, SolanaError> {
    // Validate required fields are not empty FIRST
    if payload.mint.trim().is_empty()
        || payload.destination.trim().is_empty()
        || payload.authority.trim().is_empty()
        || payload.amount == 0
    {
        return Err(SolanaError::MissingFields);
    }

    // Parse public keys AFTER validation
    let mint = payload
        .mint
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid mint address".to_string()))?;

    let destination = payload
        .destination
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid destination address".to_string()))?;

    let authority = payload
        .authority
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid authority address".to_string()))?;

    // Create mint to instruction
    let instruction = mint_to(
        &spl_token::id(),
        &mint,
        &destination,
        &authority,
        &[],
        payload.amount,
    )
    .map_err(|e| SolanaError::TokenError(e.to_string()))?;

    let accounts: Vec<AccountMetaResponse> = instruction
        .accounts
        .iter()
        .map(|acc| AccountMetaResponse {
            pubkey: acc.pubkey.to_string(),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();

    let response = InstructionResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: general_purpose::STANDARD.encode(&instruction.data),
    };

    Ok(Json(serde_json::json!({
        "success": true,
        "data": response
    })))
}
