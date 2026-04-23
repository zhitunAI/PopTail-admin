use crate::models::SessionRecord;
use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub access_ttl_sec: i64,
    pub refresh_ttl_sec: i64,
    pub refresh_buffer_sec: i64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: std::env::var("GAA_JWT_SECRET")
                .unwrap_or_else(|_| "gaa-dev-secret-change-me".to_string()),
            access_ttl_sec: read_i64_env("GAA_ACCESS_TTL_SEC", 30 * 60),
            refresh_ttl_sec: read_i64_env("GAA_REFRESH_TTL_SEC", 7 * 24 * 3600),
            refresh_buffer_sec: read_i64_env("GAA_REFRESH_BUFFER_SEC", 5 * 60),
        }
    }
}

impl AuthConfig {
    pub fn validate(&self) -> Result<(), String> {
        if !is_production_env() {
            return Ok(());
        }

        let secret = self.jwt_secret.trim();
        let insecure_defaults = [
            "gaa-dev-secret-change-me",
            "gaa-docker-secret-change-me",
            "vue-rust-admin-dev-secret",
        ];
        if secret.len() < 32 || insecure_defaults.iter().any(|value| value == &secret) {
            return Err(
                "GAA_JWT_SECRET must be set to a production-grade secret of at least 32 characters"
                    .to_string(),
            );
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: u64,
    pub sid: String,
    pub jti: String,
    pub aid: u32,
    pub kind: String,
    pub exp: i64,
    pub iat: i64,
}

pub fn now_ts() -> i64 {
    Utc::now().timestamp()
}

pub fn new_id() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub fn create_access_token(
    cfg: &AuthConfig,
    user_id: u64,
    authority_id: u32,
    sid: &str,
) -> Result<(String, String, i64), String> {
    let iat = now_ts();
    let exp = iat + cfg.access_ttl_sec;
    let jti = new_id();
    let claims = Claims {
        sub: user_id,
        sid: sid.to_string(),
        jti: jti.clone(),
        aid: authority_id,
        kind: "human_access".to_string(),
        exp,
        iat,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(cfg.jwt_secret.as_bytes()),
    )
    .map_err(|err| err.to_string())?;
    Ok((token, jti, exp))
}

pub fn create_refresh_token(
    cfg: &AuthConfig,
    user_id: u64,
    authority_id: u32,
    sid: &str,
) -> Result<(String, String, i64), String> {
    let iat = now_ts();
    let exp = iat + cfg.refresh_ttl_sec;
    let jti = new_id();
    let claims = Claims {
        sub: user_id,
        sid: sid.to_string(),
        jti: jti.clone(),
        aid: authority_id,
        kind: "human_refresh".to_string(),
        exp,
        iat,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(cfg.jwt_secret.as_bytes()),
    )
    .map_err(|err| err.to_string())?;
    Ok((token, jti, exp))
}

pub fn decode_token(cfg: &AuthConfig, token: &str) -> Result<Claims, String> {
    let validation = Validation::default();
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(cfg.jwt_secret.as_bytes()),
        &validation,
    )
    .map(|v| v.claims)
    .map_err(|err| err.to_string())
}

pub fn needs_rolling_refresh(cfg: &AuthConfig, session: &SessionRecord) -> bool {
    (session.expires_at - now_ts()) <= cfg.refresh_buffer_sec
}

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|err| err.to_string())
}

pub fn verify_password(password: &str, encoded_hash: &str) -> bool {
    PasswordHash::new(encoded_hash)
        .ok()
        .and_then(|hash| {
            Argon2::default()
                .verify_password(password.as_bytes(), &hash)
                .ok()
        })
        .is_some()
}

pub fn extract_token(value: Option<&str>, auth_header: Option<&str>) -> Option<String> {
    if let Some(x_token) = value {
        let t = x_token.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    if let Some(auth) = auth_header {
        let trimmed = auth.trim();
        if let Some(raw) = trimmed.strip_prefix("Bearer ") {
            let candidate = raw.trim();
            if !candidate.is_empty() {
                return Some(candidate.to_string());
            }
        }
    }
    None
}

pub fn extract_cookie_value(cookie_header: Option<&str>, cookie_name: &str) -> Option<String> {
    let cookie_header = cookie_header?;
    for item in cookie_header.split(';') {
        let mut parts = item.trim().splitn(2, '=');
        let name = parts.next()?.trim();
        let value = parts.next().unwrap_or("").trim();
        if name == cookie_name && !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

pub fn extract_token_with_cookie(
    value: Option<&str>,
    auth_header: Option<&str>,
    cookie_header: Option<&str>,
    cookie_name: &str,
) -> Option<String> {
    extract_token(value, auth_header).or_else(|| extract_cookie_value(cookie_header, cookie_name))
}

fn read_i64_env(key: &str, default: i64) -> i64 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn is_production_env() -> bool {
    std::env::var("GAA_ENV")
        .or_else(|_| std::env::var("RUST_ENV"))
        .or_else(|_| std::env::var("APP_ENV"))
        .map(|value| matches!(value.to_ascii_lowercase().as_str(), "prod" | "production"))
        .unwrap_or(false)
}
