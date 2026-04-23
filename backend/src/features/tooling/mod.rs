pub mod dto;
pub mod router;
pub mod service;

#[allow(unused_imports)]
use axum::extract::{Json, Query, State};
#[allow(unused_imports)]
use axum::http::{HeaderMap, StatusCode};
#[allow(unused_imports)]
use axum::response::Response;

use crate::core::http::require_auth;
use crate::models::*;
use crate::state::AppState;

include!("handlers.rs");
include!("advanced_handlers.rs");
