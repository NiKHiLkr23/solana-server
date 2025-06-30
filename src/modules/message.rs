use crate::utils::errors::SolanaError;
use axum::{routing::post, Json, Router};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
};
use tracing::info;

#[derive(Deserialize, Serialize)]
pub struct SignMessageRequest {
    pub message: Option<String>,
    pub secret: Option<String>, // Base58 encoded secret key
}

#[derive(Deserialize, Serialize)]
pub struct VerifyMessageRequest {
    pub message: Option<String>,
    pub signature: Option<String>, // Base64 encoded signature
    pub pubkey: Option<String>,    // Base58 encoded public key
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
    info!(
        "POST /message/sign - Request: {}",
        serde_json::to_string(&payload).unwrap_or_default()
    );

    // Validate required fields are present and not empty
    let message = payload
        .message
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .ok_or(SolanaError::MissingFields)?;

    let secret = payload
        .secret
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .ok_or(SolanaError::MissingFields)?;

    // Decode the base58 secret key
    let secret_bytes = bs58::decode(secret)
        .into_vec()
        .map_err(|_| SolanaError::InvalidInput("Invalid secret key format".to_string()))?;

    // Validate secret key length
    if secret_bytes.len() != 64 {
        return Err(SolanaError::InvalidInput(
            "Invalid secret key length".to_string(),
        ));
    }

    // Create keypair from secret key
    let keypair = Keypair::from_bytes(&secret_bytes)
        .map_err(|_| SolanaError::InvalidInput("Invalid secret key".to_string()))?;

    // Sign the message
    let message_bytes = message.as_bytes();
    let signature = keypair.sign_message(message_bytes);

    info!(
        "Signed message '{}' with pubkey: {}",
        message,
        keypair.pubkey()
    );

    let response = SignMessageResponse {
        signature: general_purpose::STANDARD.encode(signature.as_ref()),
        public_key: keypair.pubkey().to_string(),
        message: message.to_string(),
    };

    let json_response = serde_json::json!({
        "success": true,
        "data": response
    });

    info!("Response: 200 - Message signed successfully");

    Ok(Json(json_response))
}

async fn verify_message(
    Json(payload): Json<VerifyMessageRequest>,
) -> Result<Json<serde_json::Value>, SolanaError> {
    info!(
        "POST /message/verify - Request: {}",
        serde_json::to_string(&payload).unwrap_or_default()
    );

    // Validate required fields are present and not empty
    let message = payload
        .message
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .ok_or(SolanaError::MissingFields)?;

    let signature_str = payload
        .signature
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .ok_or(SolanaError::MissingFields)?;

    let pubkey_str = payload
        .pubkey
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .ok_or(SolanaError::MissingFields)?;

    // Parse the public key
    let pubkey = pubkey_str
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid public key format".to_string()))?;

    // Decode the base64 signature
    let signature_bytes = general_purpose::STANDARD
        .decode(signature_str)
        .map_err(|_| SolanaError::InvalidInput("Invalid signature format".to_string()))?;

    // Validate signature length
    if signature_bytes.len() != 64 {
        return Err(SolanaError::InvalidInput(
            "Invalid signature length".to_string(),
        ));
    }

    // Create signature from bytes
    let signature = Signature::try_from(signature_bytes.as_slice())
        .map_err(|_| SolanaError::InvalidInput("Invalid signature".to_string()))?;

    // Verify the signature using the correct method for message signing
    let message_bytes = message.as_bytes();

    // The correct way to verify a message signature in Solana
    // Use signature.verify with pubkey bytes and message bytes
    let valid = signature.verify(&pubkey.to_bytes(), message_bytes);

    info!(
        "Verification result: {} for message '{}' with pubkey: {}",
        valid, message, pubkey
    );

    let response = VerifyMessageResponse {
        valid,
        message: message.to_string(),
        pubkey: pubkey_str.to_string(),
    };

    let json_response = serde_json::json!({
        "success": true,
        "data": response
    });

    info!("Response: 200 - Message verification completed");

    Ok(Json(json_response))
}
