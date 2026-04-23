use axum::Router;
use axum::routing::{get, post};

use crate::state::AppState;

use crate::core::http::health;

use super::{login, logout, refresh, switch_authority, update_profile, user_info, user_upsert};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/healthz", get(health))
        .route("/base/login", post(login))
        .route("/base/refresh", post(refresh))
        .route("/base/logout", post(logout))
        .route("/user/getUserInfo", get(user_info))
        .route("/user/updateProfile", post(update_profile))
        .route("/user/saveUser", post(user_upsert))
        .route("/user/setUserAuthority", post(switch_authority))
        .route("/user/switchAuthority", post(switch_authority))
}
