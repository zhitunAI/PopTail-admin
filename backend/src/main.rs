use std::net::SocketAddr;

use pop_tail_auth::{build_router, state::AppState};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let state = AppState::seed()
        .await
        .expect("failed to seed application state");
    let app = build_router(state);

    let bind_addr = std::env::var("POP_TAIL_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8888".to_string());
    let addr: SocketAddr = bind_addr.parse().expect("invalid bind address");
    tracing::info!("PopTail auth server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind listener");
    axum::serve(listener, app)
        .await
        .expect("failed to start server");
}
