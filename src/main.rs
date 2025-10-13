mod engine;
mod api;

use axum::{Router, http::Request, routing::get};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let port = "3000";
    let app = Router::new().route("/", get(root));
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    let _ = axum::serve(listener, app).await.unwrap();
    println!("Launched")
}

async fn root() -> &'static str {
    "Hello, World!"
}

