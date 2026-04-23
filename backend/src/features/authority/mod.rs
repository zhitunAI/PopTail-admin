pub mod dto;
pub mod router;
pub mod service;

#[allow(unused_imports)]
use axum::extract::{Json, Query, State};
#[allow(unused_imports)]
use axum::http::{HeaderMap, HeaderValue, StatusCode};
#[allow(unused_imports)]
use axum::response::{IntoResponse, Response};

use crate::core::http::{append_access_cookie, flatten_menus, require_auth, resolve_access_token};
use crate::models::*;
use crate::state::AppState;

include!("handlers.rs");
