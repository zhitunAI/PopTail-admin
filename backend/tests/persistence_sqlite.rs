use pop_tail_auth::models::{AuditEvent, MenuInfo, MenuMeta};
use pop_tail_auth::storage::{PersistedAuthorityButtonIds, PersistedMenuState, Persistence};
use sqlx::Row;
use sqlx::sqlite::SqlitePoolOptions;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
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

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[tokio::test]
async fn sqlite_persistence_writes_auth_versions_and_audits() {
    let _lock = env_lock().lock().expect("env lock poisoned");
    let db_path = std::env::temp_dir().join(format!("pop-tail-auth-{}.db", Uuid::new_v4()));
    let sqlite_url = format!("sqlite://{}", db_path.display());
    let _guard = EnvGuard::set("POP_TAIL_SQLITE_URL", &sqlite_url);

    let persistence = Persistence::new().await;
    assert!(
        persistence.is_enabled(),
        "sqlite persistence should be enabled"
    );

    persistence.upsert_user_auth_version(1, 7).await;

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

    let auth_row =
        sqlx::query("SELECT user_id, token_version FROM user_auth_state WHERE user_id = ?")
            .bind(1_i64)
            .fetch_one(&pool)
            .await
            .expect("auth state row missing");
    assert_eq!(auth_row.get::<i64, _>("user_id"), 1);
    assert_eq!(auth_row.get::<i64, _>("token_version"), 7);

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

#[tokio::test]
async fn sqlite_persistence_round_trips_menu_state() {
    let _lock = env_lock().lock().expect("env lock poisoned");
    let db_path = std::env::temp_dir().join(format!("pop-tail-menu-{}.db", Uuid::new_v4()));
    let sqlite_url = format!("sqlite://{}", db_path.display());
    let _guard = EnvGuard::set("POP_TAIL_SQLITE_URL", &sqlite_url);

    let persistence = Persistence::new().await;
    assert!(persistence.is_enabled(), "sqlite persistence should be enabled");

    let state = PersistedMenuState {
        authority_button_ids: vec![PersistedAuthorityButtonIds {
            authority_id: 888,
            menu_id: 23,
            selected: vec![2301, 2302],
        }],
        authority_menu_ids: HashMap::from([(888_u32, vec![1_u64, 23_u64])]),
        menus: vec![MenuInfo {
            id: 23,
            parent_id: 2,
            name: "menus".to_string(),
            path: "/system/menus".to_string(),
            component: "views/admin/MenusView.vue".to_string(),
            sort: 23,
            hidden: false,
            meta: MenuMeta {
                title: "菜单管理".to_string(),
                icon: "menu".to_string(),
            },
            menu_btn: vec![],
            parameters: vec![],
            children: vec![],
        }],
    };

    persistence.save_menu_state(&state).await;
    let restored = persistence
        .load_menu_state()
        .await
        .expect("menu state should be restored");

    assert_eq!(restored.menus.len(), 1);
    assert_eq!(restored.menus[0].path, "/system/menus");
    assert_eq!(restored.authority_menu_ids.get(&888), Some(&vec![1, 23]));
    assert_eq!(restored.authority_button_ids.len(), 1);
    assert_eq!(restored.authority_button_ids[0].selected, vec![2301, 2302]);

    let _ = std::fs::remove_file(db_path);
}
