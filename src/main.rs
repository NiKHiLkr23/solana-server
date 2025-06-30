use axum::routing::get;
use axum::{http::Method, response::Json, Router};
use dotenv::dotenv;
use serde_json::json;
use std::env;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod modules;
mod utils;

// Fallback handler for unmatched routes
async fn handle_404() -> (axum::http::StatusCode, Json<serde_json::Value>) {
    info!("Response: 400 - Endpoint not found");

    (
        axum::http::StatusCode::BAD_REQUEST,
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
        .fallback(handle_404)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    let user_agent = request
                        .headers()
                        .get(axum::http::header::USER_AGENT)
                        .and_then(|value| value.to_str().ok())
                        .unwrap_or("unknown");

                    tracing::info_span!(
                        "request",
                        method = %request.method(),
                        uri = %request.uri(),
                        user_agent = %user_agent
                    )
                })
                .on_response(
                    |response: &axum::http::Response<_>,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        info!(status = response.status().as_u16(), latency = ?latency);
                    },
                ),
        )
        .layer(cors);

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}").parse().unwrap();

    info!("Solana Server running on http://localhost:{port}");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
