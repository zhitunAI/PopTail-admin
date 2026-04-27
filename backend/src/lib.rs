pub mod auth;
pub mod casbin_port;
pub mod core;
pub mod features;
pub mod models;
pub mod state;
pub mod storage;

use axum::Router;
use axum::http::Method;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderName, HeaderValue};
use axum::middleware;
use state::AppState;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub fn build_router(state: AppState) -> Router {
    let allowed_origins = [
        HeaderValue::from_static("http://127.0.0.1:5666"),
        HeaderValue::from_static("http://localhost:5666"),
        HeaderValue::from_static("http://127.0.0.1:3000"),
        HeaderValue::from_static("http://localhost:3000"),
    ];
    let allowed_headers = [
        AUTHORIZATION,
        CONTENT_TYPE,
        HeaderName::from_static("x-token"),
        HeaderName::from_static("x-csrf-token"),
    ];
    let allowed_methods = [
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::OPTIONS,
    ];

    Router::new()
        .merge(features::auth::router::router())
        .merge(features::authority::router::router())
        .merge(features::system::router::router())
        .merge(features::frontend::router::router())
        .merge(features::tooling::router::router())
        .merge(features::examples::router::router())
        .merge(features::customer::router::router())
        .merge(features::ai::router::router())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            core::http::enforce_route_access,
        ))
        .layer(
            CorsLayer::new()
                .allow_methods(allowed_methods)
                .allow_headers(allowed_headers)
                .allow_origin(allowed_origins)
                .allow_credentials(true),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[cfg(test)]
mod router_tests;
