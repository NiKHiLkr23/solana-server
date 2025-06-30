use crate::utils::errors::SolanaError;
use axum::{routing::post, Json, Router};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};

#[derive(Deserialize)]
pub struct TransferRequest {
    pub from_private_key: String, // Base64 encoded private key
    pub to_public_key: String,
    pub amount_sol: f64,
}

#[derive(Serialize)]
pub struct TransferResponse {
    pub transaction_signature: String,
    pub from_public_key: String,
    pub to_public_key: String,
    pub amount_sol: f64,
    pub amount_lamports: u64,
    pub message: String,
}

pub fn routes() -> Router {
    Router::new().route("/transfer", post(transfer_sol))
}

async fn transfer_sol(
    Json(payload): Json<TransferRequest>,
) -> Result<Json<TransferResponse>, SolanaError> {
    if payload.amount_sol <= 0.0 {
        return Err(SolanaError::InvalidInput(
            "Amount must be greater than 0".to_string(),
        ));
    }

    // Parse recipient public key
    let to_pubkey = payload.to_public_key.parse::<Pubkey>().map_err(|_| {
        SolanaError::InvalidInput("Invalid recipient public key format".to_string())
    })?;

    // Decode private key
    let private_key_bytes = general_purpose::STANDARD
        .decode(&payload.from_private_key)
        .map_err(|_| SolanaError::InvalidInput("Invalid private key format".to_string()))?;

    let from_keypair = Keypair::from_bytes(&private_key_bytes)
        .map_err(|_| SolanaError::InvalidInput("Invalid private key".to_string()))?;

    let client = crate::utils::solana_client::get_rpc_client();
    let amount_lamports = (payload.amount_sol * 1_000_000_000.0) as u64;

    // Check balance
    let balance = client
        .get_balance(&from_keypair.pubkey())
        .map_err(SolanaError::ClientError)?;

    if balance < amount_lamports {
        return Err(SolanaError::InsufficientFunds);
    }

    // Get recent blockhash
    let recent_blockhash = client
        .get_latest_blockhash()
        .map_err(SolanaError::ClientError)?;

    // Create transfer instruction
    let transfer_instruction =
        system_instruction::transfer(&from_keypair.pubkey(), &to_pubkey, amount_lamports);

    // Create and sign transaction
    let transaction = Transaction::new_signed_with_payer(
        &[transfer_instruction],
        Some(&from_keypair.pubkey()),
        &[&from_keypair],
        recent_blockhash,
    );

    // Send transaction
    let signature = client
        .send_and_confirm_transaction(&transaction)
        .map_err(|e| SolanaError::TransactionFailed(e.to_string()))?;

    Ok(Json(TransferResponse {
        transaction_signature: signature.to_string(),
        from_public_key: from_keypair.pubkey().to_string(),
        to_public_key: payload.to_public_key,
        amount_sol: payload.amount_sol,
        amount_lamports,
        message: format!("Successfully transferred {} SOL", payload.amount_sol),
    }))
}
