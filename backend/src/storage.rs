use crate::models::{AuditEvent, EmailPresetRecord, MenuInfo, UserAuthStateRecord};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Postgres, Row, Sqlite};
use std::collections::HashMap;
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistedMenuState {
    pub authority_button_ids: Vec<PersistedAuthorityButtonIds>,
    pub authority_menu_ids: HashMap<u32, Vec<u64>>,
    pub menus: Vec<MenuInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistedAuthorityButtonIds {
    pub authority_id: u32,
    pub menu_id: u64,
    pub selected: Vec<u64>,
}

impl Persistence {
    pub async fn new() -> Self {
        let url = std::env::var("POP_TAIL_DATABASE_URL")
            .or_else(|_| std::env::var("POP_TAIL_SQLITE_URL"))
            .unwrap_or_else(|_| "sqlite://pop_tail_auth.db".into());

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

    pub async fn upsert_user_auth_version(&self, user_id: u64, token_version: u64) {
        match &self.backend {
            Some(DatabaseBackend::Sqlite(pool)) => {
                let result = sqlx::query(
                    r#"
                    INSERT INTO user_auth_state (
                      user_id, token_version, updated_at
                    ) VALUES (?, ?, strftime('%s','now'))
                    ON CONFLICT(user_id) DO UPDATE SET
                      token_version=excluded.token_version,
                      updated_at=strftime('%s','now')
                    "#,
                )
                .bind(user_id as i64)
                .bind(token_version as i64)
                .execute(pool)
                .await;

                if let Err(err) = result {
                    error!("failed to persist sqlite user auth state: {err}");
                }
            }
            Some(DatabaseBackend::Postgres(pool)) => {
                let result = sqlx::query(
                    r#"
                    INSERT INTO user_auth_state (
                      user_id, token_version, updated_at
                    ) VALUES ($1, $2, EXTRACT(EPOCH FROM NOW())::BIGINT)
                    ON CONFLICT(user_id) DO UPDATE SET
                      token_version=EXCLUDED.token_version,
                      updated_at=EXTRACT(EPOCH FROM NOW())::BIGINT
                    "#,
                )
                .bind(user_id as i64)
                .bind(token_version as i64)
                .execute(pool)
                .await;

                if let Err(err) = result {
                    error!("failed to persist postgres user auth state: {err}");
                }
            }
            None => {}
        }
    }

    pub async fn load_user_auth_versions(&self) -> HashMap<u64, u64> {
        let states = match &self.backend {
            Some(DatabaseBackend::Sqlite(pool)) => load_sqlite_user_auth_states(pool).await,
            Some(DatabaseBackend::Postgres(pool)) => load_postgres_user_auth_states(pool).await,
            None => Vec::new(),
        };
        states
            .into_iter()
            .map(|state| (state.user_id, state.token_version.max(1)))
            .collect()
    }

    pub async fn load_audits(&self) -> Vec<AuditEvent> {
        match &self.backend {
            Some(DatabaseBackend::Sqlite(pool)) => load_sqlite_audits(pool).await,
            Some(DatabaseBackend::Postgres(pool)) => load_postgres_audits(pool).await,
            None => Vec::new(),
        }
    }

    pub async fn load_menu_state(&self) -> Option<PersistedMenuState> {
        match &self.backend {
            Some(DatabaseBackend::Sqlite(pool)) => load_sqlite_menu_state(pool).await,
            Some(DatabaseBackend::Postgres(pool)) => load_postgres_menu_state(pool).await,
            None => None,
        }
    }

    pub async fn save_menu_state(&self, state: &PersistedMenuState) {
        let payload = match serde_json::to_string(state) {
            Ok(payload) => payload,
            Err(err) => {
                error!("failed to serialize menu state: {err}");
                return;
            }
        };

        match &self.backend {
            Some(DatabaseBackend::Sqlite(pool)) => {
                let result = sqlx::query(
                    r#"
                    INSERT INTO app_state (
                      state_key, payload, updated_at
                    ) VALUES (?, ?, strftime('%s','now'))
                    ON CONFLICT(state_key) DO UPDATE SET
                      payload=excluded.payload,
                      updated_at=strftime('%s','now')
                    "#,
                )
                .bind("menu_state")
                .bind(payload)
                .execute(pool)
                .await;

                if let Err(err) = result {
                    error!("failed to persist sqlite menu state: {err}");
                }
            }
            Some(DatabaseBackend::Postgres(pool)) => {
                let result = sqlx::query(
                    r#"
                    INSERT INTO app_state (
                      state_key, payload, updated_at
                    ) VALUES ($1, $2, EXTRACT(EPOCH FROM NOW())::BIGINT)
                    ON CONFLICT(state_key) DO UPDATE SET
                      payload=EXCLUDED.payload,
                      updated_at=EXTRACT(EPOCH FROM NOW())::BIGINT
                    "#,
                )
                .bind("menu_state")
                .bind(payload)
                .execute(pool)
                .await;

                if let Err(err) = result {
                    error!("failed to persist postgres menu state: {err}");
                }
            }
            None => {}
        }
    }

    pub async fn load_email_presets(&self) -> Vec<EmailPresetRecord> {
        match &self.backend {
            Some(DatabaseBackend::Sqlite(pool)) => load_sqlite_email_presets(pool).await,
            Some(DatabaseBackend::Postgres(pool)) => load_postgres_email_presets(pool).await,
            None => Vec::new(),
        }
    }

    pub async fn upsert_email_preset(&self, preset: &EmailPresetRecord) {
        match &self.backend {
            Some(DatabaseBackend::Sqlite(pool)) => {
                let result = sqlx::query(
                    r#"
                    INSERT INTO email_presets (
                      id, name, description, recipient, subject, body, created_at, updated_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                    ON CONFLICT(id) DO UPDATE SET
                      name=excluded.name,
                      description=excluded.description,
                      recipient=excluded.recipient,
                      subject=excluded.subject,
                      body=excluded.body,
                      updated_at=excluded.updated_at
                    "#,
                )
                .bind(preset.id as i64)
                .bind(&preset.name)
                .bind(&preset.description)
                .bind(&preset.to)
                .bind(&preset.subject)
                .bind(&preset.body)
                .bind(preset.created_at)
                .bind(preset.updated_at)
                .execute(pool)
                .await;

                if let Err(err) = result {
                    error!("failed to persist sqlite email preset: {err}");
                }
            }
            Some(DatabaseBackend::Postgres(pool)) => {
                let result = sqlx::query(
                    r#"
                    INSERT INTO email_presets (
                      id, name, description, recipient, subject, body, created_at, updated_at
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                    ON CONFLICT(id) DO UPDATE SET
                      name=EXCLUDED.name,
                      description=EXCLUDED.description,
                      recipient=EXCLUDED.recipient,
                      subject=EXCLUDED.subject,
                      body=EXCLUDED.body,
                      updated_at=EXCLUDED.updated_at
                    "#,
                )
                .bind(preset.id as i64)
                .bind(&preset.name)
                .bind(&preset.description)
                .bind(&preset.to)
                .bind(&preset.subject)
                .bind(&preset.body)
                .bind(preset.created_at)
                .bind(preset.updated_at)
                .execute(pool)
                .await;

                if let Err(err) = result {
                    error!("failed to persist postgres email preset: {err}");
                }
            }
            None => {}
        }
    }

    pub async fn delete_email_preset(&self, id: u64) {
        match &self.backend {
            Some(DatabaseBackend::Sqlite(pool)) => {
                let result = sqlx::query("DELETE FROM email_presets WHERE id = ?")
                    .bind(id as i64)
                    .execute(pool)
                    .await;
                if let Err(err) = result {
                    error!("failed to delete sqlite email preset: {err}");
                }
            }
            Some(DatabaseBackend::Postgres(pool)) => {
                let result = sqlx::query("DELETE FROM email_presets WHERE id = $1")
                    .bind(id as i64)
                    .execute(pool)
                    .await;
                if let Err(err) = result {
                    error!("failed to delete postgres email preset: {err}");
                }
            }
            None => {}
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

async fn load_sqlite_user_auth_states(pool: &Pool<Sqlite>) -> Vec<UserAuthStateRecord> {
    match sqlx::query(
        r#"
        SELECT user_id, token_version
        FROM user_auth_state
        "#,
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows
            .into_iter()
            .map(|row| UserAuthStateRecord {
                user_id: row.get::<i64, _>("user_id") as u64,
                token_version: row.get::<i64, _>("token_version") as u64,
            })
            .collect(),
        Err(err) => {
            error!("failed to load sqlite user auth state: {err}");
            Vec::new()
        }
    }
}

async fn load_sqlite_menu_state(pool: &Pool<Sqlite>) -> Option<PersistedMenuState> {
    match sqlx::query(
        r#"
        SELECT payload
        FROM app_state
        WHERE state_key = ?
        "#,
    )
    .bind("menu_state")
    .fetch_optional(pool)
    .await
    {
        Ok(Some(row)) => {
            let payload = row.get::<String, _>("payload");
            match serde_json::from_str::<PersistedMenuState>(&payload) {
                Ok(state) => Some(state),
                Err(err) => {
                    error!("failed to decode sqlite menu state: {err}");
                    None
                }
            }
        }
        Ok(None) => None,
        Err(err) => {
            error!("failed to load sqlite menu state: {err}");
            None
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

async fn load_postgres_user_auth_states(pool: &Pool<Postgres>) -> Vec<UserAuthStateRecord> {
    match sqlx::query(
        r#"
        SELECT user_id, token_version
        FROM user_auth_state
        "#,
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows
            .into_iter()
            .map(|row| UserAuthStateRecord {
                user_id: row.get::<i64, _>("user_id") as u64,
                token_version: row.get::<i64, _>("token_version") as u64,
            })
            .collect(),
        Err(err) => {
            error!("failed to load postgres user auth state: {err}");
            Vec::new()
        }
    }
}

async fn load_postgres_menu_state(pool: &Pool<Postgres>) -> Option<PersistedMenuState> {
    match sqlx::query(
        r#"
        SELECT payload
        FROM app_state
        WHERE state_key = $1
        "#,
    )
    .bind("menu_state")
    .fetch_optional(pool)
    .await
    {
        Ok(Some(row)) => {
            let payload = row.get::<String, _>("payload");
            match serde_json::from_str::<PersistedMenuState>(&payload) {
                Ok(state) => Some(state),
                Err(err) => {
                    error!("failed to decode postgres menu state: {err}");
                    None
                }
            }
        }
        Ok(None) => None,
        Err(err) => {
            error!("failed to load postgres menu state: {err}");
            None
        }
    }
}

async fn load_sqlite_email_presets(pool: &Pool<Sqlite>) -> Vec<EmailPresetRecord> {
    match sqlx::query(
        r#"
        SELECT id, name, description, recipient, subject, body, created_at, updated_at
        FROM email_presets
        ORDER BY updated_at DESC, id DESC
        "#,
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows
            .into_iter()
            .map(|row| EmailPresetRecord {
                id: row.get::<i64, _>("id") as u64,
                name: row.get::<String, _>("name"),
                description: row.get::<String, _>("description"),
                to: row.get::<String, _>("recipient"),
                subject: row.get::<String, _>("subject"),
                body: row.get::<String, _>("body"),
                created_at: row.get::<i64, _>("created_at"),
                updated_at: row.get::<i64, _>("updated_at"),
            })
            .collect(),
        Err(err) => {
            error!("failed to load sqlite email presets: {err}");
            Vec::new()
        }
    }
}

async fn load_postgres_email_presets(pool: &Pool<Postgres>) -> Vec<EmailPresetRecord> {
    match sqlx::query(
        r#"
        SELECT id, name, description, recipient, subject, body, created_at, updated_at
        FROM email_presets
        ORDER BY updated_at DESC, id DESC
        "#,
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows
            .into_iter()
            .map(|row| EmailPresetRecord {
                id: row.get::<i64, _>("id") as u64,
                name: row.get::<String, _>("name"),
                description: row.get::<String, _>("description"),
                to: row.get::<String, _>("recipient"),
                subject: row.get::<String, _>("subject"),
                body: row.get::<String, _>("body"),
                created_at: row.get::<i64, _>("created_at"),
                updated_at: row.get::<i64, _>("updated_at"),
            })
            .collect(),
        Err(err) => {
            error!("failed to load postgres email presets: {err}");
            Vec::new()
        }
    }
}

async fn initialize_sqlite(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_auth_state (
          user_id INTEGER PRIMARY KEY,
          token_version INTEGER NOT NULL,
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

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS app_state (
          state_key TEXT PRIMARY KEY,
          payload TEXT NOT NULL,
          updated_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS email_presets (
          id INTEGER PRIMARY KEY,
          name TEXT NOT NULL,
          description TEXT NOT NULL,
          recipient TEXT NOT NULL,
          subject TEXT NOT NULL,
          body TEXT NOT NULL,
          created_at INTEGER NOT NULL,
          updated_at INTEGER NOT NULL
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
        CREATE TABLE IF NOT EXISTS user_auth_state (
          user_id BIGINT PRIMARY KEY,
          token_version BIGINT NOT NULL,
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

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS app_state (
          state_key TEXT PRIMARY KEY,
          payload TEXT NOT NULL,
          updated_at BIGINT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS email_presets (
          id BIGINT PRIMARY KEY,
          name TEXT NOT NULL,
          description TEXT NOT NULL,
          recipient TEXT NOT NULL,
          subject TEXT NOT NULL,
          body TEXT NOT NULL,
          created_at BIGINT NOT NULL,
          updated_at BIGINT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
