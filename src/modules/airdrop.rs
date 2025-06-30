use crate::utils::errors::SolanaError;
use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Deserialize)]
pub struct AirdropRequest {
    pub public_key: String,
    pub amount_sol: f64,
}

#[derive(Serialize)]
pub struct AirdropResponse {
    pub transaction_signature: String,
    pub public_key: String,
    pub amount_sol: f64,
    pub amount_lamports: u64,
    pub message: String,
}

pub fn routes() -> Router {
    Router::new().route("/airdrop", post(request_airdrop))
}

async fn request_airdrop(
    Json(payload): Json<AirdropRequest>,
) -> Result<Json<AirdropResponse>, SolanaError> {
    // Validate SOL amount (devnet has limits)
    if payload.amount_sol <= 0.0 || payload.amount_sol > 5.0 {
        return Err(SolanaError::InvalidInput(
            "Amount must be between 0.1 and 5.0 SOL for devnet".to_string(),
        ));
    }

    let pubkey = payload
        .public_key
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid public key format".to_string()))?;

    let client = crate::utils::solana_client::get_rpc_client();
    let amount_lamports = (payload.amount_sol * 1_000_000_000.0) as u64;

    let signature = client
        .request_airdrop(&pubkey, amount_lamports)
        .map_err(SolanaError::ClientError)?;

    // Wait for confirmation
    let confirmation = client
        .confirm_transaction(&signature)
        .map_err(SolanaError::ClientError)?;

    if !confirmation {
        return Err(SolanaError::TransactionFailed(
            "Airdrop transaction failed to confirm".to_string(),
        ));
    }

    Ok(Json(AirdropResponse {
        transaction_signature: signature.to_string(),
        public_key: payload.public_key,
        amount_sol: payload.amount_sol,
        amount_lamports,
        message: format!(
            "Successfully airdropped {} SOL to account",
            payload.amount_sol
        ),
    }))
}
