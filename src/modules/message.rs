use crate::utils::errors::SolanaError;
use axum::{routing::post, Json, Router};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
};

#[derive(Deserialize)]
pub struct SignMessageRequest {
    pub message: String,
    pub secret: String, // Base58 encoded secret key
}

#[derive(Deserialize)]
pub struct VerifyMessageRequest {
    pub message: String,
    pub signature: String, // Base64 encoded signature
    pub pubkey: String,    // Base58 encoded public key
}

#[derive(Serialize)]
pub struct SignMessageResponse {
    pub signature: String,  // Base64 encoded
    pub public_key: String, // Base58 encoded
    pub message: String,
}

#[derive(Serialize)]
pub struct VerifyMessageResponse {
    pub valid: bool,
    pub message: String,
    pub pubkey: String,
}

pub fn routes() -> Router {
    Router::new()
        .route("/message/sign", post(sign_message))
        .route("/message/verify", post(verify_message))
}

async fn sign_message(
    Json(payload): Json<SignMessageRequest>,
) -> Result<Json<serde_json::Value>, SolanaError> {
    // Validate required fields
    if payload.message.is_empty() || payload.secret.is_empty() {
        return Err(SolanaError::MissingFields);
    }

    // Decode the base58 secret key
    let secret_bytes = bs58::decode(&payload.secret)
        .into_vec()
        .map_err(|_| SolanaError::InvalidInput("Invalid secret key format".to_string()))?;

    // Create keypair from secret key
    let keypair = Keypair::from_bytes(&secret_bytes)
        .map_err(|_| SolanaError::InvalidInput("Invalid secret key".to_string()))?;

    // Sign the message
    let message_bytes = payload.message.as_bytes();
    let signature = keypair.sign_message(message_bytes);

    let response = SignMessageResponse {
        signature: general_purpose::STANDARD.encode(signature.as_ref()),
        public_key: keypair.pubkey().to_string(),
        message: payload.message,
    };

    Ok(Json(serde_json::json!({
        "success": true,
        "data": response
    })))
}

async fn verify_message(
    Json(payload): Json<VerifyMessageRequest>,
) -> Result<Json<serde_json::Value>, SolanaError> {
    // Validate required fields
    if payload.message.is_empty() || payload.signature.is_empty() || payload.pubkey.is_empty() {
        return Err(SolanaError::MissingFields);
    }

    // Parse the public key
    let pubkey = payload
        .pubkey
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid public key format".to_string()))?;

    // Decode the base64 signature
    let signature_bytes = general_purpose::STANDARD
        .decode(&payload.signature)
        .map_err(|_| SolanaError::InvalidInput("Invalid signature format".to_string()))?;

    // Create signature from bytes
    let signature = Signature::try_from(signature_bytes.as_slice())
        .map_err(|_| SolanaError::InvalidInput("Invalid signature".to_string()))?;

    // Verify the signature
    let message_bytes = payload.message.as_bytes();
    let valid = signature.verify(pubkey.as_ref(), message_bytes);

    let response = VerifyMessageResponse {
        valid,
        message: payload.message,
        pubkey: payload.pubkey,
    };

    Ok(Json(serde_json::json!({
        "success": true,
        "data": response
    })))
}
