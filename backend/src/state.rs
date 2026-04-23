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

    pub async fn list_users(&self, query: UserListRequest) -> PageResult<UserInfo> {
        let guard = self.inner.read().await;
        let mut items: Vec<UserInfo> = guard
            .users_by_id
            .values()
            .map(|item| item.info.clone())
            .collect();
        items.sort_by_key(|item| item.id);

        let username = query.username.unwrap_or_default().to_lowercase();
        let nick_name = query.nick_name.unwrap_or_default().to_lowercase();
        let phone = query.phone.unwrap_or_default().to_lowercase();
        let email = query.email.unwrap_or_default().to_lowercase();

        items.retain(|item| {
            (username.is_empty() || item.user_name.to_lowercase().contains(&username))
                && (nick_name.is_empty() || item.nick_name.to_lowercase().contains(&nick_name))
                && (phone.is_empty() || item.phone.to_lowercase().contains(&phone))
                && (email.is_empty() || item.email.to_lowercase().contains(&email))
        });

        page_result(items, query.page, query.page_size)
    }

    pub async fn list_authorities(&self) -> Vec<AuthorityInfo> {
        let guard = self.inner.read().await;
        guard.authorities.clone()
    }

    pub async fn list_menu_tree(&self) -> Vec<MenuInfo> {
        let guard = self.inner.read().await;
        guard.menus.clone()
    }

    pub async fn get_user_ids_by_authority(&self, authority_id: u32) -> Vec<u64> {
        let guard = self.inner.read().await;
        let mut user_ids = guard
            .users_by_id
            .values()
            .filter(|record| {
                record.info.authority_id == authority_id
                    || record
                        .info
                        .authorities
                        .iter()
                        .any(|item| item.authority_id == authority_id)
            })
            .map(|record| record.info.id)
            .collect::<Vec<_>>();
        user_ids.sort_unstable();
        user_ids
    }

    pub async fn set_role_users(
        &self,
        authority_id: u32,
        user_ids: Vec<u64>,
    ) -> Result<(), String> {
        let selected_authority = flatten_authorities(&self.list_authorities().await)
            .into_iter()
            .find(|item| item.authority_id == authority_id)
            .ok_or_else(|| "角色不存在".to_string())?;

        let mut guard = self.inner.write().await;
        let selected = user_ids.into_iter().collect::<HashSet<_>>();
        let all_authorities = flatten_authorities(&guard.authorities);

        for record in guard.users_by_id.values_mut() {
            let had_primary = record.info.authority_id == authority_id;
            let should_have = selected.contains(&record.info.id) || had_primary;
            if should_have {
                if !record
                    .info
                    .authorities
                    .iter()
                    .any(|item| item.authority_id == authority_id)
                {
                    record.info.authorities.push(selected_authority.clone());
                }
            } else {
                record
                    .info
                    .authorities
                    .retain(|item| item.authority_id != authority_id);
            }
            if record.info.authorities.is_empty() {
                record.info.authorities = all_authorities.clone();
            }
        }

        let refreshed = guard
            .users_by_id
            .values()
            .map(|record| (record.info.user_name.clone(), record.clone()))
            .collect::<HashMap<_, _>>();
        guard.users_by_name = refreshed;
        Ok(())
    }

    pub async fn get_menu_authority(&self, authority_id: u32) -> Vec<MenuInfo> {
        let guard = self.inner.read().await;
        let flat_menus = flatten_menus_for_state(&guard.menus);
        let selected = guard
            .authority_menu_ids
            .get(&authority_id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<HashSet<_>>();
        flat_menus
            .into_iter()
            .filter(|menu| selected.contains(&menu.id))
            .collect()
    }

    pub async fn set_menu_authority(
        &self,
        authority_id: u32,
        menu_ids: Vec<u64>,
    ) -> Result<(), String> {
        let mut guard = self.inner.write().await;
        guard
            .authority_menu_ids
            .insert(authority_id, dedupe_u64(menu_ids));
        Ok(())
    }

    pub async fn get_authority_buttons(&self, authority_id: u32, menu_id: u64) -> Vec<u64> {
        let guard = self.inner.read().await;
        guard
            .authority_button_ids
            .get(&(authority_id, menu_id))
            .cloned()
            .unwrap_or_default()
    }

    pub async fn set_authority_buttons(
        &self,
        authority_id: u32,
        menu_id: u64,
        selected: Vec<u64>,
    ) -> Result<(), String> {
        let mut guard = self.inner.write().await;
        guard
            .authority_button_ids
            .insert((authority_id, menu_id), dedupe_u64(selected));
        Ok(())
    }

    pub async fn get_menu_roles(&self, menu_id: u64) -> (Vec<u32>, Vec<u32>) {
        let guard = self.inner.read().await;
        let authority_ids = guard
            .authority_menu_ids
            .iter()
            .filter_map(|(authority_id, menu_ids)| {
                menu_ids.contains(&menu_id).then_some(*authority_id)
            })
            .collect::<Vec<_>>();
        let menu_name = flatten_menus_for_state(&guard.menus)
            .into_iter()
            .find(|menu| menu.id == menu_id)
            .map(|menu| menu.name)
            .unwrap_or_default();
        let default_router_authority_ids = flatten_authorities(&guard.authorities)
            .into_iter()
            .filter(|authority| authority.default_router == menu_name)
            .map(|authority| authority.authority_id)
            .collect::<Vec<_>>();
        (authority_ids, default_router_authority_ids)
    }

    pub async fn set_menu_roles(
        &self,
        menu_id: u64,
        authority_ids: Vec<u32>,
    ) -> Result<(), String> {
        let selected = authority_ids.into_iter().collect::<HashSet<_>>();
        let mut guard = self.inner.write().await;
        for (authority_id, menu_ids) in &mut guard.authority_menu_ids {
            if selected.contains(authority_id) {
                if !menu_ids.contains(&menu_id) {
                    menu_ids.push(menu_id);
                }
            } else {
                menu_ids.retain(|id| *id != menu_id);
            }
            *menu_ids = dedupe_u64(menu_ids.clone());
        }
        for authority_id in selected {
            guard
                .authority_menu_ids
                .entry(authority_id)
                .or_insert_with(|| vec![menu_id]);
        }
        Ok(())
    }

    pub async fn list_api_entries(&self, query: ApiListRequest) -> PageResult<ApiInfo> {
        let guard = self.inner.read().await;
        let mut items = guard.apis.clone();

        let path = query.path.unwrap_or_default().to_lowercase();
        let description = query.description.unwrap_or_default().to_lowercase();
        let api_group = query.api_group.unwrap_or_default().to_lowercase();
        let method = query.method.unwrap_or_default().to_lowercase();

        items.retain(|item| {
            (path.is_empty() || item.path.to_lowercase().contains(&path))
                && (description.is_empty()
                    || item.description.to_lowercase().contains(&description))
                && (api_group.is_empty() || item.api_group.to_lowercase().contains(&api_group))
                && (method.is_empty() || item.method.to_lowercase() == method)
        });

        page_result(items, query.page, query.page_size)
    }

    pub async fn list_dictionaries(&self) -> PageResult<DictionaryInfo> {
        let guard = self.inner.read().await;
        page_result(guard.dictionaries.clone(), Some(1), Some(50))
    }

    pub async fn list_params(&self) -> PageResult<ParamInfo> {
        let guard = self.inner.read().await;
        page_result(guard.params.clone(), Some(1), Some(50))
    }

    pub async fn upsert_param(&self, input: crate::models::ParamUpsertRequest) -> ParamInfo {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard
            .params
            .iter_mut()
            .find(|item| input.id == Some(item.id) || item.key == input.key)
        {
            existing.key = input.key;
            existing.value = input.value;
            existing.desc = input.desc;
            return existing.clone();
        }

        let next_id = guard.params.iter().map(|item| item.id).max().unwrap_or(0) + 1;
        let created = ParamInfo {
            id: next_id,
            key: input.key,
            value: input.value,
            desc: input.desc,
        };
        guard.params.push(created.clone());
        created
    }

    pub async fn update_profile(
        &self,
        user_id: u64,
        input: crate::models::ProfileUpdateRequest,
    ) -> Result<UserInfo, String> {
        let mut guard = self.inner.write().await;
        let user_name = guard
            .users_by_id
            .get(&user_id)
            .map(|record| record.info.user_name.clone())
            .ok_or_else(|| "用户不存在".to_string())?;

        let updated = {
            let record = guard
                .users_by_id
                .get_mut(&user_id)
                .ok_or_else(|| "用户不存在".to_string())?;
            record.info.nick_name = input.nick_name;
            record.info.phone = input.phone;
            record.info.email = input.email;
            record.info.clone()
        };

        if let Some(record) = guard.users_by_name.get_mut(&user_name) {
            record.info = updated.clone();
        }

        Ok(updated)
    }

    pub async fn upsert_user(
        &self,
        input: crate::models::UserUpsertRequest,
    ) -> Result<UserInfo, String> {
        let authorities = flatten_authorities(&self.list_authorities().await);
        let selected_authority = authorities
            .iter()
            .find(|item| item.authority_id == input.authority_id)
            .cloned()
            .ok_or_else(|| "角色不存在".to_string())?;
        let mut selected_authority_ids = input.authority_ids.clone();
        if !selected_authority_ids.contains(&input.authority_id) {
            selected_authority_ids.insert(0, input.authority_id);
        }
        selected_authority_ids.sort_unstable();
        selected_authority_ids.dedup();
        let switchable_authorities = selected_authority_ids
            .iter()
            .filter_map(|authority_id| {
                authorities
                    .iter()
                    .find(|item| item.authority_id == *authority_id)
                    .cloned()
            })
            .collect::<Vec<_>>();
        let switchable_authorities = if switchable_authorities.is_empty() {
            vec![selected_authority.clone()]
        } else {
            switchable_authorities
        };

        let mut guard = self.inner.write().await;
        let target_id = input
            .id
            .unwrap_or_else(|| guard.users_by_id.keys().max().copied().unwrap_or(0) + 1);

        if guard
            .users_by_name
            .iter()
            .any(|(user_name, record)| *user_name == input.user_name && record.info.id != target_id)
        {
            return Err("用户名已存在".to_string());
        }

        let existing_user_name = guard
            .users_by_id
            .get(&target_id)
            .map(|record| record.info.user_name.clone());
        if let Some(old_user_name) = existing_user_name {
            guard.users_by_name.remove(&old_user_name);
        }

        let existing_uuid = guard
            .users_by_id
            .get(&target_id)
            .map(|record| record.info.uuid);
        let password = guard
            .users_by_id
            .get(&target_id)
            .map(|record| record.password.clone())
            .unwrap_or_else(|| {
                read_default_user_password()
                    .and_then(|password| hash_password(&password))
                    .expect("hash default user password")
            });

        let info = UserInfo {
            id: target_id,
            uuid: existing_uuid.unwrap_or_else(Uuid::new_v4),
            user_name: input.user_name,
            nick_name: input.nick_name,
            authority_id: input.authority_id,
            authority: selected_authority,
            authorities: switchable_authorities,
            header_img: format!("https://example.com/gaa-user-{}.png", target_id),
            phone: input.phone,
            email: input.email,
            enable: input.enable,
        };
        let record = UserRecord {
            info: info.clone(),
            password,
        };

        guard
            .users_by_name
            .insert(info.user_name.clone(), record.clone());
        guard.users_by_id.insert(target_id, record);
        Ok(info)
    }

    pub async fn upsert_authority(
        &self,
        input: crate::models::AuthorityUpsertRequest,
    ) -> Result<AuthorityInfo, String> {
        let mut guard = self.inner.write().await;
        let mut flat = flatten_authorities(&guard.authorities);
        let normalized_status = if input.enable == 2 || input.status == 2 {
            2
        } else {
            1
        };
        let node_id = input.id.unwrap_or_else(|| {
            flat.iter()
                .find(|item| item.authority_id == input.authority_id)
                .map(|item| item.id)
                .unwrap_or_else(|| flat.iter().map(|item| item.id).max().unwrap_or(0) + 1)
        });

        if flat
            .iter()
            .any(|item| item.authority_id == input.authority_id && item.id != node_id)
        {
            return Err("角色标识已存在".to_string());
        }

        if let Some(existing) = flat.iter_mut().find(|item| item.id == node_id) {
            existing.authority_id = input.authority_id;
            existing.authority_name = input.authority_name;
            existing.default_router = input.default_router;
            existing.parent_id = input.parent_id;
            existing.enable = normalized_status;
            existing.status = normalized_status;
        } else {
            flat.push(AuthorityInfo {
                id: node_id,
                authority_id: input.authority_id,
                authority_name: input.authority_name,
                default_router: input.default_router,
                parent_id: input.parent_id,
                enable: normalized_status,
                status: normalized_status,
                children: vec![],
            });
        }

        let saved = flat
            .iter()
            .find(|item| item.id == node_id)
            .cloned()
            .ok_or_else(|| "保存角色失败".to_string())?;
        guard.authorities = rebuild_authority_tree(flat);
        Ok(saved)
    }

    pub async fn upsert_menu(&self, input: crate::models::MenuUpsertRequest) -> MenuInfo {
        let mut guard = self.inner.write().await;
        let mut flat = flatten_menus_for_state(&guard.menus);
        let node_id = input
            .id
            .unwrap_or_else(|| flat.iter().map(|item| item.id).max().unwrap_or(0) + 1);

        if let Some(existing) = flat.iter_mut().find(|item| item.id == node_id) {
            existing.parent_id = input.parent_id;
            existing.name = input.name;
            existing.path = input.path;
            existing.component = input.component;
            existing.sort = input.sort;
            existing.hidden = input.hidden;
            existing.meta = input.meta;
            existing.menu_btn = input.menu_btn;
            existing.parameters = input.parameters;
        } else {
            flat.push(MenuInfo {
                id: node_id,
                parent_id: input.parent_id,
                name: input.name,
                path: input.path,
                component: input.component,
                sort: input.sort,
                hidden: input.hidden,
                meta: input.meta,
                menu_btn: input.menu_btn,
                parameters: input.parameters,
                children: vec![],
            });
        }

        let saved = flat
            .iter()
            .find(|item| item.id == node_id)
            .cloned()
            .unwrap_or(MenuInfo {
                id: node_id,
                parent_id: 0,
                name: String::new(),
                path: String::new(),
                component: String::new(),
                sort: 0,
                hidden: false,
                meta: MenuMeta {
                    title: String::new(),
                    icon: String::new(),
                },
                menu_btn: vec![],
                parameters: vec![],
                children: vec![],
            });
        guard.menus = rebuild_menu_tree(flat);
        saved
    }

    pub async fn upsert_api(&self, input: crate::models::ApiUpsertRequest) -> ApiInfo {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard.apis.iter_mut().find(|item| {
            input.id == Some(item.id) || (item.path == input.path && item.method == input.method)
        }) {
            existing.path = input.path;
            existing.api_group = input.api_group;
            existing.description = input.description;
            existing.method = input.method;
            return existing.clone();
        }

        let next_id = guard.apis.iter().map(|item| item.id).max().unwrap_or(0) + 1;
        let created = ApiInfo {
            id: next_id,
            path: input.path,
            api_group: input.api_group,
            description: input.description,
            method: input.method,
        };
        guard.apis.push(created.clone());
        created
    }

    pub async fn upsert_dictionary(
        &self,
        input: crate::models::DictionaryUpsertRequest,
    ) -> DictionaryInfo {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard
            .dictionaries
            .iter_mut()
            .find(|item| input.id == Some(item.id) || item.type_name == input.type_name)
        {
            existing.name = input.name;
            existing.type_name = input.type_name;
            existing.status = input.status;
            existing.desc = input.desc;
            return existing.clone();
        }

        let next_id = guard
            .dictionaries
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            + 1;
        let created = DictionaryInfo {
            id: next_id,
            name: input.name,
            type_name: input.type_name,
            status: input.status,
            desc: input.desc,
        };
        guard.dictionaries.push(created.clone());
        created
    }

    pub async fn update_system_config(
        &self,
        input: crate::models::SystemConfigUpdateRequest,
    ) -> SystemConfigInfo {
        let mut guard = self.inner.write().await;
        guard.system_config = SystemConfigInfo {
            bind_address: input.bind_address,
            database_url: input.database_url,
            redis_url: input.redis_url,
            multipoint_enabled: input.multipoint_enabled,
            compatibility_refresh_headers: input.compatibility_refresh_headers,
        };
        redact_system_config(guard.system_config.clone())
    }

    pub async fn list_llm_configs(&self) -> PageResult<LlmConfigRecord> {
        let guard = self.inner.read().await;
        page_result(guard.llm_configs.clone(), Some(1), Some(50))
    }

    pub async fn upsert_llm_config(
        &self,
        input: crate::models::LlmConfigUpsertRequest,
    ) -> LlmConfigRecord {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard
            .llm_configs
            .iter_mut()
            .find(|item| input.id == Some(item.id))
        {
            existing.provider = input.provider;
            existing.model = input.model;
            existing.base_url = input.base_url;
            existing.api_key = input.api_key;
            existing.status = input.status;
            return existing.clone();
        }

        let next_id = next_id(guard.llm_configs.iter().map(|item| item.id));
        let created = LlmConfigRecord {
            id: next_id,
            provider: input.provider,
            model: input.model,
            base_url: input.base_url,
            api_key: input.api_key,
            status: input.status,
        };
        guard.llm_configs.insert(0, created.clone());
        created
    }

    pub async fn delete_llm_config(&self, id: u64) -> bool {
        let mut guard = self.inner.write().await;
        let before = guard.llm_configs.len();
        guard.llm_configs.retain(|item| item.id != id);
        before != guard.llm_configs.len()
    }

    pub async fn list_frontend_nav(&self) -> PageResult<FrontendNavRecord> {
        let guard = self.inner.read().await;
        let mut rows = guard.frontend_nav.clone();
        rows.sort_by_key(|item| (item.order, item.id));
        page_result(rows, Some(1), Some(100))
    }

    pub async fn upsert_frontend_nav(
        &self,
        input: crate::models::FrontendNavUpsertRequest,
    ) -> FrontendNavRecord {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard
            .frontend_nav
            .iter_mut()
            .find(|item| input.id == Some(item.id))
        {
            existing.title = input.title;
            existing.path = input.path;
            existing.icon = input.icon;
            existing.order = input.order;
            existing.visible = input.visible;
            return existing.clone();
        }

        let created = FrontendNavRecord {
            id: next_id(guard.frontend_nav.iter().map(|item| item.id)),
            title: input.title,
            path: input.path,
            icon: input.icon,
            order: input.order,
            visible: input.visible,
        };
        guard.frontend_nav.insert(0, created.clone());
        created
    }

    pub async fn delete_frontend_nav(&self, id: u64) -> bool {
        let mut guard = self.inner.write().await;
        let before = guard.frontend_nav.len();
        guard.frontend_nav.retain(|item| item.id != id);
        before != guard.frontend_nav.len()
    }

    pub async fn frontend_settings(&self) -> FrontendSettingsRecord {
        let guard = self.inner.read().await;
        guard.frontend_settings.clone()
    }

    pub async fn update_frontend_settings(
        &self,
        input: FrontendSettingsUpdateRequest,
    ) -> FrontendSettingsRecord {
        let mut guard = self.inner.write().await;
        guard.frontend_settings = FrontendSettingsRecord {
            logo_url: input.logo_url,
            site_name: input.site_name,
            site_slogan: input.site_slogan,
            site_description: input.site_description,
            record_number: input.record_number,
            smtp_host: input.smtp_host,
            smtp_port: input.smtp_port,
            smtp_user: input.smtp_user,
            smtp_password: input.smtp_password,
            mail_from: input.mail_from,
        };
        guard.frontend_settings.clone()
    }

    pub async fn list_article_categories(&self) -> PageResult<ArticleCategoryRecord> {
        let guard = self.inner.read().await;
        let mut rows = guard.article_categories.clone();
        rows.sort_by_key(|item| (item.sort, item.id));
        page_result(rows, Some(1), Some(100))
    }

    pub async fn upsert_article_category(
        &self,
        input: crate::models::ArticleCategoryUpsertRequest,
    ) -> ArticleCategoryRecord {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard
            .article_categories
            .iter_mut()
            .find(|item| input.id == Some(item.id))
        {
            existing.name = input.name;
            existing.slug = input.slug;
            existing.sort = input.sort;
            existing.status = input.status;
            return existing.clone();
        }

        let created = ArticleCategoryRecord {
            id: next_id(guard.article_categories.iter().map(|item| item.id)),
            name: input.name,
            slug: input.slug,
            sort: input.sort,
            status: input.status,
        };
        guard.article_categories.insert(0, created.clone());
        created
    }

    pub async fn delete_article_category(&self, id: u64) -> bool {
        let mut guard = self.inner.write().await;
        let before = guard.article_categories.len();
        guard.article_categories.retain(|item| item.id != id);
        before != guard.article_categories.len()
    }

    pub async fn list_articles(&self) -> PageResult<ArticleRecord> {
        let guard = self.inner.read().await;
        page_result(guard.articles.clone(), Some(1), Some(100))
    }

    pub async fn upsert_article(
        &self,
        input: crate::models::ArticleUpsertRequest,
    ) -> ArticleRecord {
        let mut guard = self.inner.write().await;
        let updated_at = timestamp_ms_string();
        if let Some(existing) = guard
            .articles
            .iter_mut()
            .find(|item| input.id == Some(item.id))
        {
            existing.category_id = input.category_id;
            existing.title = input.title;
            existing.slug = input.slug;
            existing.content = input.content;
            existing.status = input.status;
            existing.updated_at = updated_at;
            return existing.clone();
        }

        let created = ArticleRecord {
            id: next_id(guard.articles.iter().map(|item| item.id)),
            category_id: input.category_id,
            title: input.title,
            slug: input.slug,
            content: input.content,
            status: input.status,
            updated_at,
        };
        guard.articles.insert(0, created.clone());
        created
    }

    pub async fn delete_article(&self, id: u64) -> bool {
        let mut guard = self.inner.write().await;
        let before = guard.articles.len();
        guard.articles.retain(|item| item.id != id);
        before != guard.articles.len()
    }

    pub async fn list_members(&self) -> PageResult<MemberRecord> {
        let guard = self.inner.read().await;
        page_result(guard.members.clone(), Some(1), Some(100))
    }

    pub async fn upsert_member(&self, input: crate::models::MemberUpsertRequest) -> MemberRecord {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard
            .members
            .iter_mut()
            .find(|item| input.id == Some(item.id))
        {
            existing.email = input.email;
            existing.nickname = input.nickname;
            existing.provider = input.provider;
            existing.status = input.status;
            existing.last_login = input.last_login.unwrap_or_else(timestamp_ms_string);
            existing.console_path = input.console_path;
            return existing.clone();
        }

        let created = MemberRecord {
            id: next_id(guard.members.iter().map(|item| item.id)),
            email: input.email,
            nickname: input.nickname,
            provider: input.provider,
            status: input.status,
            last_login: input.last_login.unwrap_or_else(timestamp_ms_string),
            console_path: input.console_path,
        };
        guard.members.insert(0, created.clone());
        created
    }

    pub async fn delete_member(&self, id: u64) -> bool {
        let mut guard = self.inner.write().await;
        let before = guard.members.len();
        guard.members.retain(|item| item.id != id);
        before != guard.members.len()
    }

    pub async fn list_console_menus(&self) -> PageResult<ConsoleMenuRecord> {
        let guard = self.inner.read().await;
        let mut rows = guard.console_menus.clone();
        rows.sort_by_key(|item| (item.order, item.id));
        page_result(rows, Some(1), Some(100))
    }

    pub async fn upsert_console_menu(
        &self,
        input: crate::models::ConsoleMenuUpsertRequest,
    ) -> ConsoleMenuRecord {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard
            .console_menus
            .iter_mut()
            .find(|item| input.id == Some(item.id))
        {
            existing.title = input.title;
            existing.group = input.group;
            existing.path = input.path;
            existing.icon = input.icon;
            existing.order = input.order;
            existing.visible = input.visible;
            return existing.clone();
        }

        let created = ConsoleMenuRecord {
            id: next_id(guard.console_menus.iter().map(|item| item.id)),
            title: input.title,
            group: input.group,
            path: input.path,
            icon: input.icon,
            order: input.order,
            visible: input.visible,
        };
        guard.console_menus.insert(0, created.clone());
        created
    }

    pub async fn delete_console_menu(&self, id: u64) -> bool {
        let mut guard = self.inner.write().await;
        let before = guard.console_menus.len();
        guard.console_menus.retain(|item| item.id != id);
        before != guard.console_menus.len()
    }

    pub async fn list_api_tokens(&self) -> PageResult<ApiTokenRecord> {
        let guard = self.inner.read().await;
        page_result(guard.api_tokens.clone(), Some(1), Some(50))
    }

    pub async fn issue_api_token(
        &self,
        input: crate::models::ApiTokenIssueRequest,
    ) -> ApiTokenRecord {
        let mut guard = self.inner.write().await;
        let next_id = guard
            .api_tokens
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            + 1;
        let created = ApiTokenRecord {
            id: next_id,
            name: input.name,
            scope: input.scope,
            ttl: input.ttl,
            status: "已登记".to_string(),
        };
        guard.api_tokens.insert(0, created.clone());
        guard.api_tokens.truncate(20);
        created
    }

    pub async fn clear_api_tokens(&self) {
        let mut guard = self.inner.write().await;
        guard.api_tokens.clear();
    }

    pub async fn list_packages(&self) -> PageResult<PackageRecord> {
        let guard = self.inner.read().await;
        page_result(guard.packages.clone(), Some(1), Some(50))
    }

    pub async fn upsert_package(
        &self,
        input: crate::models::PackageUpsertRequest,
    ) -> PackageRecord {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard
            .packages
            .iter_mut()
            .find(|item| input.id == Some(item.id) || item.name == input.name)
        {
            existing.kind = input.kind;
            existing.output = input.output;
            existing.summary = input.summary;
            return existing.clone();
        }

        let next_id = guard.packages.iter().map(|item| item.id).max().unwrap_or(0) + 1;
        let created = PackageRecord {
            id: next_id,
            name: input.name,
            kind: input.kind,
            output: input.output,
            summary: input.summary,
        };
        guard.packages.insert(0, created.clone());
        guard.packages.truncate(20);
        created
    }

    pub async fn list_plugin_manifests(&self) -> PageResult<PluginManifestRecord> {
        let guard = self.inner.read().await;
        page_result(guard.plugin_manifests.clone(), Some(1), Some(20))
    }

    pub async fn upsert_plugin_manifest(
        &self,
        input: crate::models::PluginManifestUpsertRequest,
    ) -> PluginManifestRecord {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard
            .plugin_manifests
            .iter_mut()
            .find(|item| item.plugin_name == input.plugin_name)
        {
            existing.menu_group = input.menu_group;
            existing.menu_ids = input.menu_ids;
            existing.api_ids = input.api_ids;
            existing.dictionary_ids = input.dictionary_ids;
            existing.saved_at = now_ts() * 1000;
            return existing.clone();
        }

        let next_id = guard
            .plugin_manifests
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            + 1;
        let created = PluginManifestRecord {
            id: next_id,
            plugin_name: input.plugin_name,
            menu_group: input.menu_group,
            menu_ids: input.menu_ids,
            api_ids: input.api_ids,
            dictionary_ids: input.dictionary_ids,
            saved_at: now_ts() * 1000,
        };
        guard.plugin_manifests.insert(0, created.clone());
        guard.plugin_manifests.truncate(20);
        created
    }

    pub async fn list_plugin_installs(&self) -> PageResult<PluginInstallRecord> {
        let guard = self.inner.read().await;
        page_result(guard.plugin_installs.clone(), Some(1), Some(20))
    }

    pub async fn install_plugin(
        &self,
        input: crate::models::PluginInstallRequest,
    ) -> PluginInstallRecord {
        let mut guard = self.inner.write().await;
        let next_id = guard
            .plugin_installs
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            + 1;
        let created = PluginInstallRecord {
            id: next_id,
            name: input.file_name.trim_end_matches(".zip").to_string(),
            kind: input.kind,
            target: input.target,
            manifest: input.manifest,
            status: "已登记".to_string(),
            created_at: now_ts() * 1000,
        };
        guard.plugin_installs.insert(0, created.clone());
        guard.plugin_installs.truncate(20);
        created
    }

    pub async fn list_auto_code_registry(&self) -> PageResult<AutoCodeRegistryRecord> {
        let guard = self.inner.read().await;
        page_result(guard.auto_code_registry.clone(), Some(1), Some(20))
    }

    pub async fn append_auto_code_registry(
        &self,
        input: crate::models::AutoCodeRegistryUpsertRequest,
    ) -> AutoCodeRegistryRecord {
        let mut guard = self.inner.write().await;
        let next_id = guard
            .auto_code_registry
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            + 1;
        let created = AutoCodeRegistryRecord {
            id: next_id,
            created_at: now_ts() * 1000,
            payload: input.payload,
        };
        guard.auto_code_registry.insert(0, created.clone());
        guard.auto_code_registry.truncate(20);
        created
    }

    pub async fn clear_auto_code_registry(&self) {
        let mut guard = self.inner.write().await;
        guard.auto_code_registry.clear();
    }

    pub async fn list_releases(&self) -> PageResult<ReleaseRecord> {
        let guard = self.inner.read().await;
        page_result(guard.releases.clone(), Some(1), Some(20))
    }

    pub async fn append_release(
        &self,
        input: crate::models::ReleaseAppendRequest,
    ) -> ReleaseRecord {
        let mut guard = self.inner.write().await;
        let next_id = guard.releases.iter().map(|item| item.id).max().unwrap_or(0) + 1;
        let created = ReleaseRecord {
            id: next_id,
            created_at: now_ts() * 1000,
            note: input.note,
        };
        guard.releases.insert(0, created.clone());
        guard.releases.truncate(20);
        created
    }

    pub async fn list_email_records(&self) -> PageResult<EmailRecord> {
        let guard = self.inner.read().await;
        page_result(guard.email_records.clone(), Some(1), Some(20))
    }

    pub async fn send_email(
        &self,
        input: crate::models::EmailSendRequest,
        mode: &str,
    ) -> EmailRecord {
        let mut guard = self.inner.write().await;
        let next_id = guard
            .email_records
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            + 1;
        let record = EmailRecord {
            id: next_id,
            to: input
                .to
                .unwrap_or_else(|| "ops-team@gaa.local".to_string())
                .trim()
                .to_string(),
            subject: input
                .subject
                .unwrap_or_else(|| "GAA 邮件测试".to_string())
                .trim()
                .to_string(),
            body: input
                .body
                .unwrap_or_else(|| "当前邮件由 Rust 重构版邮件工作台登记。".to_string())
                .trim()
                .to_string(),
            mode: mode.to_string(),
            status: if mode == "test" {
                "测试已发送"
            } else {
                "已发送"
            }
            .to_string(),
            created_at: now_ts() * 1000,
        };
        guard.email_records.insert(0, record.clone());
        guard.email_records.truncate(20);
        record
    }

    pub async fn list_announcements(
        &self,
        input: crate::models::AnnouncementListRequest,
    ) -> PageResult<AnnouncementRecord> {
        let guard = self.inner.read().await;
        let keyword = input.title.unwrap_or_default().trim().to_lowercase();
        let rows = guard
            .announcements
            .iter()
            .filter(|item| {
                keyword.is_empty()
                    || item.title.to_lowercase().contains(&keyword)
                    || item.content.to_lowercase().contains(&keyword)
            })
            .cloned()
            .collect();
        page_result(rows, input.page, input.page_size)
    }

    pub async fn get_announcement(&self, id: u64) -> Option<AnnouncementRecord> {
        let guard = self.inner.read().await;
        guard
            .announcements
            .iter()
            .find(|item| item.id == id)
            .cloned()
    }

    pub async fn upsert_announcement(
        &self,
        input: crate::models::AnnouncementUpsertRequest,
    ) -> AnnouncementRecord {
        let mut guard = self.inner.write().await;
        let now = now_ts() * 1000;
        if let Some(existing) = guard
            .announcements
            .iter_mut()
            .find(|item| item.id == input.id.unwrap_or(0))
        {
            existing.title = input.title.trim().to_string();
            existing.content = input.content.trim().to_string();
            existing.user_id = input.user_id;
            existing.attachments = sanitize_announcement_attachments(input.attachments);
            existing.updated_at = now;
            return existing.clone();
        }

        let next_id = guard
            .announcements
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            + 1;
        let record = AnnouncementRecord {
            id: next_id,
            created_at: now,
            updated_at: now,
            title: input.title.trim().to_string(),
            content: input.content.trim().to_string(),
            user_id: input.user_id,
            attachments: sanitize_announcement_attachments(input.attachments),
        };
        guard.announcements.insert(0, record.clone());
        record
    }

    pub async fn delete_announcement(&self, id: u64) -> bool {
        let mut guard = self.inner.write().await;
        let before = guard.announcements.len();
        guard.announcements.retain(|item| item.id != id);
        before != guard.announcements.len()
    }

    pub async fn delete_announcements(&self, ids: Vec<u64>) -> usize {
        let mut guard = self.inner.write().await;
        let id_set: HashSet<u64> = ids.into_iter().collect();
        let before = guard.announcements.len();
        guard
            .announcements
            .retain(|item| !id_set.contains(&item.id));
        before.saturating_sub(guard.announcements.len())
    }

    pub async fn announcement_data_source(&self) -> AnnouncementDataSourcePayload {
        let guard = self.inner.read().await;
        let mut users = guard
            .users_by_id
            .values()
            .map(|item| SelectOption {
                label: item.info.nick_name.clone(),
                value: item.info.id,
            })
            .collect::<Vec<_>>();
        users.sort_by(|a, b| a.value.cmp(&b.value));
        AnnouncementDataSourcePayload { user_id: users }
    }

    pub async fn replace_upload_queue(
        &self,
        input: crate::models::UploadQueueRequest,
    ) -> PageResult<UploadFileRecord> {
        let mut guard = self.inner.write().await;
        let mode = input.mode.unwrap_or_else(|| "replace".to_string());

        if mode == "inspect" {
            return page_result(guard.upload_files.clone(), Some(1), Some(50));
        }

        let incoming = input
            .files
            .into_iter()
            .map(|item| UploadFileRecord {
                id: 0,
                name: item.name,
                size: item.size,
                status: "待上传".to_string(),
            })
            .collect::<Vec<_>>();

        guard.upload_files = if mode == "append" {
            let mut merged = guard.upload_files.clone();
            for item in incoming {
                if let Some(existing) = merged
                    .iter_mut()
                    .find(|existing| existing.name == item.name && existing.size == item.size)
                {
                    if existing.status != "上传完成" {
                        existing.status = "待上传".to_string();
                    }
                } else {
                    merged.push(item);
                }
            }
            merged
        } else {
            incoming
        };

        for (index, item) in guard.upload_files.iter_mut().enumerate() {
            item.id = (index + 1) as u64;
        }

        page_result(guard.upload_files.clone(), Some(1), Some(50))
    }

    pub async fn complete_upload_queue(&self) -> PageResult<UploadFileRecord> {
        let mut guard = self.inner.write().await;
        for item in &mut guard.upload_files {
            item.status = "上传完成".to_string();
        }
        page_result(guard.upload_files.clone(), Some(1), Some(50))
    }

    pub async fn list_resume_uploads(&self) -> PageResult<ResumeUploadRecord> {
        let guard = self.inner.read().await;
        page_result(guard.resume_uploads.clone(), Some(1), Some(50))
    }

    pub async fn advance_resume_uploads(&self) -> PageResult<ResumeUploadRecord> {
        let mut guard = self.inner.write().await;
        for item in &mut guard.resume_uploads {
            if item.progress < 100 {
                item.progress = (item.progress + 12).min(100);
                item.status = if item.progress >= 100 {
                    "已完成".to_string()
                } else {
                    "上传中".to_string()
                };
            }
        }
        page_result(guard.resume_uploads.clone(), Some(1), Some(50))
    }

    pub async fn resume_interrupted_upload(&self) -> PageResult<ResumeUploadRecord> {
        let mut guard = self.inner.write().await;
        if let Some(item) = guard.resume_uploads.first_mut() {
            item.status = "已恢复续传".to_string();
        }
        page_result(guard.resume_uploads.clone(), Some(1), Some(50))
    }

    pub async fn get_or_create_scan_session(&self) -> ScanSessionRecord {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard.scan_sessions.first().cloned() {
            return existing;
        }
        let created = ScanSessionRecord {
            session_id: format!("scan-{}", now_ts()),
            status: "待扫码".to_string(),
        };
        guard.scan_sessions.push(created.clone());
        created
    }

    pub async fn update_scan_session(
        &self,
        input: crate::models::ScanSessionUpdateRequest,
    ) -> ScanSessionRecord {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard
            .scan_sessions
            .iter_mut()
            .find(|item| item.session_id == input.session_id)
        {
            existing.status = input.status;
            return existing.clone();
        }

        let created = ScanSessionRecord {
            session_id: input.session_id,
            status: input.status,
        };
        guard.scan_sessions.push(created.clone());
        created
    }

    pub async fn list_customers(
        &self,
        input: crate::models::CustomerListRequest,
    ) -> PageResult<CustomerRecord> {
        let guard = self.inner.read().await;
        let keyword = input.keyword.unwrap_or_default().trim().to_lowercase();
        let rows = guard
            .customers
            .iter()
            .filter(|item| {
                keyword.is_empty()
                    || item.customer_name.to_lowercase().contains(&keyword)
                    || item.customer_phone_data.contains(&keyword)
                    || item.customer_level.to_lowercase().contains(&keyword)
                    || item.customer_status.to_lowercase().contains(&keyword)
            })
            .cloned()
            .collect();
        page_result(rows, input.page, input.page_size)
    }

    pub async fn get_customer(&self, id: u64) -> Option<CustomerRecord> {
        let guard = self.inner.read().await;
        guard.customers.iter().find(|item| item.id == id).cloned()
    }

    pub async fn upsert_customer(
        &self,
        input: crate::models::CustomerUpsertRequest,
        actor_user_id: u64,
    ) -> CustomerRecord {
        let mut guard = self.inner.write().await;
        let now = now_ts() * 1000;
        if let Some(existing) = guard
            .customers
            .iter_mut()
            .find(|item| item.id == input.id.unwrap_or(0))
        {
            existing.customer_name = input.customer_name;
            existing.customer_phone_data = input.customer_phone_data;
            existing.customer_level = non_empty(input.customer_level, &existing.customer_level);
            existing.customer_status = non_empty(input.customer_status, &existing.customer_status);
            existing.remark = input.remark.unwrap_or_default();
            existing.updated_at = now;
            return existing.clone();
        }

        let next_id = guard
            .customers
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            + 1;
        let created = CustomerRecord {
            id: next_id,
            created_at: now,
            updated_at: now,
            customer_name: input.customer_name,
            customer_phone_data: input.customer_phone_data,
            sys_user_id: input.sys_user_id.unwrap_or(actor_user_id),
            customer_level: non_empty(input.customer_level, "A"),
            customer_status: non_empty(input.customer_status, "跟进中"),
            remark: input.remark.unwrap_or_default(),
        };
        guard.customers.insert(0, created.clone());
        created
    }

    pub async fn delete_customer(&self, id: u64) -> bool {
        let mut guard = self.inner.write().await;
        let before = guard.customers.len();
        guard.customers.retain(|item| item.id != id);
        before != guard.customers.len()
    }

    pub async fn save_mcp_tool(&self, input: crate::models::McpToolUpsertRequest) -> McpToolRecord {
        let mut guard = self.inner.write().await;
        let now = now_ts() * 1000;
        if let Some(existing) = guard
            .mcp_tools
            .iter_mut()
            .find(|item| item.name.eq_ignore_ascii_case(&input.name))
        {
            existing.description = input.description;
            existing.params = sanitize_mcp_params(input.params);
            existing.response = sanitize_mcp_outputs(input.response);
            existing.updated_at = now;
            return existing.clone();
        }

        let next_id = guard
            .mcp_tools
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            + 1;
        let created = McpToolRecord {
            id: next_id,
            created_at: now,
            updated_at: now,
            name: input.name.trim().to_string(),
            description: input.description.trim().to_string(),
            params: sanitize_mcp_params(input.params),
            response: sanitize_mcp_outputs(input.response),
        };
        guard.mcp_tools.insert(0, created.clone());
        created
    }

    pub async fn list_mcp_tools(&self) -> Vec<McpToolDescriptor> {
        let guard = self.inner.read().await;
        guard
            .mcp_tools
            .iter()
            .cloned()
            .map(build_mcp_descriptor)
            .collect()
    }

    pub async fn mcp_service_status(&self) -> McpServiceStatusPayload {
        let guard = self.inner.read().await;
        let state = if guard.mcp_service_running {
            "running"
        } else {
            "stopped"
        };
        McpServiceStatusPayload {
            managed: true,
            state: state.to_string(),
            reachable: guard.mcp_service_running,
            base_url: "http://127.0.0.1:8889/mcp".to_string(),
            health_url: "http://127.0.0.1:8889/healthz".to_string(),
            started_at: guard.mcp_service_started_at,
            last_error: guard.mcp_service_last_error.clone(),
            message: if guard.mcp_service_running {
                format!(
                    "MCP 独立服务已启动，当前注册 {} 个工具",
                    guard.mcp_tools.len()
                )
            } else {
                "MCP 独立服务未启动".to_string()
            },
        }
    }

    pub async fn start_mcp_service(&self) -> McpServiceStatusPayload {
        let mut guard = self.inner.write().await;
        guard.mcp_service_running = true;
        guard.mcp_service_started_at = Some(now_ts() * 1000);
        guard.mcp_service_last_error.clear();
        drop(guard);
        self.mcp_service_status().await
    }

    pub async fn stop_mcp_service(&self) -> McpServiceStatusPayload {
        let mut guard = self.inner.write().await;
        guard.mcp_service_running = false;
        drop(guard);
        self.mcp_service_status().await
    }

    pub async fn test_mcp_tool(
        &self,
        input: crate::models::McpTestRequest,
    ) -> Result<McpTestResult, String> {
        let guard = self.inner.read().await;
        let tool = guard
            .mcp_tools
            .iter()
            .find(|item| item.name == input.name)
            .cloned()
            .ok_or_else(|| "工具不存在".to_string())?;
        let args = match input.args {
            serde_json::Value::Null => serde_json::json!({}),
            other => other,
        };
        let object = args
            .as_object()
            .ok_or_else(|| "args 必须是 JSON 对象".to_string())?;
        let mut missing = vec![];
        for param in tool.params.iter().filter(|item| item.required) {
            if !object.contains_key(&param.name) {
                missing.push(param.name.clone());
            }
        }
        if !missing.is_empty() {
            return Err(format!("缺少必填参数: {}", missing.join(", ")));
        }

        let validation = if object.is_empty() {
            "empty payload"
        } else {
            "schema accepted"
        };
        Ok(McpTestResult {
            tool: tool.name.clone(),
            accepted: true,
            validation: validation.to_string(),
            executed_at: now_ts() * 1000,
            output: serde_json::json!({
                "tool": tool.name,
                "echo": object,
                "responseTypes": tool.response.iter().map(|item| item.type_name.clone()).collect::<Vec<_>>(),
                "serverState": if guard.mcp_service_running { "running" } else { "stopped" },
            }),
        })
    }

    pub async fn list_skill_tools(&self) -> Vec<SkillToolDefinition> {
        vec![
            SkillToolDefinition {
                key: "shell".to_string(),
                label: "Shell".to_string(),
                summary: "执行本地命令、脚本和校验流程".to_string(),
            },
            SkillToolDefinition {
                key: "filesystem".to_string(),
                label: "Filesystem".to_string(),
                summary: "读取、写入和组织工作区文件".to_string(),
            },
            SkillToolDefinition {
                key: "http".to_string(),
                label: "HTTP".to_string(),
                summary: "拉取外部文档、状态页和在线资产".to_string(),
            },
        ]
    }

    pub async fn list_skills(&self) -> Vec<SkillSummary> {
        let guard = self.inner.read().await;
        guard
            .skills
            .iter()
            .map(|item| SkillSummary {
                name: item.name.clone(),
                description: item.description.clone(),
                enabled: item.enabled,
                tags: item.tags.clone(),
                updated_at: item.updated_at,
            })
            .collect()
    }

    pub async fn get_skill_detail(&self, name: &str) -> Option<SkillRecord> {
        let guard = self.inner.read().await;
        guard.skills.iter().find(|item| item.name == name).cloned()
    }

    pub async fn save_skill(&self, input: crate::models::SkillSaveRequest) -> SkillRecord {
        let mut guard = self.inner.write().await;
        let now = now_ts() * 1000;
        if let Some(existing) = guard.skills.iter_mut().find(|item| item.name == input.name) {
            existing.description = input.description;
            existing.allowed_tools = input.allowed_tools;
            existing.context = input.context;
            existing.agent = input.agent;
            existing.markdown = input.markdown;
            existing.enabled = input.enabled;
            existing.tags = sanitize_tags(input.tags);
            existing.updated_at = now;
            return existing.clone();
        }

        let created = SkillRecord {
            name: input.name,
            description: input.description,
            allowed_tools: input.allowed_tools,
            context: input.context,
            agent: input.agent,
            markdown: input.markdown,
            enabled: input.enabled,
            tags: sanitize_tags(input.tags),
            created_at: now,
            updated_at: now,
            scripts: vec![],
            resources: vec![],
            references: vec![],
            templates: vec![],
        };
        guard.skills.insert(0, created.clone());
        created
    }

    pub async fn delete_skill(&self, name: &str) -> bool {
        let mut guard = self.inner.write().await;
        let before = guard.skills.len();
        guard.skills.retain(|item| item.name != name);
        before != guard.skills.len()
    }

    pub async fn create_skill_asset(
        &self,
        skill_name: &str,
        kind: &str,
        name: &str,
    ) -> Result<SkillAssetRecord, String> {
        let mut guard = self.inner.write().await;
        let skill = guard
            .skills
            .iter_mut()
            .find(|item| item.name == skill_name)
            .ok_or_else(|| "技能不存在".to_string())?;
        let now = now_ts() * 1000;
        let target = select_skill_assets(skill, kind)?;
        if target.iter().any(|item| item.name == name) {
            return Err("文件已存在".to_string());
        }
        let file = SkillAssetRecord {
            name: name.to_string(),
            content: String::new(),
            updated_at: now,
        };
        target.push(file.clone());
        skill.updated_at = now;
        Ok(file)
    }

    pub async fn get_skill_asset(
        &self,
        skill_name: &str,
        kind: &str,
        name: &str,
    ) -> Result<SkillAssetRecord, String> {
        let guard = self.inner.read().await;
        let skill = guard
            .skills
            .iter()
            .find(|item| item.name == skill_name)
            .ok_or_else(|| "技能不存在".to_string())?;
        select_skill_assets_readonly(skill, kind)?
            .iter()
            .find(|item| item.name == name)
            .cloned()
            .ok_or_else(|| "文件不存在".to_string())
    }

    pub async fn save_skill_asset(
        &self,
        skill_name: &str,
        kind: &str,
        name: &str,
        content: String,
    ) -> Result<SkillAssetRecord, String> {
        let mut guard = self.inner.write().await;
        let skill = guard
            .skills
            .iter_mut()
            .find(|item| item.name == skill_name)
            .ok_or_else(|| "技能不存在".to_string())?;
        let now = now_ts() * 1000;
        let updated = {
            let target = select_skill_assets(skill, kind)?;
            if let Some(existing) = target.iter_mut().find(|item| item.name == name) {
                existing.content = content.clone();
                existing.updated_at = now;
                Some(existing.clone())
            } else {
                None
            }
        };
        if let Some(existing) = updated {
            skill.updated_at = now;
            return Ok(existing);
        }
        let created = SkillAssetRecord {
            name: name.to_string(),
            content,
            updated_at: now,
        };
        let target = select_skill_assets(skill, kind)?;
        target.push(created.clone());
        skill.updated_at = now;
        Ok(created)
    }

    pub async fn get_global_constraint(&self) -> GlobalConstraintRecord {
        let guard = self.inner.read().await;
        guard.global_constraint.clone()
    }

    pub async fn save_global_constraint(&self, content: String) -> GlobalConstraintRecord {
        let mut guard = self.inner.write().await;
        guard.global_constraint = GlobalConstraintRecord {
            content,
            updated_at: now_ts() * 1000,
        };
        guard.global_constraint.clone()
    }

    pub fn list_operation_logs(&self) -> PageResult<OperationLogInfo> {
        page_result(
            vec![
                OperationLogInfo {
                    id: 1,
                    ip: "127.0.0.1".to_string(),
                    method: "POST".to_string(),
                    path: "/base/login".to_string(),
                    status: 200,
                    latency: 12,
                },
                OperationLogInfo {
                    id: 2,
                    ip: "127.0.0.1".to_string(),
                    method: "POST".to_string(),
                    path: "/ai/moderation/decision".to_string(),
                    status: 200,
                    latency: 26,
                },
            ],
            Some(1),
            Some(50),
        )
    }

    pub fn list_login_logs(&self) -> PageResult<LoginLogInfo> {
        page_result(
            vec![
                LoginLogInfo {
                    id: 1,
                    username: "admin".to_string(),
                    ip: "127.0.0.1".to_string(),
                    status: true,
                    error_message: "登录成功".to_string(),
                },
                LoginLogInfo {
                    id: 2,
                    username: "disabled".to_string(),
                    ip: "127.0.0.1".to_string(),
                    status: false,
                    error_message: "用户被禁止登录".to_string(),
                },
            ],
            Some(1),
            Some(50),
        )
    }

    pub async fn dictionary_detail_tree(&self, dictionary_id: u64) -> Vec<DictionaryDetailInfo> {
        let guard = self.inner.read().await;
        guard
            .dictionary_details
            .get(&dictionary_id)
            .cloned()
            .unwrap_or_default()
    }

    pub async fn upsert_dictionary_detail(
        &self,
        input: crate::models::DictionaryDetailUpsertRequest,
    ) -> DictionaryDetailInfo {
        let mut guard = self.inner.write().await;
        let roots = guard
            .dictionary_details
            .entry(input.sys_dictionary_id)
            .or_insert_with(Vec::new);

        let mut flat = flatten_dictionary_nodes(roots);
        let next_id = flat.iter().map(|item| item.id).max().unwrap_or(0) + 1;
        let node_id = input.id.unwrap_or(next_id);

        if let Some(existing) = flat.iter_mut().find(|item| item.id == node_id) {
            existing.label = input.label;
            existing.value = input.value;
            existing.extend = input.extend;
            existing.level = input.level.max(1);
            existing.status = input.status;
            existing.sort = input.sort;
            existing.parent_id = input.parent_id;
        } else {
            flat.push(DictionaryDetailInfo {
                id: node_id,
                label: input.label,
                value: input.value,
                extend: input.extend,
                level: input.level.max(1),
                status: input.status,
                sort: input.sort,
                parent_id: input.parent_id,
                children: vec![],
            });
        }

        let target = flat
            .iter()
            .find(|item| item.id == node_id)
            .cloned()
            .unwrap_or(DictionaryDetailInfo {
                id: node_id,
                label: String::new(),
                value: String::new(),
                extend: String::new(),
                level: 1,
                status: true,
                sort: 1,
                parent_id: None,
                children: vec![],
            });

        *roots = rebuild_dictionary_tree(flat);
        target
    }

    pub fn list_error_logs(&self) -> PageResult<ErrorLogInfo> {
        page_result(
            vec![
                ErrorLogInfo {
                    id: 1,
                    error: "权限包刷新失败".to_string(),
                    path: "/casbin/getPolicyPathByAuthorityId".to_string(),
                    status: "已处理".to_string(),
                },
                ErrorLogInfo {
                    id: 2,
                    error: "导出模板配置缺失映射".to_string(),
                    path: "/sysExportTemplate/getSysExportTemplateList".to_string(),
                    status: "待处理".to_string(),
                },
            ],
            Some(1),
            Some(50),
        )
    }

    pub fn list_export_templates(&self) -> PageResult<ExportTemplateInfo> {
        page_result(
            vec![
                ExportTemplateInfo {
                    id: 1,
                    name: "接口导出模板".to_string(),
                    template_id: "api".to_string(),
                    desc: "接口列表导出模板".to_string(),
                },
                ExportTemplateInfo {
                    id: 2,
                    name: "用户导出模板".to_string(),
                    template_id: "user".to_string(),
                    desc: "用户列表导出模板".to_string(),
                },
            ],
            Some(1),
            Some(50),
        )
    }

    pub fn runtime_info(&self) -> RuntimeInfo {
        RuntimeInfo {
            os: std::env::consts::OS.to_string(),
            cpu_cores: std::thread::available_parallelism()
                .map(|value| value.get())
                .unwrap_or(1),
            rust_version: env!("CARGO_PKG_VERSION").to_string(),
            db_backend: std::env::var("GAA_DATABASE_URL")
                .ok()
                .filter(|value| !value.is_empty())
                .map(|value| {
                    if value.starts_with("postgres://") || value.starts_with("postgresql://") {
                        "postgres".to_string()
                    } else {
                        "sqlite".to_string()
                    }
                })
                .unwrap_or_else(|| "sqlite".to_string()),
            redis_enabled: std::env::var("GAA_REDIS_URL")
                .map(|value| !value.is_empty())
                .unwrap_or(false),
        }
    }

    pub async fn system_config_info(&self) -> SystemConfigInfo {
        let guard = self.inner.read().await;
        redact_system_config(guard.system_config.clone())
    }
}

fn redact_system_config(mut config: SystemConfigInfo) -> SystemConfigInfo {
    config.database_url = redact_url_secret(&config.database_url);
    config.redis_url = redact_url_secret(&config.redis_url);
    config
}

fn redact_url_secret(url: &str) -> String {
    let Some(scheme_end) = url.find("://") else {
        return url.to_string();
    };
    let credentials_start = scheme_end + 3;
    let Some(at_offset) = url[credentials_start..].find('@') else {
        return url.to_string();
    };
    let at_index = credentials_start + at_offset;
    let credentials = &url[credentials_start..at_index];
    let Some(colon_offset) = credentials.rfind(':') else {
        return url.to_string();
    };
    let password_start = credentials_start + colon_offset + 1;
    format!("{}****{}", &url[..password_start], &url[at_index..])
}

fn read_service_token() -> Result<String, String> {
    match std::env::var("GAA_SERVICE_MODERATION_TOKEN") {
        Ok(value) if is_strong_runtime_secret(&value) => Ok(value),
        Ok(_) if is_production_env() => Err(
            "GAA_SERVICE_MODERATION_TOKEN must be at least 32 characters in production".to_string(),
        ),
        Ok(value) => Ok(value),
        Err(_) if is_production_env() => {
            Err("GAA_SERVICE_MODERATION_TOKEN must be set in production".to_string())
        }
        Err(_) => Ok("svc-moderation-token".to_string()),
    }
}

fn read_bootstrap_admin_password() -> Result<String, String> {
    match std::env::var("GAA_BOOTSTRAP_ADMIN_PASSWORD") {
        Ok(value) if is_strong_password(&value) => Ok(value),
        Ok(_) if is_production_env() => Err(
            "GAA_BOOTSTRAP_ADMIN_PASSWORD must be at least 12 characters in production".to_string(),
        ),
        Ok(value) => Ok(value),
        Err(_) if is_production_env() => {
            Err("GAA_BOOTSTRAP_ADMIN_PASSWORD must be set in production".to_string())
        }
        Err(_) => Ok("123456".to_string()),
    }
}

fn read_default_user_password() -> Result<String, String> {
    match std::env::var("GAA_DEFAULT_USER_PASSWORD") {
        Ok(value) if is_strong_password(&value) => Ok(value),
        Ok(_) if is_production_env() => Err(
            "GAA_DEFAULT_USER_PASSWORD must be at least 12 characters in production".to_string(),
        ),
        Ok(value) => Ok(value),
        Err(_) if is_production_env() => {
            Err("GAA_DEFAULT_USER_PASSWORD must be set in production".to_string())
        }
        Err(_) => Ok("123456".to_string()),
    }
}

fn is_strong_runtime_secret(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.len() >= 32 && trimmed != "svc-moderation-token"
}

fn is_strong_password(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.len() >= 12 && trimmed != "123456"
}

fn is_production_env() -> bool {
    std::env::var("GAA_ENV")
        .or_else(|_| std::env::var("RUST_ENV"))
        .or_else(|_| std::env::var("APP_ENV"))
        .map(|value| matches!(value.to_ascii_lowercase().as_str(), "prod" | "production"))
        .unwrap_or(false)
}

fn build_active_sessions_index(sessions: &[SessionRecord]) -> HashMap<u64, String> {
    let mut active_sessions = HashMap::new();
    let now = now_ts();
    for session in sessions
        .iter()
        .filter(|session| !session.revoked && session.refresh_expires_at > now)
    {
        active_sessions
            .entry(session.user_id)
            .and_modify(|current_session_id| {
                let current = sessions
                    .iter()
                    .find(|candidate| candidate.session_id == *current_session_id)
                    .map(|candidate| candidate.refresh_expires_at)
                    .unwrap_or_default();
                if session.refresh_expires_at >= current {
                    *current_session_id = session.session_id.clone();
                }
            })
            .or_insert_with(|| session.session_id.clone());
    }
    active_sessions
}

fn seeded_dictionary_details() -> HashMap<u64, Vec<DictionaryDetailInfo>> {
    HashMap::from([
        (
            1,
            vec![
                DictionaryDetailInfo {
                    id: 11,
                    label: "待审核".to_string(),
                    value: "pending".to_string(),
                    extend: "neutral".to_string(),
                    level: 1,
                    status: true,
                    sort: 1,
                    parent_id: None,
                    children: vec![],
                },
                DictionaryDetailInfo {
                    id: 12,
                    label: "已通过".to_string(),
                    value: "approved".to_string(),
                    extend: "positive".to_string(),
                    level: 1,
                    status: true,
                    sort: 2,
                    parent_id: None,
                    children: vec![],
                },
                DictionaryDetailInfo {
                    id: 13,
                    label: "已拒绝".to_string(),
                    value: "rejected".to_string(),
                    extend: "negative".to_string(),
                    level: 1,
                    status: true,
                    sort: 3,
                    parent_id: None,
                    children: vec![],
                },
            ],
        ),
        (
            2,
            vec![
                DictionaryDetailInfo {
                    id: 21,
                    label: "轻度违规".to_string(),
                    value: "minor".to_string(),
                    extend: "warning".to_string(),
                    level: 1,
                    status: true,
                    sort: 1,
                    parent_id: None,
                    children: vec![DictionaryDetailInfo {
                        id: 22,
                        label: "高频访问".to_string(),
                        value: "burst_access".to_string(),
                        extend: "crawler".to_string(),
                        level: 2,
                        status: true,
                        sort: 1,
                        parent_id: Some(21),
                        children: vec![],
                    }],
                },
                DictionaryDetailInfo {
                    id: 23,
                    label: "严重违规".to_string(),
                    value: "severe".to_string(),
                    extend: "ban".to_string(),
                    level: 1,
                    status: true,
                    sort: 2,
                    parent_id: None,
                    children: vec![],
                },
            ],
        ),
    ])
}

fn seeded_authorities() -> Vec<AuthorityInfo> {
    vec![
        AuthorityInfo {
            id: 1,
            authority_id: 888,
            authority_name: "管理员".to_string(),
            default_router: "dashboard".to_string(),
            parent_id: 0,
            enable: 1,
            status: 1,
            children: vec![],
        },
        AuthorityInfo {
            id: 2,
            authority_id: 9528,
            authority_name: "AI运营角色".to_string(),
            default_router: "dashboard".to_string(),
            parent_id: 0,
            enable: 1,
            status: 1,
            children: vec![AuthorityInfo {
                id: 3,
                authority_id: 9529,
                authority_name: "AI审核子角色".to_string(),
                default_router: "authorities".to_string(),
                parent_id: 9528,
                enable: 1,
                status: 1,
                children: vec![],
            }],
        },
    ]
}

fn flatten_authorities(items: &[AuthorityInfo]) -> Vec<AuthorityInfo> {
    items
        .iter()
        .flat_map(|item| {
            let mut cloned = item.clone();
            cloned.children = vec![];
            std::iter::once(cloned)
                .chain(flatten_authorities(&item.children))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn rebuild_authority_tree(mut items: Vec<AuthorityInfo>) -> Vec<AuthorityInfo> {
    items.sort_by_key(|item| (item.parent_id, item.authority_id));
    let mut by_parent: HashMap<u32, Vec<AuthorityInfo>> = HashMap::new();
    for mut item in items {
        item.children = vec![];
        by_parent.entry(item.parent_id).or_default().push(item);
    }

    fn attach(
        parent_id: u32,
        by_parent: &mut HashMap<u32, Vec<AuthorityInfo>>,
    ) -> Vec<AuthorityInfo> {
        let mut children = by_parent.remove(&parent_id).unwrap_or_default();
        children.sort_by_key(|item| item.authority_id);
        for child in &mut children {
            child.children = attach(child.authority_id, by_parent);
        }
        children
    }

    attach(0, &mut by_parent)
}

fn seeded_menu_tree() -> Vec<MenuInfo> {
    vec![
        menu_node(
            1,
            0,
            "dashboard",
            "/dashboard",
            "views/DashboardView.vue",
            "仪表盘",
            "layout-dashboard",
            vec![],
        ),
        menu_node(
            3,
            0,
            "profile",
            "/profile",
            "views/ProfileView.vue",
            "个人中心",
            "user-circle",
            vec![],
        ),
        menu_node(
            4,
            0,
            "about",
            "/about",
            "views/AboutView.vue",
            "关于系统",
            "info",
            vec![],
        ),
        menu_node(
            5,
            0,
            "examplesRoot",
            "/examples",
            "views/examples/ExamplesView.vue",
            "示例中心",
            "flask-conical",
            vec![
                menu_node(
                    51,
                    5,
                    "customerExample",
                    "/examples/customer",
                    "views/examples/CustomerExampleView.vue",
                    "客户示例",
                    "briefcase",
                    vec![],
                ),
                menu_node(
                    52,
                    5,
                    "uploadExample",
                    "/examples/upload",
                    "views/examples/UploadExampleView.vue",
                    "上传示例",
                    "upload",
                    vec![],
                ),
                menu_node(
                    53,
                    5,
                    "breakpointExample",
                    "/examples/breakpoint",
                    "views/examples/BreakpointExampleView.vue",
                    "断点续传示例",
                    "pause-circle",
                    vec![],
                ),
            ],
        ),
        menu_node(
            2,
            0,
            "system-root",
            "/system",
            "views/shell/PlaceholderView.vue",
            "系统管理",
            "layout-grid",
            vec![
                menu_node(
                    20,
                    2,
                    "systemOverview",
                    "/system/overview",
                    "views/admin/SystemOverviewView.vue",
                    "管理总览",
                    "layout-grid",
                    vec![],
                ),
                menu_node(
                    21,
                    2,
                    "users",
                    "/system/users",
                    "views/admin/UsersView.vue",
                    "用户管理",
                    "user",
                    vec![],
                ),
                menu_node(
                    22,
                    2,
                    "authorities",
                    "/system/authorities",
                    "views/admin/AuthoritiesView.vue",
                    "角色管理",
                    "shield",
                    vec![],
                ),
                menu_node(
                    23,
                    2,
                    "menus",
                    "/system/menus",
                    "views/admin/MenusView.vue",
                    "菜单管理",
                    "menu",
                    vec![],
                ),
                menu_node(
                    24,
                    2,
                    "apis",
                    "/system/apis",
                    "views/admin/ApisView.vue",
                    "接口管理",
                    "plug",
                    vec![],
                ),
                menu_node(
                    245,
                    2,
                    "menuIcons",
                    "/system/icons",
                    "views/admin/IconGalleryView.vue",
                    "菜单图标",
                    "image",
                    vec![],
                ),
                menu_node(
                    25,
                    2,
                    "dictionaries",
                    "/system/dictionaries",
                    "views/admin/DictionariesView.vue",
                    "字典管理",
                    "book",
                    vec![],
                ),
                menu_node(
                    26,
                    2,
                    "params",
                    "/system/params",
                    "views/admin/ParamsView.vue",
                    "参数管理",
                    "sliders",
                    vec![],
                ),
                menu_node(
                    27,
                    2,
                    "operationLogs",
                    "/system/operation-logs",
                    "views/admin/OperationLogsView.vue",
                    "操作日志",
                    "file-text",
                    vec![],
                ),
                menu_node(
                    28,
                    2,
                    "loginLogs",
                    "/system/login-logs",
                    "views/admin/LoginLogsView.vue",
                    "登录日志",
                    "history",
                    vec![],
                ),
                menu_node(
                    29,
                    2,
                    "systemTools",
                    "/system/tools",
                    "views/admin/SystemToolsView.vue",
                    "系统工具",
                    "tool",
                    vec![
                        menu_node(
                            291,
                            29,
                            "aiWorkflow",
                            "/system/tools/ai-workflow",
                            "views/tools/AiWorkflowView.vue",
                            "AI 工作流",
                            "sparkles",
                            vec![],
                        ),
                        menu_node(
                            292,
                            29,
                            "apiTokens",
                            "/system/tools/api-tokens",
                            "views/tools/ApiTokensView.vue",
                            "API Token",
                            "key",
                            vec![],
                        ),
                        menu_node(
                            293,
                            29,
                            "llmConfig",
                            "/system/tools/llm-config",
                            "views/tools/LlmConfigView.vue",
                            "大模型配置",
                            "bot",
                            vec![],
                        ),
                        menu_node(
                            294,
                            29,
                            "skills",
                            "/system/tools/skills",
                            "views/tools/SkillsView.vue",
                            "技能管理",
                            "layers",
                            vec![],
                        ),
                        menu_node(
                            295,
                            29,
                            "systemConfig",
                            "/system/tools/config",
                            "views/tools/SystemConfigView.vue",
                            "系统配置",
                            "settings",
                            vec![],
                        ),
                        menu_node(
                            296,
                            29,
                            "pluginEmail",
                            "/system/tools/plugin-email",
                            "views/tools/EmailPluginView.vue",
                            "邮件管理",
                            "message-square",
                            vec![],
                        ),
                        menu_node(
                            297,
                            29,
                            "announcementInfo",
                            "/system/tools/announcement",
                            "views/tools/AnnouncementView.vue",
                            "公告管理",
                            "megaphone",
                            vec![],
                        ),
                    ],
                ),
                menu_node(
                    30,
                    2,
                    "systemState",
                    "/system/state",
                    "views/admin/SystemStateView.vue",
                    "系统状态",
                    "server",
                    vec![],
                ),
            ],
        ),
        menu_node(
            6,
            0,
            "frontend-root",
            "/frontend",
            "views/shell/PlaceholderView.vue",
            "前台管理",
            "monitor",
            vec![
                menu_node(
                    60,
                    6,
                    "frontendSettings",
                    "/frontend/settings",
                    "views/frontend/FrontendSettingsView.vue",
                    "前台设置",
                    "settings",
                    vec![],
                ),
                menu_node(
                    61,
                    6,
                    "frontendNav",
                    "/frontend/nav",
                    "views/frontend/FrontendNavView.vue",
                    "前端导航管理",
                    "navigation",
                    vec![],
                ),
                menu_node(
                    62,
                    6,
                    "articleCategories",
                    "/frontend/article-categories",
                    "views/frontend/ArticleCategoriesView.vue",
                    "文章分类",
                    "folder-tree",
                    vec![],
                ),
                menu_node(
                    63,
                    6,
                    "articles",
                    "/frontend/articles",
                    "views/frontend/ArticlesView.vue",
                    "文章管理",
                    "file-text",
                    vec![],
                ),
                menu_node(
                    64,
                    6,
                    "members",
                    "/frontend/members",
                    "views/frontend/MembersView.vue",
                    "会员管理",
                    "users",
                    vec![],
                ),
                menu_node(
                    65,
                    6,
                    "frontendConsoleMenus",
                    "/frontend/console-menus",
                    "views/frontend/ConsoleMenusView.vue",
                    "前端控制台菜单",
                    "panel-left",
                    vec![],
                ),
            ],
        ),
    ]
}

fn seeded_authority_menu_ids(menus: &[MenuInfo]) -> HashMap<u32, Vec<u64>> {
    let all_menu_ids = dedupe_u64(
        flatten_menus_for_state(menus)
            .into_iter()
            .map(|menu| menu.id)
            .collect(),
    );
    HashMap::from([
        (888, all_menu_ids.clone()),
        (
            9528,
            vec![
                1, 2, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 6, 60, 61, 62, 63, 64, 65,
            ],
        ),
        (9529, vec![1, 2, 22]),
    ])
}

fn seeded_authority_button_ids(
    menus: &[MenuInfo],
    authority_menu_ids: &HashMap<u32, Vec<u64>>,
) -> HashMap<(u32, u64), Vec<u64>> {
    let flat_menus = flatten_menus_for_state(menus);
    let mut out = HashMap::new();
    for (authority_id, menu_ids) in authority_menu_ids {
        for menu_id in menu_ids {
            let buttons = flat_menus
                .iter()
                .find(|menu| menu.id == *menu_id)
                .map(|menu| {
                    menu.menu_btn
                        .iter()
                        .map(|button| button.id)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            out.insert((*authority_id, *menu_id), buttons);
        }
    }
    out
}

fn seeded_apis() -> Vec<ApiInfo> {
    vec![
        ApiInfo {
            id: 1,
            path: "/base/login".to_string(),
            api_group: "鉴权".to_string(),
            description: "用户登录".to_string(),
            method: "POST".to_string(),
        },
        ApiInfo {
            id: 2,
            path: "/user/getUserInfo".to_string(),
            api_group: "用户".to_string(),
            description: "获取用户信息".to_string(),
            method: "GET".to_string(),
        },
        ApiInfo {
            id: 3,
            path: "/user/getUserList".to_string(),
            api_group: "用户".to_string(),
            description: "分页获取用户列表".to_string(),
            method: "POST".to_string(),
        },
        ApiInfo {
            id: 4,
            path: "/authority/getAuthorityList".to_string(),
            api_group: "角色".to_string(),
            description: "获取角色树".to_string(),
            method: "POST".to_string(),
        },
        ApiInfo {
            id: 5,
            path: "/menu/getMenu".to_string(),
            api_group: "菜单".to_string(),
            description: "获取动态菜单".to_string(),
            method: "POST".to_string(),
        },
        ApiInfo {
            id: 6,
            path: "/api/getApiList".to_string(),
            api_group: "接口".to_string(),
            description: "分页获取接口列表".to_string(),
            method: "POST".to_string(),
        },
        ApiInfo {
            id: 7,
            path: "/email/emailTest".to_string(),
            api_group: "邮件".to_string(),
            description: "发送测试邮件".to_string(),
            method: "POST".to_string(),
        },
        ApiInfo {
            id: 8,
            path: "/email/sendEmail".to_string(),
            api_group: "邮件".to_string(),
            description: "发送邮件".to_string(),
            method: "POST".to_string(),
        },
        ApiInfo {
            id: 9,
            path: "/info/getInfoList".to_string(),
            api_group: "公告".to_string(),
            description: "获取公告列表".to_string(),
            method: "GET".to_string(),
        },
        ApiInfo {
            id: 10,
            path: "/info/createInfo".to_string(),
            api_group: "公告".to_string(),
            description: "创建公告".to_string(),
            method: "POST".to_string(),
        },
    ]
}

fn seeded_dictionaries() -> Vec<DictionaryInfo> {
    vec![
        DictionaryInfo {
            id: 1,
            name: "内容状态".to_string(),
            type_name: "content_status".to_string(),
            status: true,
            desc: "内容审核状态字典".to_string(),
        },
        DictionaryInfo {
            id: 2,
            name: "违规类型".to_string(),
            type_name: "violation_type".to_string(),
            status: true,
            desc: "用户和爬虫违规场景字典".to_string(),
        },
    ]
}

fn seeded_system_config(
    compatibility_refresh_headers: bool,
    multipoint_enabled: bool,
) -> SystemConfigInfo {
    SystemConfigInfo {
        bind_address: std::env::var("GAA_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8888".to_string()),
        database_url: std::env::var("GAA_DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://gaa_auth.db".to_string()),
        redis_url: std::env::var("GAA_REDIS_URL").unwrap_or_default(),
        multipoint_enabled,
        compatibility_refresh_headers,
    }
}

fn seeded_api_tokens() -> Vec<ApiTokenRecord> {
    vec![ApiTokenRecord {
        id: 1,
        name: "moderation-service".to_string(),
        scope: "article.review".to_string(),
        ttl: "24h".to_string(),
        status: "已登记".to_string(),
    }]
}

fn seeded_packages() -> Vec<PackageRecord> {
    vec![PackageRecord {
        id: 1,
        name: "feature-bundle".to_string(),
        kind: "package".to_string(),
        output: "/modules/feature-bundle".to_string(),
        summary: "用户列表、详情表单、角色切换、导出模板，以及开发完成后的 Tauri 2 桌面端重构。"
            .to_string(),
    }]
}

fn seeded_plugin_manifests() -> Vec<PluginManifestRecord> {
    vec![PluginManifestRecord {
        id: 1,
        plugin_name: "ops-toolkit".to_string(),
        menu_group: "运维工具".to_string(),
        menu_ids: vec![29],
        api_ids: vec![6],
        dictionary_ids: vec![1],
        saved_at: now_ts() * 1000,
    }]
}

fn seeded_plugin_installs() -> Vec<PluginInstallRecord> {
    vec![]
}

fn seeded_auto_code_registry() -> Vec<AutoCodeRegistryRecord> {
    vec![]
}

fn seeded_releases() -> Vec<ReleaseRecord> {
    vec![ReleaseRecord {
        id: 1,
        created_at: now_ts() * 1000,
        note: "记录当前前后端页面与权限恢复批次。".to_string(),
    }]
}

fn seeded_email_records() -> Vec<EmailRecord> {
    vec![EmailRecord {
        id: 1,
        to: "ops-team@gaa.local".to_string(),
        subject: "GAA 邮件测试".to_string(),
        body: "当前邮件由 Rust 重构版邮件工作台登记。".to_string(),
        mode: "test".to_string(),
        status: "测试已发送".to_string(),
        created_at: now_ts() * 1000,
    }]
}

fn seeded_announcements() -> Vec<AnnouncementRecord> {
    vec![AnnouncementRecord {
        id: 1,
        created_at: now_ts() * 1000,
        updated_at: now_ts() * 1000,
        title: "重构版验收通知".to_string(),
        content: "当前 Rust / Vben 重构版已进入页面级验收阶段，请按恢复矩阵继续核对。".to_string(),
        user_id: 1,
        attachments: vec![AnnouncementAttachment {
            name: "page-parity-matrix.md".to_string(),
            url: "/docs/page-parity-matrix.md".to_string(),
        }],
    }]
}

fn seeded_llm_configs() -> Vec<LlmConfigRecord> {
    vec![
        LlmConfigRecord {
            id: 1,
            provider: "openai".to_string(),
            model: "gpt-4.1".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: "sk-xxx".to_string(),
            status: "enabled".to_string(),
        },
        LlmConfigRecord {
            id: 2,
            provider: "qwen".to_string(),
            model: "qwen-max".to_string(),
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            api_key: "qwen-xxx".to_string(),
            status: "enabled".to_string(),
        },
    ]
}

fn seeded_frontend_nav() -> Vec<FrontendNavRecord> {
    vec![
        FrontendNavRecord {
            id: 1,
            title: "首页".to_string(),
            path: "/".to_string(),
            icon: "lucide:home".to_string(),
            order: 1,
            visible: true,
        },
        FrontendNavRecord {
            id: 2,
            title: "文章".to_string(),
            path: "/articles".to_string(),
            icon: "lucide:file-text".to_string(),
            order: 2,
            visible: true,
        },
    ]
}

fn seeded_frontend_settings() -> FrontendSettingsRecord {
    FrontendSettingsRecord {
        logo_url: String::new(),
        site_name: "即梦工作台".to_string(),
        site_slogan: "开启你的 Agent 模式，即刻造梦！".to_string(),
        site_description: "面向前台用户的 AI 创作首页、会员入口和内容展示配置。".to_string(),
        record_number: "沪ICP备00000000号-1".to_string(),
        smtp_host: "smtp.example.com".to_string(),
        smtp_port: 465,
        smtp_user: "notice@example.com".to_string(),
        smtp_password: String::new(),
        mail_from: "即梦工作台 <notice@example.com>".to_string(),
    }
}

fn seeded_article_categories() -> Vec<ArticleCategoryRecord> {
    vec![
        ArticleCategoryRecord {
            id: 1,
            name: "公告".to_string(),
            slug: "notice".to_string(),
            sort: 1,
            status: true,
        },
        ArticleCategoryRecord {
            id: 2,
            name: "教程".to_string(),
            slug: "guide".to_string(),
            sort: 2,
            status: true,
        },
    ]
}

fn seeded_articles() -> Vec<ArticleRecord> {
    vec![ArticleRecord {
        id: 1,
        category_id: 1,
        title: "前台内容系统初始化".to_string(),
        slug: "frontend-content-bootstrap".to_string(),
        content: "# 前台内容系统初始化\n\n这里可以用 Markdown 管理前台文章、公告和教程。"
            .to_string(),
        status: "draft".to_string(),
        updated_at: "2026/04/22 09:30:00".to_string(),
    }]
}

fn seeded_members() -> Vec<MemberRecord> {
    vec![
        MemberRecord {
            id: 1,
            email: "member@example.com".to_string(),
            nickname: "演示会员".to_string(),
            provider: "email".to_string(),
            status: "active".to_string(),
            last_login: "2026/04/22 09:30:00".to_string(),
            console_path: "/member/profile".to_string(),
        },
        MemberRecord {
            id: 2,
            email: "google-user@example.com".to_string(),
            nickname: "Google 用户".to_string(),
            provider: "google".to_string(),
            status: "pending".to_string(),
            last_login: "未登录".to_string(),
            console_path: "/member/dashboard".to_string(),
        },
    ]
}

fn seeded_console_menus() -> Vec<ConsoleMenuRecord> {
    vec![
        ConsoleMenuRecord {
            id: 1,
            title: "信息修改".to_string(),
            group: "账户".to_string(),
            path: "/member/profile".to_string(),
            icon: "lucide:user-cog".to_string(),
            order: 1,
            visible: true,
        },
        ConsoleMenuRecord {
            id: 2,
            title: "安全设置".to_string(),
            group: "账户".to_string(),
            path: "/member/security".to_string(),
            icon: "lucide:shield-check".to_string(),
            order: 2,
            visible: true,
        },
    ]
}

fn seeded_resume_uploads() -> Vec<ResumeUploadRecord> {
    vec![
        ResumeUploadRecord {
            id: 1,
            name: "archive-2026-04.zip".to_string(),
            progress: 68,
            status: "上传中".to_string(),
        },
        ResumeUploadRecord {
            id: 2,
            name: "snapshot-logs.tar".to_string(),
            progress: 100,
            status: "已完成".to_string(),
        },
    ]
}

fn seeded_customers() -> Vec<CustomerRecord> {
    let now = now_ts() * 1000;
    vec![
        CustomerRecord {
            id: 1,
            created_at: now,
            updated_at: now,
            customer_name: "澄岳物流".to_string(),
            customer_phone_data: "021-60881234".to_string(),
            sys_user_id: 1,
            customer_level: "A".to_string(),
            customer_status: "已签约".to_string(),
            remark: "重点关注交付时效看板。".to_string(),
        },
        CustomerRecord {
            id: 2,
            created_at: now,
            updated_at: now,
            customer_name: "栖光制造".to_string(),
            customer_phone_data: "0755-33117788".to_string(),
            sys_user_id: 1,
            customer_level: "B".to_string(),
            customer_status: "跟进中".to_string(),
            remark: "正在确认质检工作流联调窗口。".to_string(),
        },
    ]
}

fn seeded_mcp_tools() -> Vec<McpToolRecord> {
    let now = now_ts() * 1000;
    vec![McpToolRecord {
        id: 1,
        created_at: now,
        updated_at: now,
        name: "CurrentTime".to_string(),
        description: "返回当前时区时间，用于界面打点与任务追踪。".to_string(),
        params: vec![crate::models::McpToolParam {
            name: "timezone".to_string(),
            description: "IANA 时区名".to_string(),
            type_name: "string".to_string(),
            required: true,
            default_value: Some("Asia/Shanghai".to_string()),
        }],
        response: vec![crate::models::McpToolOutput {
            type_name: "text".to_string(),
        }],
    }]
}

fn seeded_skills() -> Vec<SkillRecord> {
    let now = now_ts() * 1000;
    vec![
        SkillRecord {
            name: "parity-restore".to_string(),
            description: "整理页面恢复差异、补齐缺失流程并记录验收结论。".to_string(),
            allowed_tools: "filesystem,shell,http".to_string(),
            context: "fork".to_string(),
            agent: "executor".to_string(),
            markdown: "## Instructions\n- 先定位未恢复页面\n- 再补齐可验证流程\n".to_string(),
            enabled: true,
            tags: vec!["parity".to_string(), "recovery".to_string()],
            created_at: now,
            updated_at: now,
            scripts: vec![SkillAssetRecord {
                name: "check-gap.sh".to_string(),
                content: "echo \"scan gaps\"".to_string(),
                updated_at: now,
            }],
            resources: vec![SkillAssetRecord {
                name: "acceptance.md".to_string(),
                content: "记录页面、接口、数据流的对齐标准。".to_string(),
                updated_at: now,
            }],
            references: vec![],
            templates: vec![SkillAssetRecord {
                name: "handoff.md".to_string(),
                content: "## 交付摘要\n- 变更\n- 验证\n- 风险\n".to_string(),
                updated_at: now,
            }],
        },
        SkillRecord {
            name: "ops-audit".to_string(),
            description: "核对部署参数、运行状态、MCP 服务和插件资源装配。".to_string(),
            allowed_tools: "filesystem,http".to_string(),
            context: String::new(),
            agent: String::new(),
            markdown: "## Checklist\n- 校验配置\n- 校验运行态\n".to_string(),
            enabled: true,
            tags: vec!["ops".to_string(), "audit".to_string()],
            created_at: now,
            updated_at: now,
            scripts: vec![],
            resources: vec![],
            references: vec![],
            templates: vec![],
        },
    ]
}

fn seeded_global_constraint() -> GlobalConstraintRecord {
    GlobalConstraintRecord {
        content: "所有新增工作流必须保留 Rust + Tauri 2 的后续路线，不回退旧 Gin/Web 架构。"
            .to_string(),
        updated_at: now_ts() * 1000,
    }
}

fn flatten_menus_for_state(items: &[MenuInfo]) -> Vec<MenuInfo> {
    items
        .iter()
        .flat_map(|item| {
            let mut cloned = item.clone();
            cloned.children = vec![];
            std::iter::once(cloned)
                .chain(flatten_menus_for_state(&item.children))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn dedupe_u64(mut items: Vec<u64>) -> Vec<u64> {
    items.sort_unstable();
    items.dedup();
    items
}

fn sanitize_mcp_params(
    items: Vec<crate::models::McpToolParam>,
) -> Vec<crate::models::McpToolParam> {
    items
        .into_iter()
        .filter_map(|item| {
            let name = item.name.trim().to_string();
            if name.is_empty() {
                return None;
            }
            Some(crate::models::McpToolParam {
                name,
                description: item.description.trim().to_string(),
                type_name: if item.type_name.trim().is_empty() {
                    "string".to_string()
                } else {
                    item.type_name.trim().to_string()
                },
                required: item.required,
                default_value: item
                    .default_value
                    .filter(|value| !value.trim().is_empty())
                    .map(|value| value.trim().to_string()),
            })
        })
        .collect()
}

fn sanitize_mcp_outputs(
    items: Vec<crate::models::McpToolOutput>,
) -> Vec<crate::models::McpToolOutput> {
    let outputs = items
        .into_iter()
        .filter_map(|item| {
            let type_name = item.type_name.trim().to_string();
            if type_name.is_empty() {
                None
            } else {
                Some(crate::models::McpToolOutput { type_name })
            }
        })
        .collect::<Vec<_>>();
    if outputs.is_empty() {
        vec![crate::models::McpToolOutput {
            type_name: "text".to_string(),
        }]
    } else {
        outputs
    }
}

fn sanitize_announcement_attachments(
    items: Vec<AnnouncementAttachment>,
) -> Vec<AnnouncementAttachment> {
    items
        .into_iter()
        .filter_map(|item| {
            let name = item.name.trim().to_string();
            let url = item.url.trim().to_string();
            if name.is_empty() || url.is_empty() {
                return None;
            }
            Some(AnnouncementAttachment { name, url })
        })
        .collect()
}

fn build_mcp_descriptor(tool: McpToolRecord) -> McpToolDescriptor {
    let properties = tool
        .params
        .iter()
        .map(|item| {
            (
                item.name.clone(),
                McpSchemaProperty {
                    type_name: item.type_name.clone(),
                    description: item.description.clone(),
                    default_value: item.default_value.clone(),
                },
            )
        })
        .collect();
    let required = tool
        .params
        .iter()
        .filter(|item| item.required)
        .map(|item| item.name.clone())
        .collect();
    McpToolDescriptor {
        name: tool.name,
        description: tool.description,
        input_schema: McpInputSchema {
            type_name: "object".to_string(),
            properties,
            required,
        },
        response: tool.response,
        updated_at: tool.updated_at,
    }
}

fn sanitize_tags(tags: Vec<String>) -> Vec<String> {
    tags.into_iter()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .take(8)
        .collect()
}

fn select_skill_assets<'a>(
    skill: &'a mut SkillRecord,
    kind: &str,
) -> Result<&'a mut Vec<SkillAssetRecord>, String> {
    match kind {
        "script" => Ok(&mut skill.scripts),
        "resource" => Ok(&mut skill.resources),
        "reference" => Ok(&mut skill.references),
        "template" => Ok(&mut skill.templates),
        _ => Err("未知文件类型".to_string()),
    }
}

fn select_skill_assets_readonly<'a>(
    skill: &'a SkillRecord,
    kind: &str,
) -> Result<&'a Vec<SkillAssetRecord>, String> {
    match kind {
        "script" => Ok(&skill.scripts),
        "resource" => Ok(&skill.resources),
        "reference" => Ok(&skill.references),
        "template" => Ok(&skill.templates),
        _ => Err("未知文件类型".to_string()),
    }
}

fn non_empty(value: Option<String>, fallback: &str) -> String {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

fn rebuild_menu_tree(mut items: Vec<MenuInfo>) -> Vec<MenuInfo> {
    items.sort_by_key(|item| (item.parent_id, item.sort, item.id));
    let mut by_parent: HashMap<u64, Vec<MenuInfo>> = HashMap::new();
    for mut item in items {
        item.children = vec![];
        by_parent.entry(item.parent_id).or_default().push(item);
    }

    fn attach(parent_id: u64, by_parent: &mut HashMap<u64, Vec<MenuInfo>>) -> Vec<MenuInfo> {
        let mut children = by_parent.remove(&parent_id).unwrap_or_default();
        children.sort_by_key(|item| (item.sort, item.id));
        for child in &mut children {
            child.children = attach(child.id, by_parent);
        }
        children
    }

    attach(0, &mut by_parent)
}

fn flatten_dictionary_nodes(nodes: &[DictionaryDetailInfo]) -> Vec<DictionaryDetailInfo> {
    nodes
        .iter()
        .flat_map(|node| {
            let mut cloned = node.clone();
            cloned.children = vec![];
            std::iter::once(cloned)
                .chain(flatten_dictionary_nodes(&node.children))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn rebuild_dictionary_tree(mut nodes: Vec<DictionaryDetailInfo>) -> Vec<DictionaryDetailInfo> {
    nodes.sort_by_key(|item| (item.level, item.sort, item.id));
    let mut by_parent: HashMap<Option<u64>, Vec<DictionaryDetailInfo>> = HashMap::new();
    for mut node in nodes {
        node.children = vec![];
        by_parent.entry(node.parent_id).or_default().push(node);
    }

    fn attach(
        parent_id: Option<u64>,
        parent_level: u32,
        by_parent: &mut HashMap<Option<u64>, Vec<DictionaryDetailInfo>>,
    ) -> Vec<DictionaryDetailInfo> {
        let mut children = by_parent.remove(&parent_id).unwrap_or_default();
        children.sort_by_key(|item| (item.sort, item.id));
        for child in &mut children {
            child.level = if parent_id.is_some() {
                parent_level + 1
            } else {
                child.level.max(1)
            };
            child.children = attach(Some(child.id), child.level, by_parent);
        }
        children
    }

    attach(None, 0, &mut by_parent)
}

fn ttl_from(expiry_ts: i64) -> u64 {
    expiry_ts.saturating_sub(now_ts()).max(1) as u64
}

fn read_bool_env(key: &str, default: bool) -> bool {
    match std::env::var(key) {
        Ok(value) => matches!(
            value.as_str(),
            "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
        ),
        Err(_) => default,
    }
}

fn next_id(ids: impl Iterator<Item = u64>) -> u64 {
    ids.max().unwrap_or(0) + 1
}

fn timestamp_ms_string() -> String {
    (now_ts() * 1000).to_string()
}

fn page_result<T: Clone>(
    items: Vec<T>,
    page: Option<usize>,
    page_size: Option<usize>,
) -> PageResult<T> {
    let page = page.unwrap_or(1).max(1);
    let page_size = page_size.unwrap_or(10).max(1);
    let total = items.len();
    let start = (page - 1) * page_size;
    let end = usize::min(start + page_size, total);
    let list = if start >= total {
        vec![]
    } else {
        items[start..end].to_vec()
    };

    PageResult {
        list,
        total,
        page,
        page_size,
    }
}

fn menu_node(
    id: u64,
    parent_id: u64,
    name: &str,
    path: &str,
    component: &str,
    title: &str,
    icon: &str,
    children: Vec<MenuInfo>,
) -> MenuInfo {
    MenuInfo {
        id,
        parent_id,
        name: name.to_string(),
        path: path.to_string(),
        component: component.to_string(),
        sort: id as i64,
        hidden: false,
        meta: MenuMeta {
            title: title.to_string(),
            icon: icon.to_string(),
        },
        menu_btn: default_menu_buttons(id, name),
        parameters: vec![],
        children,
    }
}

fn default_menu_buttons(menu_id: u64, menu_name: &str) -> Vec<crate::models::MenuButtonInfo> {
    let buttons: &[(&str, &str)] = match menu_name {
        "users" => &[
            ("新增用户", "create"),
            ("编辑用户", "edit"),
            ("删除用户", "delete"),
            ("多角色分配", "setAuthorities"),
        ],
        "authorities" => &[
            ("新增角色", "create"),
            ("编辑角色", "edit"),
            ("删除角色", "delete"),
            ("设置权限", "configurePermissions"),
            ("分配给用户", "assignUsers"),
        ],
        "menus" => &[
            ("新增菜单", "create"),
            ("编辑菜单", "edit"),
            ("删除菜单", "delete"),
            ("分配角色", "assignRoles"),
        ],
        "apis" => &[
            ("新增接口", "create"),
            ("编辑接口", "edit"),
            ("删除接口", "delete"),
            ("分配角色", "assignRoles"),
        ],
        "params" => &[
            ("新增参数", "create"),
            ("编辑参数", "edit"),
            ("删除参数", "delete"),
        ],
        "dictionaries" => &[
            ("新增字典", "create"),
            ("编辑字典", "edit"),
            ("删除字典", "delete"),
        ],
        _ => &[],
    };

    buttons
        .iter()
        .enumerate()
        .map(|(index, (name, desc))| crate::models::MenuButtonInfo {
            id: menu_id * 100 + index as u64 + 1,
            sys_base_menu_id: menu_id,
            name: (*name).to_string(),
            desc: (*desc).to_string(),
        })
        .collect()
}
