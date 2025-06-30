use crate::utils::errors::SolanaError;
use axum::{
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};

#[derive(Deserialize)]
pub struct CreateAccountRequest {
    #[serde(default)]
    pub save_private_key: bool,
}

#[derive(Serialize)]
pub struct CreateAccountResponse {
    pub public_key: String,
    pub private_key: Option<String>,
    pub message: String,
}

#[derive(Serialize)]
pub struct AccountInfoResponse {
    pub public_key: String,
    pub balance_sol: f64,
    pub balance_lamports: u64,
    pub executable: bool,
    pub owner: String,
    pub rent_epoch: u64,
}

pub fn routes() -> Router {
    Router::new()
        .route("/account/create", post(create_account))
        .route("/account/:pubkey", get(get_account_info))
}

async fn create_account(
    Json(payload): Json<CreateAccountRequest>,
) -> Result<Json<CreateAccountResponse>, SolanaError> {
    let keypair = Keypair::new();
    let public_key = keypair.pubkey().to_string();

    let private_key = if payload.save_private_key {
        Some(general_purpose::STANDARD.encode(&keypair.to_bytes()))
    } else {
        None
    };

    Ok(Json(CreateAccountResponse {
        public_key,
        private_key,
        message: "Account created successfully. Note: This is a new keypair, it needs to be funded before use.".to_string(),
    }))
}

async fn get_account_info(
    axum::extract::Path(pubkey_str): axum::extract::Path<String>,
) -> Result<Json<AccountInfoResponse>, SolanaError> {
    let pubkey = pubkey_str
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid public key format".to_string()))?;

    let client = crate::utils::solana_client::get_rpc_client();

    let account = client
        .get_account(&pubkey)
        .map_err(SolanaError::ClientError)?;

    let balance_lamports = account.lamports;
    let balance_sol = balance_lamports as f64 / 1_000_000_000.0; // Convert lamports to SOL

    Ok(Json(AccountInfoResponse {
        public_key: pubkey.to_string(),
        balance_sol,
        balance_lamports,
        executable: account.executable,
        owner: account.owner.to_string(),
        rent_epoch: account.rent_epoch,
    }))
}
