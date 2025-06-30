use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

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
}

impl IntoResponse for SolanaError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            SolanaError::ClientError(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            SolanaError::SdkError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            SolanaError::InvalidInput(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            SolanaError::InsufficientFunds => (StatusCode::BAD_REQUEST, self.to_string()),
            SolanaError::TransactionFailed(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}
