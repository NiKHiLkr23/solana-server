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

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Missing required fields")]
    MissingFields,

    #[error("Token error: {0}")]
    TokenError(String),
}

impl IntoResponse for SolanaError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            SolanaError::MissingFields => (StatusCode::BAD_REQUEST, self.to_string()),
            SolanaError::InvalidInput(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            SolanaError::TokenError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            SolanaError::ClientError(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
        };

        info!("Response: {} - {}", status.as_u16(), error_message);

        let body = Json(json!({
            "success": false,
            "error": error_message
        }));

        (status, body).into_response()
    }
}
