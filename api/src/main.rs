use std::env;

use axum::{Router, routing::get};
use dotenv::dotenv;
use tracing_subscriber;
use crate::routes::{get_user_op, health_check};
use tokio::net::TcpListener;

mod db;
mod models;
mod routes;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let db = db::connect().await.expect("❌ DB connection failed");

    let app = Router::new()
    .route("/user_op/:hash", get(get_user_op))
    .route("/health", get(health_check))
    .with_state(db);

    // 👇 Read from environment variables
    let host = env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("API_PORT").unwrap_or_else(|_| "8081".to_string());
    let addr = format!("{host}:{port}");

    tracing::info!("🔧 Starting API server on {addr}");
    let listener = TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}