use axum::body::Body;
use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, HeaderValue, Request, StatusCode};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::{Json, response::Response};

use crate::auth::extract_cookie_value;
use crate::models::{ApiResponse, fail};
use crate::state::AppState;

pub(crate) const ACCESS_TOKEN_COOKIE: &str = "gaa_access_token";
pub(crate) const REFRESH_TOKEN_COOKIE: &str = "gaa_refresh_token";

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

fn read_bool_env(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .map(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(default)
}

fn read_cookie_same_site() -> String {
    let value = std::env::var("POP_TAIL_COOKIE_SAMESITE").unwrap_or_else(|_| "Lax".to_string());
    match value.to_ascii_lowercase().as_str() {
        "strict" => "Strict".to_string(),
        "none" => "None".to_string(),
        _ => "Lax".to_string(),
    }
}

fn read_cookie_domain() -> Option<String> {
    std::env::var("POP_TAIL_COOKIE_DOMAIN")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn build_auth_cookie(name: &str, value: &str, max_age_sec: i64) -> String {
    let mut parts = vec![
        format!("{}={}", name, value),
        "Path=/".to_string(),
        "HttpOnly".to_string(),
        format!("Max-Age={}", max_age_sec.max(0)),
        format!("SameSite={}", read_cookie_same_site()),
    ];
    if read_bool_env("POP_TAIL_COOKIE_SECURE", is_production_env()) {
        parts.push("Secure".to_string());
    }
    if let Some(domain) = read_cookie_domain() {
        parts.push(format!("Domain={}", domain));
    }
    parts.join("; ")
}

fn append_cookie(headers: &mut HeaderMap, cookie: String) {
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        headers.append(SET_COOKIE, value);
    }
}

pub fn append_access_cookie(headers: &mut HeaderMap, token: &str, max_age_sec: i64) {
    append_cookie(
        headers,
        build_auth_cookie(ACCESS_TOKEN_COOKIE, token, max_age_sec),
    );
}

pub fn append_refresh_cookie(headers: &mut HeaderMap, token: &str, max_age_sec: i64) {
    append_cookie(
        headers,
        build_auth_cookie(REFRESH_TOKEN_COOKIE, token, max_age_sec),
    );
}

pub fn append_clear_auth_cookies(headers: &mut HeaderMap) {
    append_cookie(
        headers,
        build_auth_cookie(ACCESS_TOKEN_COOKIE, "deleted", 0),
    );
    append_cookie(
        headers,
        build_auth_cookie(REFRESH_TOKEN_COOKIE, "deleted", 0),
    );
}

pub fn resolve_access_token(headers: &HeaderMap) -> Option<String> {
    let header_token = crate::auth::extract_token(
        headers.get("x-token").and_then(|v| v.to_str().ok()),
        headers.get("authorization").and_then(|v| v.to_str().ok()),
    );
    header_token.or_else(|| {
        if read_bool_env("POP_TAIL_ALLOW_COOKIE_AUTH", false) {
            extract_cookie_value(
                headers.get("cookie").and_then(|v| v.to_str().ok()),
                ACCESS_TOKEN_COOKIE,
            )
        } else {
            None
        }
    })
}

pub async fn enforce_route_access(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let path = request.uri().path().to_string();
    let method = request.method().as_str().to_string();

    if is_public_route(&path) || method == "OPTIONS" {
        return Ok(next.run(request).await);
    }

    let token = resolve_access_token(request.headers())
        .ok_or_else(|| auth_error(StatusCode::UNAUTHORIZED, "未登录或非法访问，请登录"))?;
    let auth = state
        .authorize_human(&token)
        .await
        .map_err(|err| auth_error(StatusCode::UNAUTHORIZED, &err))?;

    if !state
        .is_allowed(auth.user.authority_id, &path, &method)
        .await
    {
        return Err(auth_error(StatusCode::FORBIDDEN, "权限不足"));
    }

    Ok(next.run(request).await)
}

fn is_public_route(path: &str) -> bool {
    matches!(
        path,
        "/healthz" | "/base/login" | "/base/refresh" | "/base/logout"
    ) || path.starts_with("/public/")
        || path == "/ai/moderation/decision"
}

fn auth_error(status: StatusCode, message: &str) -> Response {
    (status, Json(fail(serde_json::json!({}), message))).into_response()
}

pub async fn require_auth(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::state::Authenticated, (StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let token = resolve_access_token(headers).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(fail(serde_json::json!({}), "未登录或非法访问，请登录")),
    ))?;

    state.authenticate_human(&token).await.map_err(|err| {
        (
            StatusCode::UNAUTHORIZED,
            Json(fail(serde_json::json!({}), &err)),
        )
    })
}

pub fn flatten_menus(items: &[crate::models::MenuInfo]) -> Vec<crate::models::MenuInfo> {
    let mut out = Vec::new();
    for item in items {
        out.push(item.clone());
        out.extend(flatten_menus(&item.children));
    }
    out
}

pub fn sanitize_upload_name(name: &str) -> String {
    let sanitized = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if sanitized.is_empty() {
        "file.bin".to_string()
    } else {
        sanitized
    }
}

fn is_production_env() -> bool {
    std::env::var("POP_TAIL_ENV")
        .or_else(|_| std::env::var("RUST_ENV"))
        .or_else(|_| std::env::var("APP_ENV"))
        .map(|value| matches!(value.to_ascii_lowercase().as_str(), "prod" | "production"))
        .unwrap_or(false)
}
