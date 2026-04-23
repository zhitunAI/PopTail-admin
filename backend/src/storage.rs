use crate::models::{AuditEvent, SessionRecord};
use sqlx::postgres::PgPoolOptions;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Postgres, Row, Sqlite};
use std::str::FromStr;
use tracing::{error, warn};

#[derive(Clone)]
pub struct Persistence {
    backend: Option<DatabaseBackend>,
}

#[derive(Clone)]
enum DatabaseBackend {
    Sqlite(Pool<Sqlite>),
    Postgres(Pool<Postgres>),
}

impl Persistence {
    pub async fn new() -> Self {
        let url = std::env::var("GAA_DATABASE_URL")
            .or_else(|_| std::env::var("GAA_SQLITE_URL"))
            .unwrap_or_else(|_| "sqlite://gaa_auth.db".into());

        let backend = if url.starts_with("postgres://") || url.starts_with("postgresql://") {
            connect_postgres(&url).await
        } else {
            connect_sqlite(&url).await
        };

        Self { backend }
    }

    pub fn is_enabled(&self) -> bool {
        self.backend.is_some()
    }

    pub async fn upsert_session(&self, session: &SessionRecord) {
        match &self.backend {
            Some(DatabaseBackend::Sqlite(pool)) => {
                let result = sqlx::query(
                    r#"
                    INSERT INTO sessions (
                      session_id, user_id, authority_id, access_jti, refresh_jti,
                      expires_at, refresh_expires_at, revoked, updated_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, strftime('%s','now'))
                    ON CONFLICT(session_id) DO UPDATE SET
                      user_id=excluded.user_id,
                      authority_id=excluded.authority_id,
                      access_jti=excluded.access_jti,
                      refresh_jti=excluded.refresh_jti,
                      expires_at=excluded.expires_at,
                      refresh_expires_at=excluded.refresh_expires_at,
                      revoked=excluded.revoked,
                      updated_at=strftime('%s','now')
                    "#,
                )
                .bind(&session.session_id)
                .bind(session.user_id as i64)
                .bind(session.authority_id as i64)
                .bind(&session.access_jti)
                .bind(&session.refresh_jti)
                .bind(session.expires_at)
                .bind(session.refresh_expires_at)
                .bind(if session.revoked { 1_i64 } else { 0_i64 })
                .execute(pool)
                .await;

                if let Err(err) = result {
                    error!("failed to persist sqlite session: {err}");
                }
            }
            Some(DatabaseBackend::Postgres(pool)) => {
                let result = sqlx::query(
                    r#"
                    INSERT INTO sessions (
                      session_id, user_id, authority_id, access_jti, refresh_jti,
                      expires_at, refresh_expires_at, revoked, updated_at
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, EXTRACT(EPOCH FROM NOW())::BIGINT)
                    ON CONFLICT(session_id) DO UPDATE SET
                      user_id=EXCLUDED.user_id,
                      authority_id=EXCLUDED.authority_id,
                      access_jti=EXCLUDED.access_jti,
                      refresh_jti=EXCLUDED.refresh_jti,
                      expires_at=EXCLUDED.expires_at,
                      refresh_expires_at=EXCLUDED.refresh_expires_at,
                      revoked=EXCLUDED.revoked,
                      updated_at=EXTRACT(EPOCH FROM NOW())::BIGINT
                    "#,
                )
                .bind(&session.session_id)
                .bind(session.user_id as i64)
                .bind(session.authority_id as i32)
                .bind(&session.access_jti)
                .bind(&session.refresh_jti)
                .bind(session.expires_at)
                .bind(session.refresh_expires_at)
                .bind(session.revoked)
                .execute(pool)
                .await;

                if let Err(err) = result {
                    error!("failed to persist postgres session: {err}");
                }
            }
            None => {}
        }
    }

    pub async fn append_audit(&self, event: &AuditEvent) {
        match &self.backend {
            Some(DatabaseBackend::Sqlite(pool)) => {
                let result = sqlx::query(
                    r#"
                    INSERT INTO audits (
                      audit_id, actor, effective_role, target, action,
                      reason, workflow_run_id, timestamp
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                    "#,
                )
                .bind(&event.audit_id)
                .bind(&event.actor)
                .bind(&event.effective_role)
                .bind(&event.target)
                .bind(&event.action)
                .bind(&event.reason)
                .bind(&event.workflow_run_id)
                .bind(event.timestamp)
                .execute(pool)
                .await;

                if let Err(err) = result {
                    error!("failed to persist sqlite audit: {err}");
                }
            }
            Some(DatabaseBackend::Postgres(pool)) => {
                let result = sqlx::query(
                    r#"
                    INSERT INTO audits (
                      audit_id, actor, effective_role, target, action,
                      reason, workflow_run_id, timestamp
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                    "#,
                )
                .bind(&event.audit_id)
                .bind(&event.actor)
                .bind(&event.effective_role)
                .bind(&event.target)
                .bind(&event.action)
                .bind(&event.reason)
                .bind(&event.workflow_run_id)
                .bind(event.timestamp)
                .execute(pool)
                .await;

                if let Err(err) = result {
                    error!("failed to persist postgres audit: {err}");
                }
            }
            None => {}
        }
    }

    pub async fn load_sessions(&self) -> Vec<SessionRecord> {
        match &self.backend {
            Some(DatabaseBackend::Sqlite(pool)) => load_sqlite_sessions(pool).await,
            Some(DatabaseBackend::Postgres(pool)) => load_postgres_sessions(pool).await,
            None => Vec::new(),
        }
    }

    pub async fn load_audits(&self) -> Vec<AuditEvent> {
        match &self.backend {
            Some(DatabaseBackend::Sqlite(pool)) => load_sqlite_audits(pool).await,
            Some(DatabaseBackend::Postgres(pool)) => load_postgres_audits(pool).await,
            None => Vec::new(),
        }
    }
}

async fn connect_sqlite(url: &str) -> Option<DatabaseBackend> {
    match SqliteConnectOptions::from_str(url) {
        Ok(options) => match SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options.create_if_missing(true))
            .await
        {
            Ok(pool) => {
                if let Err(err) = initialize_sqlite(&pool).await {
                    warn!("sqlite init failed, fallback to in-memory-only persistence: {err}");
                    None
                } else {
                    Some(DatabaseBackend::Sqlite(pool))
                }
            }
            Err(err) => {
                warn!("sqlite open failed, fallback to in-memory-only persistence: {err}");
                None
            }
        },
        Err(err) => {
            warn!("sqlite url parse failed, fallback to in-memory-only persistence: {err}");
            None
        }
    }
}

async fn connect_postgres(url: &str) -> Option<DatabaseBackend> {
    match PgPoolOptions::new().max_connections(5).connect(url).await {
        Ok(pool) => {
            if let Err(err) = initialize_postgres(&pool).await {
                warn!("postgres init failed, fallback to in-memory-only persistence: {err}");
                None
            } else {
                Some(DatabaseBackend::Postgres(pool))
            }
        }
        Err(err) => {
            warn!("postgres open failed, fallback to in-memory-only persistence: {err}");
            None
        }
    }
}

async fn load_sqlite_sessions(pool: &Pool<Sqlite>) -> Vec<SessionRecord> {
    match sqlx::query(
        r#"
        SELECT
          session_id,
          user_id,
          authority_id,
          access_jti,
          refresh_jti,
          expires_at,
          refresh_expires_at,
          revoked
        FROM sessions
        "#,
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows
            .into_iter()
            .map(|row| SessionRecord {
                session_id: row.get::<String, _>("session_id"),
                user_id: row.get::<i64, _>("user_id") as u64,
                authority_id: row.get::<i64, _>("authority_id") as u32,
                access_jti: row.get::<String, _>("access_jti"),
                refresh_jti: row.get::<String, _>("refresh_jti"),
                expires_at: row.get::<i64, _>("expires_at"),
                refresh_expires_at: row.get::<i64, _>("refresh_expires_at"),
                revoked: row.get::<i64, _>("revoked") != 0,
            })
            .collect(),
        Err(err) => {
            error!("failed to load sqlite sessions: {err}");
            Vec::new()
        }
    }
}

async fn load_postgres_sessions(pool: &Pool<Postgres>) -> Vec<SessionRecord> {
    match sqlx::query(
        r#"
        SELECT
          session_id,
          user_id,
          authority_id,
          access_jti,
          refresh_jti,
          expires_at,
          refresh_expires_at,
          revoked
        FROM sessions
        "#,
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows
            .into_iter()
            .map(|row| SessionRecord {
                session_id: row.get::<String, _>("session_id"),
                user_id: row.get::<i64, _>("user_id") as u64,
                authority_id: row.get::<i32, _>("authority_id") as u32,
                access_jti: row.get::<String, _>("access_jti"),
                refresh_jti: row.get::<String, _>("refresh_jti"),
                expires_at: row.get::<i64, _>("expires_at"),
                refresh_expires_at: row.get::<i64, _>("refresh_expires_at"),
                revoked: row.get::<bool, _>("revoked"),
            })
            .collect(),
        Err(err) => {
            error!("failed to load postgres sessions: {err}");
            Vec::new()
        }
    }
}

async fn load_sqlite_audits(pool: &Pool<Sqlite>) -> Vec<AuditEvent> {
    match sqlx::query(
        r#"
        SELECT
          audit_id,
          actor,
          effective_role,
          target,
          action,
          reason,
          workflow_run_id,
          timestamp
        FROM audits
        ORDER BY id ASC
        "#,
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows
            .into_iter()
            .map(|row| AuditEvent {
                audit_id: row.get::<String, _>("audit_id"),
                actor: row.get::<String, _>("actor"),
                effective_role: row.get::<String, _>("effective_role"),
                target: row.get::<String, _>("target"),
                action: row.get::<String, _>("action"),
                reason: row.get::<String, _>("reason"),
                workflow_run_id: row.get::<String, _>("workflow_run_id"),
                timestamp: row.get::<i64, _>("timestamp"),
            })
            .collect(),
        Err(err) => {
            error!("failed to load sqlite audits: {err}");
            Vec::new()
        }
    }
}

async fn load_postgres_audits(pool: &Pool<Postgres>) -> Vec<AuditEvent> {
    match sqlx::query(
        r#"
        SELECT
          audit_id,
          actor,
          effective_role,
          target,
          action,
          reason,
          workflow_run_id,
          timestamp
        FROM audits
        ORDER BY id ASC
        "#,
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows
            .into_iter()
            .map(|row| AuditEvent {
                audit_id: row.get::<String, _>("audit_id"),
                actor: row.get::<String, _>("actor"),
                effective_role: row.get::<String, _>("effective_role"),
                target: row.get::<String, _>("target"),
                action: row.get::<String, _>("action"),
                reason: row.get::<String, _>("reason"),
                workflow_run_id: row.get::<String, _>("workflow_run_id"),
                timestamp: row.get::<i64, _>("timestamp"),
            })
            .collect(),
        Err(err) => {
            error!("failed to load postgres audits: {err}");
            Vec::new()
        }
    }
}

async fn initialize_sqlite(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
          session_id TEXT PRIMARY KEY,
          user_id INTEGER NOT NULL,
          authority_id INTEGER NOT NULL,
          access_jti TEXT NOT NULL,
          refresh_jti TEXT NOT NULL,
          expires_at INTEGER NOT NULL,
          refresh_expires_at INTEGER NOT NULL,
          revoked INTEGER NOT NULL DEFAULT 0,
          updated_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS audits (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          audit_id TEXT NOT NULL,
          actor TEXT NOT NULL,
          effective_role TEXT NOT NULL,
          target TEXT NOT NULL,
          action TEXT NOT NULL,
          reason TEXT NOT NULL,
          workflow_run_id TEXT NOT NULL,
          timestamp INTEGER NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn initialize_postgres(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
          session_id TEXT PRIMARY KEY,
          user_id BIGINT NOT NULL,
          authority_id INTEGER NOT NULL,
          access_jti TEXT NOT NULL,
          refresh_jti TEXT NOT NULL,
          expires_at BIGINT NOT NULL,
          refresh_expires_at BIGINT NOT NULL,
          revoked BOOLEAN NOT NULL DEFAULT FALSE,
          updated_at BIGINT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS audits (
          id BIGSERIAL PRIMARY KEY,
          audit_id TEXT NOT NULL,
          actor TEXT NOT NULL,
          effective_role TEXT NOT NULL,
          target TEXT NOT NULL,
          action TEXT NOT NULL,
          reason TEXT NOT NULL,
          workflow_run_id TEXT NOT NULL,
          timestamp BIGINT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
