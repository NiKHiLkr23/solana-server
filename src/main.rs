use axum::routing::get;
use axum::{
    extract::rejection::JsonRejection,
    http::{Method, StatusCode},
    response::Json,
    Router,
};
use dotenv::dotenv;
use serde_json::json;
use std::{env, net::SocketAddr};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod modules;
mod utils;

async fn handle_json_rejection(err: JsonRejection) -> (StatusCode, Json<serde_json::Value>) {
    let message = match err {
        JsonRejection::JsonDataError(_) => "Missing required fields",
        JsonRejection::JsonSyntaxError(_) => "Invalid JSON format",
        JsonRejection::MissingJsonContentType(_) => "Missing Content-Type: application/json header",
        _ => "Missing required fields",
    };

    info!("Response: 400 - {}", message);

    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "success": false,
            "error": message
        })),
    )
}

// Fallback handler for unmatched routes
async fn handle_404() -> (StatusCode, Json<serde_json::Value>) {
    info!("Response: 400 - Endpoint not found");

    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "success": false,
            "error": "Endpoint not found"
        })),
    )
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let environment = env::var("ENV").unwrap_or_else(|_| "LOCAL".to_string());

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_ansi(environment == "LOCAL")
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(|| async { "Solana Server is Healthy!" }))
        .merge(modules::keypair::routes())
        .merge(modules::token::routes())
        .merge(modules::message::routes())
        .merge(modules::send::routes())
        .merge(modules::account::routes())
        .merge(modules::airdrop::routes())
        .merge(modules::transfer::routes())
        .fallback(handle_404)
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let socket_address: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();

    info!("Solana Server running on {}", socket_address);

    axum::Server::bind(&socket_address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
