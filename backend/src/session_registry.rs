use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client};
use tracing::warn;

#[derive(Clone)]
pub struct SessionRegistry {
    connection: MultiplexedConnection,
    key_prefix: String,
}

impl SessionRegistry {
    pub async fn from_env() -> Option<Self> {
        let url = std::env::var("GAA_REDIS_URL").ok()?;
        let key_prefix =
            std::env::var("GAA_REDIS_KEY_PREFIX").unwrap_or_else(|_| "gaa:auth".to_string());

        let client = match Client::open(url.clone()) {
            Ok(client) => client,
            Err(err) => {
                warn!("redis url parse failed, multipoint falls back to local-only mode: {err}");
                return None;
            }
        };

        match client.get_multiplexed_async_connection().await {
            Ok(connection) => Some(Self {
                connection,
                key_prefix,
            }),
            Err(err) => {
                warn!("redis connect failed, multipoint falls back to local-only mode: {err}");
                None
            }
        }
    }

    pub async fn active_session(&self, user_id: u64) -> Option<String> {
        let mut conn = self.connection.clone();
        match conn.get(self.active_key(user_id)).await {
            Ok(value) => value,
            Err(err) => {
                warn!("redis get active session failed: {err}");
                None
            }
        }
    }

    pub async fn mark_active(&self, user_id: u64, session_id: &str, ttl_secs: u64) {
        if ttl_secs == 0 {
            return;
        }

        let mut conn = self.connection.clone();
        let result: redis::RedisResult<()> = conn
            .set_ex(self.active_key(user_id), session_id, ttl_secs)
            .await;
        if let Err(err) = result {
            warn!("redis set active session failed: {err}");
        }
    }

    pub async fn clear_active_if_matches(&self, user_id: u64, session_id: &str) {
        let mut conn = self.connection.clone();
        let key = self.active_key(user_id);
        match conn.get::<_, Option<String>>(&key).await {
            Ok(Some(current)) if current == session_id => {
                let result: redis::RedisResult<usize> = conn.del(&key).await;
                if let Err(err) = result {
                    warn!("redis delete active session failed: {err}");
                }
            }
            Ok(_) => {}
            Err(err) => warn!("redis read active session before delete failed: {err}"),
        }
    }

    pub async fn revoke_session(&self, session_id: &str, ttl_secs: u64) {
        if ttl_secs == 0 {
            return;
        }

        let mut conn = self.connection.clone();
        let result: redis::RedisResult<()> = conn
            .set_ex(self.revoked_key(session_id), "1", ttl_secs)
            .await;
        if let Err(err) = result {
            warn!("redis set revoked session failed: {err}");
        }
    }

    pub async fn is_session_revoked(&self, session_id: &str) -> bool {
        let mut conn = self.connection.clone();
        match conn.exists(self.revoked_key(session_id)).await {
            Ok(value) => value,
            Err(err) => {
                warn!("redis check revoked session failed: {err}");
                false
            }
        }
    }

    fn active_key(&self, user_id: u64) -> String {
        format!("{}:active-user:{user_id}", self.key_prefix)
    }

    fn revoked_key(&self, session_id: &str) -> String {
        format!("{}:revoked-session:{session_id}", self.key_prefix)
    }
}
