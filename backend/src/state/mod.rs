use crate::auth::{
    AuthConfig, create_access_token, create_refresh_token, decode_token, hash_password,
    needs_rolling_refresh, now_ts, verify_password,
};
use crate::casbin_port::CasbinPort;
use crate::models::{
    AnnouncementAttachment, AnnouncementDataSourcePayload, AnnouncementRecord, ApiInfo,
    ApiListRequest, ApiTokenRecord, ArticleCategoryRecord, ArticleRecord, AuditEvent,
    AuthorityInfo, AutoCodeRegistryRecord, CasbinInfo, ConsoleMenuRecord, CustomerRecord,
    DictionaryDetailInfo, DictionaryInfo, EmailRecord, ErrorLogInfo, ExportTemplateInfo,
    FrontendNavRecord, FrontendSettingsRecord, FrontendSettingsUpdateRequest,
    GlobalConstraintRecord, LlmConfigRecord, LoginLogInfo, McpInputSchema, McpSchemaProperty,
    McpServiceStatusPayload, McpTestResult, McpToolDescriptor, McpToolRecord, MemberRecord,
    MenuInfo, MenuMeta, OperationLogInfo, PackageRecord, PageResult, ParamInfo,
    PluginInstallRecord, PluginManifestRecord, ReleaseRecord, ResumeUploadRecord, RuntimeInfo,
    ScanSessionRecord, SelectOption, ServiceIdentity, SessionRecord, SkillAssetRecord, SkillRecord,
    SkillSummary, SkillToolDefinition, SystemConfigInfo, UploadFileRecord, UserInfo,
    UserListRequest, UserRecord,
};
use crate::session_registry::SessionRegistry;
use crate::storage::Persistence;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

mod admin;
mod examples;
mod misc;
mod platform;
mod seed;
mod tooling;

use self::seed::*;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub auth: AuthConfig,
    pub compatibility_refresh_headers: bool,
    pub multipoint_enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auth: AuthConfig::default(),
            compatibility_refresh_headers: read_bool_env("GAA_COMPATIBILITY_REFRESH_HEADERS", true),
            multipoint_enabled: read_bool_env("GAA_MULTIPOINT_ENABLED", false),
        }
    }
}

#[derive(Clone, Debug)]
pub struct InnerState {
    pub users_by_name: HashMap<String, UserRecord>,
    pub users_by_id: HashMap<u64, UserRecord>,
    pub sessions: HashMap<String, SessionRecord>,
    pub active_sessions_by_user: HashMap<u64, String>,
    pub blacklisted_access_jti: HashSet<String>,
    pub audits: Vec<AuditEvent>,
    pub service_identities: HashMap<String, ServiceIdentity>,
    pub params: Vec<ParamInfo>,
    pub dictionary_details: HashMap<u64, Vec<DictionaryDetailInfo>>,
    pub authorities: Vec<AuthorityInfo>,
    pub menus: Vec<MenuInfo>,
    pub authority_menu_ids: HashMap<u32, Vec<u64>>,
    pub authority_button_ids: HashMap<(u32, u64), Vec<u64>>,
    pub apis: Vec<ApiInfo>,
    pub dictionaries: Vec<DictionaryInfo>,
    pub system_config: SystemConfigInfo,
    pub api_tokens: Vec<ApiTokenRecord>,
    pub packages: Vec<PackageRecord>,
    pub plugin_manifests: Vec<PluginManifestRecord>,
    pub plugin_installs: Vec<PluginInstallRecord>,
    pub auto_code_registry: Vec<AutoCodeRegistryRecord>,
    pub releases: Vec<ReleaseRecord>,
    pub email_records: Vec<EmailRecord>,
    pub announcements: Vec<AnnouncementRecord>,
    pub llm_configs: Vec<LlmConfigRecord>,
    pub frontend_nav: Vec<FrontendNavRecord>,
    pub frontend_settings: FrontendSettingsRecord,
    pub article_categories: Vec<ArticleCategoryRecord>,
    pub articles: Vec<ArticleRecord>,
    pub members: Vec<MemberRecord>,
    pub console_menus: Vec<ConsoleMenuRecord>,
    pub upload_files: Vec<UploadFileRecord>,
    pub resume_uploads: Vec<ResumeUploadRecord>,
    pub scan_sessions: Vec<ScanSessionRecord>,
    pub customers: Vec<CustomerRecord>,
    pub mcp_tools: Vec<McpToolRecord>,
    pub mcp_service_running: bool,
    pub mcp_service_started_at: Option<i64>,
    pub mcp_service_last_error: String,
    pub skills: Vec<SkillRecord>,
    pub global_constraint: GlobalConstraintRecord,
}

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub casbin: Arc<RwLock<CasbinPort>>,
    pub inner: Arc<RwLock<InnerState>>,
    pub persistence: Persistence,
    pub session_registry: Option<SessionRegistry>,
}

#[derive(Clone, Debug)]
pub struct Authenticated {
    pub user: UserInfo,
    pub session_id: String,
    pub new_token: Option<String>,
    pub new_expires_at: Option<i64>,
}

impl AppState {
    pub async fn seed() -> Result<Self, String> {
        Self::seed_with_config(AppConfig::default()).await
    }

    pub async fn seed_with_config(config: AppConfig) -> Result<Self, String> {
        config.auth.validate()?;
        let authority_tree = seeded_authorities();
        let flat_authorities = flatten_authorities(&authority_tree);
        let admin_authority = flat_authorities
            .iter()
            .find(|item| item.authority_id == 888)
            .cloned()
            .unwrap_or_else(|| authority_tree[0].clone());
        let ai_authority = flat_authorities
            .iter()
            .find(|item| item.authority_id == 9528)
            .cloned()
            .unwrap_or_else(|| authority_tree[0].clone());

        let bootstrap_password = read_bootstrap_admin_password()?;
        let bootstrap_password_hash = hash_password(&bootstrap_password)?;

        let admin = UserRecord {
            info: UserInfo {
                id: 1,
                uuid: Uuid::new_v4(),
                user_name: "admin".to_string(),
                nick_name: "GAA管理员".to_string(),
                authority_id: 888,
                authority: admin_authority.clone(),
                authorities: vec![admin_authority.clone(), ai_authority.clone()],
                header_img: "https://example.com/gaa-admin-avatar.png".to_string(),
                phone: "13800138000".to_string(),
                email: "admin@gaa.local".to_string(),
                enable: 1,
            },
            password: bootstrap_password_hash.clone(),
        };

        let disabled = UserRecord {
            info: UserInfo {
                id: 2,
                uuid: Uuid::new_v4(),
                user_name: "disabled".to_string(),
                nick_name: "禁用账户".to_string(),
                authority_id: 888,
                authority: admin_authority.clone(),
                authorities: vec![admin_authority.clone()],
                header_img: "https://example.com/gaa-disabled-avatar.png".to_string(),
                phone: "13900139000".to_string(),
                email: "disabled@gaa.local".to_string(),
                enable: 2,
            },
            password: bootstrap_password_hash,
        };

        let mut users_by_name = HashMap::new();
        users_by_name.insert(admin.info.user_name.clone(), admin.clone());
        users_by_name.insert(disabled.info.user_name.clone(), disabled.clone());

        let mut users_by_id = HashMap::new();
        users_by_id.insert(admin.info.id, admin);
        users_by_id.insert(disabled.info.id, disabled);

        let mut policies = HashMap::<u32, Vec<CasbinInfo>>::new();
        policies.insert(
            888,
            vec![
                CasbinInfo {
                    path: "/user/getUserInfo".to_string(),
                    method: "GET".to_string(),
                },
                CasbinInfo {
                    path: "/casbin/getPolicyPathByAuthorityId".to_string(),
                    method: "POST".to_string(),
                },
                CasbinInfo {
                    path: "/base/logout".to_string(),
                    method: "POST".to_string(),
                },
                CasbinInfo {
                    path: "/user/setUserAuthority".to_string(),
                    method: "POST".to_string(),
                },
                CasbinInfo {
                    path: "/ai/moderation/decision".to_string(),
                    method: "POST".to_string(),
                },
            ],
        );
        policies.insert(
            9528,
            vec![
                CasbinInfo {
                    path: "/ai/moderation/decision".to_string(),
                    method: "POST".to_string(),
                },
                CasbinInfo {
                    path: "/user/getUserInfo".to_string(),
                    method: "GET".to_string(),
                },
                CasbinInfo {
                    path: "/casbin/getPolicyPathByAuthorityId".to_string(),
                    method: "POST".to_string(),
                },
                CasbinInfo {
                    path: "/user/setUserAuthority".to_string(),
                    method: "POST".to_string(),
                },
            ],
        );

        let casbin = CasbinPort::bootstrap(policies).await?;
        let service_token = read_service_token()?;
        let mut service_identities = HashMap::new();
        service_identities.insert(
            service_token.clone(),
            ServiceIdentity {
                token: service_token,
                name: "ai-moderation-service".to_string(),
                scopes: vec![
                    "article.review".to_string(),
                    "user.violation.handle".to_string(),
                    "crawler.violation.handle".to_string(),
                ],
                ai_role: 9528,
            },
        );
        let params = vec![
            ParamInfo {
                id: 1,
                key: "system.site_name".to_string(),
                value: "GAA Admin".to_string(),
                desc: "站点名称".to_string(),
            },
            ParamInfo {
                id: 2,
                key: "audit.retention_days".to_string(),
                value: "30".to_string(),
                desc: "审计日志保留天数".to_string(),
            },
        ];

        let persistence = Persistence::new().await;
        let restored_sessions = persistence.load_sessions().await;
        let active_sessions_by_user = build_active_sessions_index(&restored_sessions);
        let restored_sessions = restored_sessions
            .into_iter()
            .map(|session| (session.session_id.clone(), session))
            .collect();
        let restored_audits = persistence.load_audits().await;
        let session_registry = SessionRegistry::from_env().await;
        let dictionary_details = seeded_dictionary_details();
        let authorities = authority_tree;
        let menus = seeded_menu_tree();
        let authority_menu_ids = seeded_authority_menu_ids(&menus);
        let authority_button_ids = seeded_authority_button_ids(&menus, &authority_menu_ids);
        let apis = seeded_apis();
        let dictionaries = seeded_dictionaries();
        let system_config = seeded_system_config(
            config.compatibility_refresh_headers,
            config.multipoint_enabled,
        );
        let api_tokens = seeded_api_tokens();
        let packages = seeded_packages();
        let plugin_manifests = seeded_plugin_manifests();
        let plugin_installs = seeded_plugin_installs();
        let auto_code_registry = seeded_auto_code_registry();
        let releases = seeded_releases();
        let email_records = seeded_email_records();
        let announcements = seeded_announcements();
        let llm_configs = seeded_llm_configs();
        let frontend_nav = seeded_frontend_nav();
        let frontend_settings = seeded_frontend_settings();
        let article_categories = seeded_article_categories();
        let articles = seeded_articles();
        let members = seeded_members();
        let console_menus = seeded_console_menus();
        let upload_files = vec![];
        let resume_uploads = seeded_resume_uploads();
        let scan_sessions = vec![];
        let customers = seeded_customers();
        let mcp_tools = seeded_mcp_tools();
        let skills = seeded_skills();
        let global_constraint = seeded_global_constraint();

        Ok(Self {
            config,
            casbin: Arc::new(RwLock::new(casbin)),
            inner: Arc::new(RwLock::new(InnerState {
                users_by_name,
                users_by_id,
                sessions: restored_sessions,
                active_sessions_by_user,
                blacklisted_access_jti: HashSet::new(),
                audits: restored_audits,
                service_identities,
                params,
                dictionary_details,
                authorities,
                menus,
                authority_menu_ids,
                authority_button_ids,
                apis,
                dictionaries,
                system_config,
                api_tokens,
                packages,
                plugin_manifests,
                plugin_installs,
                auto_code_registry,
                releases,
                email_records,
                announcements,
                llm_configs,
                frontend_nav,
                frontend_settings,
                article_categories,
                articles,
                members,
                console_menus,
                upload_files,
                resume_uploads,
                scan_sessions,
                customers,
                mcp_tools,
                mcp_service_running: false,
                mcp_service_started_at: None,
                mcp_service_last_error: String::new(),
                skills,
                global_constraint,
            })),
            persistence,
            session_registry,
        })
    }

    pub async fn login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<(UserInfo, String, i64, String), String> {
        let user = {
            let guard = self.inner.read().await;
            guard.users_by_name.get(username).cloned()
        }
        .ok_or_else(|| "用户名不存在或者密码错误".to_string())?;

        if !verify_password(password, &user.password) {
            return Err("用户名不存在或者密码错误".to_string());
        }
        if user.info.enable != 1 {
            return Err("用户被禁止登录".to_string());
        }

        let previous_distributed_session = if self.config.multipoint_enabled {
            self.active_distributed_session(user.info.id).await
        } else {
            None
        };

        let session_id = Uuid::new_v4().to_string();
        let (access_token, access_jti, expires_at) = create_access_token(
            &self.config.auth,
            user.info.id,
            user.info.authority_id,
            &session_id,
        )?;
        let (refresh_token, refresh_jti, refresh_expires_at) = create_refresh_token(
            &self.config.auth,
            user.info.id,
            user.info.authority_id,
            &session_id,
        )?;

        let session = SessionRecord {
            session_id: session_id.clone(),
            user_id: user.info.id,
            authority_id: user.info.authority_id,
            access_jti,
            refresh_jti,
            expires_at,
            refresh_expires_at,
            revoked: false,
        };
        let persisted_session = session.clone();
        let mut extra_persisted = Vec::new();

        {
            let mut guard = self.inner.write().await;
            if self.config.multipoint_enabled {
                if let Some(previous_session_id) =
                    guard.active_sessions_by_user.get(&user.info.id).cloned()
                {
                    if previous_session_id != session_id {
                        if let Some(previous) = guard.sessions.get_mut(&previous_session_id) {
                            previous.revoked = true;
                            let access_jti = previous.access_jti.clone();
                            let persisted_previous = previous.clone();
                            let _ = previous;
                            guard.blacklisted_access_jti.insert(access_jti);
                            extra_persisted.push(persisted_previous);
                        }
                    }
                }
                guard
                    .active_sessions_by_user
                    .insert(user.info.id, session_id.clone());
            }

            guard.sessions.insert(session_id.clone(), session);
        }

        self.persistence.upsert_session(&persisted_session).await;
        for prior in extra_persisted {
            self.persistence.upsert_session(&prior).await;
        }

        if self.config.multipoint_enabled {
            if let Some(previous_session_id) = previous_distributed_session {
                if previous_session_id != session_id {
                    self.mark_session_revoked_distributed(
                        &previous_session_id,
                        ttl_from(refresh_expires_at),
                    )
                    .await;
                }
            }
            self.mark_session_active_distributed(
                user.info.id,
                &session_id,
                ttl_from(refresh_expires_at),
            )
            .await;
        }

        Ok((user.info, access_token, expires_at * 1000, refresh_token))
    }

    pub async fn refresh(&self, refresh_token: &str) -> Result<(String, i64), String> {
        let claims = decode_token(&self.config.auth, refresh_token)?;
        if claims.kind != "human_refresh" {
            return Err("refresh token类型错误".to_string());
        }

        if self.is_session_revoked_distributed(&claims.sid).await {
            return Err("refresh token失效".to_string());
        }
        if !self.is_active_session_valid(claims.sub, &claims.sid).await {
            return Err("您的帐户异地登录或令牌失效".to_string());
        }

        let mut guard = self.inner.write().await;
        let session = guard
            .sessions
            .get_mut(&claims.sid)
            .ok_or_else(|| "会话不存在".to_string())?;
        if session.revoked || session.refresh_jti != claims.jti || claims.exp <= now_ts() {
            return Err("refresh token失效".to_string());
        }

        let (new_token, new_jti, exp) = create_access_token(
            &self.config.auth,
            session.user_id,
            session.authority_id,
            &session.session_id,
        )?;
        session.access_jti = new_jti;
        session.expires_at = exp;
        let persisted = session.clone();
        drop(guard);

        self.persistence.upsert_session(&persisted).await;
        if self.config.multipoint_enabled {
            self.mark_session_active_distributed(
                persisted.user_id,
                &persisted.session_id,
                ttl_from(persisted.refresh_expires_at),
            )
            .await;
        }
        Ok((new_token, exp))
    }

    pub async fn authenticate_human(&self, access_token: &str) -> Result<Authenticated, String> {
        self.authenticate_human_with_refresh(access_token, true)
            .await
    }

    pub async fn authorize_human(&self, access_token: &str) -> Result<Authenticated, String> {
        self.authenticate_human_with_refresh(access_token, false)
            .await
    }

    async fn authenticate_human_with_refresh(
        &self,
        access_token: &str,
        allow_rolling_refresh: bool,
    ) -> Result<Authenticated, String> {
        let claims = decode_token(&self.config.auth, access_token)?;
        if claims.kind != "human_access" {
            return Err("token类型错误".to_string());
        }
        if self.is_session_revoked_distributed(&claims.sid).await {
            return Err("您的帐户异地登录或令牌失效".to_string());
        }
        if !self.is_active_session_valid(claims.sub, &claims.sid).await {
            return Err("您的帐户异地登录或令牌失效".to_string());
        }

        let (user, maybe_new, persisted) = {
            let mut guard = self.inner.write().await;
            if guard.blacklisted_access_jti.contains(&claims.jti) {
                return Err("令牌已失效".to_string());
            }

            let user = guard
                .users_by_id
                .get(&claims.sub)
                .cloned()
                .ok_or_else(|| "用户不存在".to_string())?;
            if user.info.enable != 1 {
                return Err("用户被禁止登录".to_string());
            }

            let session = guard
                .sessions
                .get_mut(&claims.sid)
                .ok_or_else(|| "会话不存在".to_string())?;
            if session.user_id != claims.sub
                || session.revoked
                || session.access_jti != claims.jti
                || claims.exp <= now_ts()
            {
                return Err("登录已过期，请重新登录".to_string());
            }

            let effective_user = Self::effective_user_info(&user, session.authority_id)?;

            let maybe_new =
                if allow_rolling_refresh && needs_rolling_refresh(&self.config.auth, session) {
                    let (new_token, new_jti, exp) = create_access_token(
                        &self.config.auth,
                        session.user_id,
                        session.authority_id,
                        &session.session_id,
                    )?;
                    session.access_jti = new_jti;
                    session.expires_at = exp;
                    Some((new_token, exp))
                } else {
                    None
                };

            let persisted = session.clone();
            (effective_user, maybe_new, persisted)
        };
        self.persistence.upsert_session(&persisted).await;
        if self.config.multipoint_enabled {
            self.mark_session_active_distributed(
                persisted.user_id,
                &persisted.session_id,
                ttl_from(persisted.refresh_expires_at),
            )
            .await;
        }

        Ok(Authenticated {
            user,
            session_id: claims.sid,
            new_token: maybe_new.as_ref().map(|v| v.0.clone()),
            new_expires_at: maybe_new.map(|v| v.1),
        })
    }

    pub async fn revoke_session(&self, session_id: &str) {
        let mut guard = self.inner.write().await;
        let persisted = if let Some(session) = guard.sessions.get_mut(session_id) {
            session.revoked = true;
            let persisted = (
                session.user_id,
                session.session_id.clone(),
                session.access_jti.clone(),
                session.clone(),
            );
            let _ = session;
            if guard.active_sessions_by_user.get(&persisted.0) == Some(&persisted.1) {
                guard.active_sessions_by_user.remove(&persisted.0);
            }
            Some(persisted)
        } else {
            None
        };
        drop(guard);

        if let Some((user_id, _session_id, access_jti, session)) = persisted {
            let mut guard = self.inner.write().await;
            guard.blacklisted_access_jti.insert(access_jti);
            drop(guard);

            self.persistence.upsert_session(&session).await;
            if self.config.multipoint_enabled {
                self.mark_session_revoked_distributed(
                    &session.session_id,
                    ttl_from(session.refresh_expires_at),
                )
                .await;
                self.clear_session_active_distributed(user_id, &session.session_id)
                    .await;
            }
        }
    }

    pub async fn get_policy_paths(&self, authority_id: u32) -> Vec<CasbinInfo> {
        let casbin = self.casbin.read().await;
        casbin.get_policy_path_by_authority_id(authority_id)
    }

    pub async fn set_policy_paths(&self, authority_id: u32, policies: Vec<CasbinInfo>) {
        let mut casbin = self.casbin.write().await;
        casbin.set_policy_path_by_authority_id(authority_id, policies);
    }

    pub async fn is_allowed(&self, authority_id: u32, path: &str, method: &str) -> bool {
        if authority_id == 888 {
            return true;
        }
        let casbin = self.casbin.read().await;
        casbin.is_allowed(authority_id, path, method)
    }

    pub async fn resolve_service_identity(&self, token: &str) -> Option<ServiceIdentity> {
        let guard = self.inner.read().await;
        guard.service_identities.get(token).cloned()
    }

    pub async fn append_audit(&self, event: AuditEvent) {
        let mut guard = self.inner.write().await;
        guard.audits.push(event);
        let persisted = guard.audits.last().cloned();
        drop(guard);
        if let Some(item) = persisted {
            self.persistence.append_audit(&item).await;
        }
    }

    pub async fn switch_authority(
        &self,
        session_id: &str,
        user_id: u64,
        authority_id: u32,
    ) -> Result<(UserInfo, String, i64), String> {
        let (updated_user, token, exp, persisted_session) = {
            let mut guard = self.inner.write().await;
            let user = guard
                .users_by_id
                .get(&user_id)
                .ok_or_else(|| "用户不存在".to_string())?
                .clone();
            let updated_user = Self::effective_user_info(&user, authority_id)?;

            let session = guard
                .sessions
                .get_mut(session_id)
                .ok_or_else(|| "会话不存在".to_string())?;
            if session.user_id != user_id || session.revoked {
                return Err("会话已失效".to_string());
            }

            session.authority_id = authority_id;
            let (new_token, new_jti, new_exp) =
                create_access_token(&self.config.auth, user_id, authority_id, session_id)?;
            session.access_jti = new_jti;
            session.expires_at = new_exp;

            (updated_user, new_token, new_exp, session.clone())
        };

        self.persistence.upsert_session(&persisted_session).await;
        if self.config.multipoint_enabled {
            self.mark_session_active_distributed(
                persisted_session.user_id,
                &persisted_session.session_id,
                ttl_from(persisted_session.refresh_expires_at),
            )
            .await;
        }
        Ok((updated_user, token, exp * 1000))
    }

    fn effective_user_info(user: &UserRecord, authority_id: u32) -> Result<UserInfo, String> {
        let authority = user
            .info
            .authorities
            .iter()
            .find(|item| item.authority_id == authority_id)
            .cloned()
            .ok_or_else(|| "用户不具备该角色".to_string())?;
        let mut effective = user.info.clone();
        effective.authority_id = authority_id;
        effective.authority = authority;
        Ok(effective)
    }

    async fn active_distributed_session(&self, user_id: u64) -> Option<String> {
        let local = {
            let guard = self.inner.read().await;
            guard.active_sessions_by_user.get(&user_id).cloned()
        };
        match &self.session_registry {
            Some(registry) => registry.active_session(user_id).await.or(local),
            None => local,
        }
    }

    async fn is_active_session_valid(&self, user_id: u64, session_id: &str) -> bool {
        if !self.config.multipoint_enabled {
            return true;
        }

        if let Some(active) = self.active_distributed_session(user_id).await {
            return active == session_id;
        }

        true
    }

    async fn is_session_revoked_distributed(&self, session_id: &str) -> bool {
        match &self.session_registry {
            Some(registry) => registry.is_session_revoked(session_id).await,
            None => false,
        }
    }

    async fn mark_session_active_distributed(&self, user_id: u64, session_id: &str, ttl_secs: u64) {
        if let Some(registry) = &self.session_registry {
            registry.mark_active(user_id, session_id, ttl_secs).await;
        }
    }

    async fn mark_session_revoked_distributed(&self, session_id: &str, ttl_secs: u64) {
        if let Some(registry) = &self.session_registry {
            registry.revoke_session(session_id, ttl_secs).await;
        }
    }

    async fn clear_session_active_distributed(&self, user_id: u64, session_id: &str) {
        if let Some(registry) = &self.session_registry {
            registry.clear_active_if_matches(user_id, session_id).await;
        }
    }
}
