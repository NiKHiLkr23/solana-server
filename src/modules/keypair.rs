use crate::utils::errors::SolanaError;
use axum::{routing::post, Json, Router};
use serde::Serialize;
use solana_sdk::signature::{Keypair, Signer};

#[derive(Serialize)]
pub struct KeypairResponse {
    pub pubkey: String, // Base58 encoded
    pub secret: String, // Base58 encoded
}

pub fn routes() -> Router {
    Router::new().route("/keypair", post(generate_keypair))
}

async fn generate_keypair() -> Result<Json<serde_json::Value>, SolanaError> {
    let keypair = Keypair::new();

    let response = KeypairResponse {
        pubkey: keypair.pubkey().to_string(),
        secret: bs58::encode(&keypair.to_bytes()).into_string(),
    };

    Ok(Json(serde_json::json!({
        "success": true,
        "data": response
    })))
}
