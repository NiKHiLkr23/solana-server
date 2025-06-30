use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum SolanaError {
    #[error("Client error: {0}")]
    ClientError(#[from] solana_client::client_error::ClientError),

    #[error("SDK error: {0}")]
    SdkError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Insufficient funds")]
    InsufficientFunds,

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Missing required fields")]
    MissingFields,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Token error: {0}")]
    TokenError(String),
}

impl IntoResponse for SolanaError {
    fn into_response(self) -> Response {
        let status = StatusCode::BAD_REQUEST;
        let error_message = self.to_string();

        info!("Response: 400 - {}", error_message);

        let body = Json(json!({
            "success": false,
            "error": error_message
        }));

        (status, body).into_response()
    }
}
