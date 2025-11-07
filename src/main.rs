mod api;
mod engine;

use std::sync::Arc;

use axum::http::StatusCode;
use axum::{
    Router,
    body::Body,
    extract::State,
    http::{self, Request, Response},
    response::IntoResponse,
    routing::get,
};
use reqwest::Client;
use serde::de;
use tokio::net::TcpListener;

use crate::{
    api::{
        http_proxy::HttpProxy,
        model::{AuthorizationError, CallError, DownstreamError},
        proxy::Proxy,
    },
    engine::rate_limiter::RateLimiter,
};

#[tokio::main]
async fn main() {
    let port = "3000";
    let original_uri = "https://www.google.com";

    println!(
        "Starting proxy server on port {}, forwading to {}",
        port, original_uri
    );

    let engine: RateLimiter = RateLimiter::new(5, 60);
    let client: Client = reqwest::Client::new();
    let http_proxy: Arc<HttpProxy> = Arc::new(api::http_proxy::HttpProxy {
        rate_limiter: engine,
        client,
        original_url: original_uri.to_string(),
    });

    let app_state = AppState {
        original_url: original_uri.to_string(),
        proxy: http_proxy,
    };

    let axum_app = Router::new().fallback(handler).with_state(app_state);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    print!("Listening on port {}", port);
    let _ = axum::serve(listener, axum_app).await.unwrap();
}

#[derive(Clone)]
struct AppState {
    original_url: String,
    proxy: Arc<HttpProxy>,
}

#[axum::debug_handler]
async fn handler(State(app_state): State<AppState>, req: Request<Body>) -> Response<Body> {
    println!("Received request: {:?}", req);
    let res = app_state.proxy.proxy_handler(req).await;

    if let Err(err_res) = res {
        match err_res {
            CallError::Authorization(AuthorizationError::TooManyQueries) => {
                return construct_response(StatusCode::TOO_MANY_REQUESTS, "Too Many Requests");
            }
            CallError::Authorization(AuthorizationError::IpHeaderMissing) => {
                return construct_response(StatusCode::BAD_REQUEST, "IP Header Missing");
            }
            CallError::Downstream(DownstreamError::DownstreamError { response }) => {
                return response;
            }
            _ => {
                return construct_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Proxy Server Error",
                );
            }
        }
    };
    Response::new(Body::from("Proxied response"))
}

fn construct_response(status: StatusCode, body: &str) -> Response<Body> {
    let mut response = Response::new(Body::from(body.to_string()));
    *response.status_mut() = status;
    response
}
