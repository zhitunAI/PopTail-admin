use axum::Router;
use axum::routing::post;

use crate::state::AppState;

use super::post_ai_moderation_decision;

pub fn router() -> Router<AppState> {
    Router::new().route("/ai/moderation/decision", post(post_ai_moderation_decision))
}
