use gaa_auth::models::{AuditEvent, SessionRecord};
use gaa_auth::storage::Persistence;
use sqlx::Row;
use sqlx::sqlite::SqlitePoolOptions;
use uuid::Uuid;

struct EnvGuard {
    key: &'static str,
}

impl EnvGuard {
    fn set(key: &'static str, value: &str) -> Self {
        unsafe {
            std::env::set_var(key, value);
        }
        Self { key }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        unsafe {
            std::env::remove_var(self.key);
        }
    }
}

#[tokio::test]
async fn sqlite_persistence_writes_sessions_and_audits() {
    let db_path = std::env::temp_dir().join(format!("gaa-auth-{}.db", Uuid::new_v4()));
    let sqlite_url = format!("sqlite://{}", db_path.display());
    let _guard = EnvGuard::set("GAA_SQLITE_URL", &sqlite_url);

    let persistence = Persistence::new().await;
    assert!(
        persistence.is_enabled(),
        "sqlite persistence should be enabled"
    );

    let session = SessionRecord {
        session_id: "session-1".to_string(),
        user_id: 1,
        authority_id: 9528,
        access_jti: "access-1".to_string(),
        refresh_jti: "refresh-1".to_string(),
        expires_at: 1_800_000_000,
        refresh_expires_at: 1_900_000_000,
        revoked: true,
    };
    persistence.upsert_session(&session).await;

    let audit = AuditEvent {
        audit_id: "audit-1".to_string(),
        actor: "service:ai-moderation-service".to_string(),
        effective_role: "9528".to_string(),
        target: "article-1".to_string(),
        action: "approve".to_string(),
        reason: "policy-safe".to_string(),
        workflow_run_id: Uuid::new_v4().to_string(),
        timestamp: 1_700_000_000,
    };
    persistence.append_audit(&audit).await;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&sqlite_url)
        .await
        .expect("open sqlite pool failed");

    let session_row = sqlx::query(
        "SELECT authority_id, revoked, access_jti, refresh_jti FROM sessions WHERE session_id = ?",
    )
    .bind(&session.session_id)
    .fetch_one(&pool)
    .await
    .expect("session row missing");
    assert_eq!(session_row.get::<i64, _>("authority_id"), 9528);
    assert_eq!(session_row.get::<i64, _>("revoked"), 1);
    assert_eq!(
        session_row.get::<String, _>("access_jti"),
        session.access_jti
    );
    assert_eq!(
        session_row.get::<String, _>("refresh_jti"),
        session.refresh_jti
    );

    let audit_row =
        sqlx::query("SELECT actor, effective_role, action FROM audits WHERE audit_id = ?")
            .bind(&audit.audit_id)
            .fetch_one(&pool)
            .await
            .expect("audit row missing");
    assert_eq!(audit_row.get::<String, _>("actor"), audit.actor);
    assert_eq!(
        audit_row.get::<String, _>("effective_role"),
        audit.effective_role
    );
    assert_eq!(audit_row.get::<String, _>("action"), audit.action);

    drop(pool);
    let _ = std::fs::remove_file(db_path);
}
