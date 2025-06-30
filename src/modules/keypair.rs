use crate::utils::errors::SolanaError;
use axum::{routing::post, Json, Router};
use serde::Serialize;
use solana_sdk::signature::{Keypair, Signer};
use tracing::info;

#[derive(Serialize)]
pub struct KeypairResponse {
    pub pubkey: String, // Base58 encoded
    pub secret: String, // Base58 encoded
}

pub fn routes() -> Router {
    Router::new().route("/keypair", post(generate_keypair))
}

async fn generate_keypair() -> Result<Json<serde_json::Value>, SolanaError> {
    info!("POST /keypair");

    let keypair = Keypair::new();

    let response = KeypairResponse {
        pubkey: keypair.pubkey().to_string(),
        secret: bs58::encode(&keypair.to_bytes()).into_string(),
    };

    let json_response = serde_json::json!({
        "success": true,
        "data": response
    });

    info!("Response: 200");

    Ok(Json(json_response))
}
