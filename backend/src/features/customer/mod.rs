pub mod dto;
pub mod router;
pub mod service;

#[allow(unused_imports)]
use axum::extract::{Json, Query, State};
#[allow(unused_imports)]
use axum::http::{HeaderMap, StatusCode};

use crate::core::http::require_auth;
use crate::models::*;
use crate::state::AppState;

include!("handlers.rs");
