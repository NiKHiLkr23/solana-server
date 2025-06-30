use axum::Router;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use std::{env, net::SocketAddr};
use axum::routing::get;
use dotenv::dotenv;

mod modules;
mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let environment = env::var("ENV").unwrap_or_else(|_| "LOCAL".to_string());

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_ansi(environment == "LOCAL")
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let app = Router::new()
        .route("/health", get(|| async { "Solana Server is Healthy!" }))
        .merge(modules::account::routes())
        .merge(modules::airdrop::routes())
        .merge(modules::transfer::routes());

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let socket_address: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    
    info!("Solana Server running on {}", socket_address);
    
    axum::Server::bind(&socket_address)
        .serve(app.into_make_service())
        .await
        .unwrap();
} 