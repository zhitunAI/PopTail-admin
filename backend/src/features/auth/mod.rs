pub mod dto;
pub mod router;
pub mod service;

#[allow(unused_imports)]
use axum::extract::{Json, State};
#[allow(unused_imports)]
use axum::http::{HeaderMap, HeaderValue, StatusCode};
#[allow(unused_imports)]
use axum::response::{IntoResponse, Response};

use crate::auth::extract_cookie_value;
use crate::core::http::{
    REFRESH_TOKEN_COOKIE, append_access_cookie, append_clear_auth_cookies, append_refresh_cookie,
    require_auth, resolve_access_token,
};
use crate::models::*;
use crate::state::AppState;

include!("handlers.rs");
include!("profile_handlers.rs");
