use crate::auth::{
    AuthConfig, create_access_token, create_refresh_token, decode_token, hash_password,
    needs_rolling_refresh, now_ts, verify_password,
};
use crate::casbin_port::CasbinPort;
use crate::models::{
    AnnouncementAttachment, AnnouncementDataSourcePayload, AnnouncementRecord, ApiInfo,
    ApiListRequest, ApiTokenRecord, ArticleCategoryRecord, ArticleRecord, AuditEvent,
    AuthorityInfo, AutoCodeRegistryRecord, CasbinInfo, ConsoleMenuRecord, CustomerRecord,
    DictionaryDetailInfo, DictionaryInfo, EmailPresetRecord, EmailRecord, ErrorLogInfo, ExportTemplateInfo,
    FrontendNavRecord, FrontendSettingsRecord, FrontendSettingsUpdateRequest,
    GlobalConstraintRecord, LlmConfigRecord, LoginLogInfo, McpInputSchema, McpSchemaProperty,
    McpServiceStatusPayload, McpTestResult, McpToolDescriptor, McpToolRecord, MemberRecord,
    MenuInfo, MenuMeta, OperationLogInfo, PackageRecord, PageResult, ParamInfo,
    PluginInstallRecord, PluginManifestRecord, ReleaseRecord, ResumeUploadRecord, RuntimeInfo,
    ScanSessionRecord, SelectOption, ServiceIdentity, SkillAssetRecord, SkillRecord, SkillSummary,
    SkillToolDefinition, SystemConfigInfo, UploadFileRecord, UserInfo, UserListRequest, UserRecord,
};
use crate::storage::{PersistedAuthorityButtonIds, PersistedMenuState, Persistence};
use std::collections::HashMap;
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
            compatibility_refresh_headers: read_bool_env("POP_TAIL_COMPATIBILITY_REFRESH_HEADERS", true),
            multipoint_enabled: read_bool_env("POP_TAIL_MULTIPOINT_ENABLED", false),
        }
    }
}

#[derive(Clone, Debug)]
pub struct InnerState {
    pub users_by_name: HashMap<String, UserRecord>,
    pub users_by_id: HashMap<u64, UserRecord>,
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
    pub email_presets: Vec<EmailPresetRecord>,
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
}

#[derive(Clone, Debug)]
pub struct Authenticated {
    pub user: UserInfo,
    pub token_version: u64,
    pub new_token: Option<String>,
    pub new_expires_at: Option<i64>,
}

impl AppState {
    async fn multipoint_enabled(&self) -> bool {
        let guard = self.inner.read().await;
        guard.system_config.multipoint_enabled
    }

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
                header_img: "https://example.com/pop-tail-admin-avatar.png".to_string(),
                phone: "13800138000".to_string(),
                email: "admin@gaa.local".to_string(),
                enable: 1,
            },
            password: bootstrap_password_hash.clone(),
            token_version: 1,
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
                header_img: "https://example.com/pop-tail-disabled-avatar.png".to_string(),
                phone: "13900139000".to_string(),
                email: "disabled@gaa.local".to_string(),
                enable: 2,
            },
            password: bootstrap_password_hash,
            token_version: 1,
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
        let restored_token_versions = persistence.load_user_auth_versions().await;
        for (user_id, token_version) in restored_token_versions {
            if let Some(record) = users_by_id.get_mut(&user_id) {
                record.token_version = token_version.max(1);
            }
        }
        users_by_name = users_by_id
            .values()
            .map(|record| (record.info.user_name.clone(), record.clone()))
            .collect();
        let restored_audits = persistence.load_audits().await;
        let dictionary_details = seeded_dictionary_details();
        let authorities = authority_tree;
        let seeded_menus = seeded_menu_tree();
        let seeded_authority_menu_ids = seeded_authority_menu_ids(&seeded_menus);
        let seeded_authority_button_ids =
            seeded_authority_button_ids(&seeded_menus, &seeded_authority_menu_ids);
        let (menus, authority_menu_ids, authority_button_ids) =
            if let Some(saved) = persistence.load_menu_state().await {
                let authority_button_ids = saved
                    .authority_button_ids
                    .into_iter()
                    .map(|item| ((item.authority_id, item.menu_id), item.selected))
                    .collect::<HashMap<_, _>>();
                (saved.menus, saved.authority_menu_ids, authority_button_ids)
            } else {
                (
                    seeded_menus,
                    seeded_authority_menu_ids,
                    seeded_authority_button_ids,
                )
            };
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
        let seeded_email_presets = seeded_email_presets();
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

        let email_presets = {
            let restored = persistence.load_email_presets().await;
            if restored.is_empty() {
                seeded_email_presets
            } else {
                restored
            }
        };

        Ok(Self {
            config,
            casbin: Arc::new(RwLock::new(casbin)),
            inner: Arc::new(RwLock::new(InnerState {
                users_by_name,
                users_by_id,
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
                email_presets,
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
        })
    }

    pub async fn persist_menu_state(&self) {
        let snapshot = {
            let guard = self.inner.read().await;
            PersistedMenuState {
                authority_button_ids: guard
                    .authority_button_ids
                    .iter()
                    .map(|((authority_id, menu_id), selected)| PersistedAuthorityButtonIds {
                        authority_id: *authority_id,
                        menu_id: *menu_id,
                        selected: selected.clone(),
                    })
                    .collect(),
                authority_menu_ids: guard.authority_menu_ids.clone(),
                menus: guard.menus.clone(),
            }
        };
        self.persistence.save_menu_state(&snapshot).await;
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

        let multipoint_enabled = self.multipoint_enabled().await;
        let (user_info, token_version, persist_version) = {
            let mut guard = self.inner.write().await;
            let record = guard
                .users_by_id
                .get_mut(&user.info.id)
                .ok_or_else(|| "用户不存在".to_string())?;
            if !multipoint_enabled {
                record.token_version = record.token_version.saturating_add(1);
            }
            let user_name = record.info.user_name.clone();
            let token_version = record.token_version;
            let user_info = record.info.clone();
            if let Some(record_by_name) = guard.users_by_name.get_mut(&user_name) {
                record_by_name.token_version = token_version;
            }
            (user_info, token_version, !multipoint_enabled)
        };

        if persist_version {
            self.persistence
                .upsert_user_auth_version(user_info.id, token_version)
                .await;
        }

        let (access_token, expires_at) = create_access_token(
            &self.config.auth,
            user_info.id,
            user_info.authority_id,
            token_version,
        )?;
        let (refresh_token, _) = create_refresh_token(
            &self.config.auth,
            user_info.id,
            user_info.authority_id,
            token_version,
        )?;

        Ok((user_info, access_token, expires_at * 1000, refresh_token))
    }

    pub async fn refresh(&self, refresh_token: &str) -> Result<(String, i64, String), String> {
        let claims = decode_token(&self.config.auth, refresh_token)?;
        if claims.kind != "human_refresh" {
            return Err("refresh token类型错误".to_string());
        }

        let guard = self.inner.read().await;
        let user = guard
            .users_by_id
            .get(&claims.sub)
            .cloned()
            .ok_or_else(|| "用户不存在".to_string())?;
        if user.info.enable != 1 {
            return Err("用户被禁止登录".to_string());
        }
        if claims.ver != user.token_version || claims.exp <= now_ts() {
            return Err("refresh token失效".to_string());
        }
        let effective_user = Self::effective_user_info(&user, claims.aid)?;
        drop(guard);

        let (new_token, exp) = create_access_token(
            &self.config.auth,
            effective_user.id,
            effective_user.authority_id,
            claims.ver,
        )?;
        let (next_refresh_token, _) = create_refresh_token(
            &self.config.auth,
            effective_user.id,
            effective_user.authority_id,
            claims.ver,
        )?;
        Ok((new_token, exp, next_refresh_token))
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
        let guard = self.inner.read().await;
        let user = guard
            .users_by_id
            .get(&claims.sub)
            .cloned()
            .ok_or_else(|| "用户不存在".to_string())?;
        if user.info.enable != 1 {
            return Err("用户被禁止登录".to_string());
        }
        if claims.ver != user.token_version || claims.exp <= now_ts() {
            return Err("登录已过期，请重新登录".to_string());
        }
        let effective_user = Self::effective_user_info(&user, claims.aid)?;
        drop(guard);

        let maybe_new =
            if allow_rolling_refresh && needs_rolling_refresh(&self.config.auth, claims.exp) {
                let (new_token, exp) = create_access_token(
                    &self.config.auth,
                    effective_user.id,
                    effective_user.authority_id,
                    claims.ver,
                )?;
                Some((new_token, exp))
            } else {
                None
            };

        Ok(Authenticated {
            user: effective_user,
            token_version: claims.ver,
            new_token: maybe_new.as_ref().map(|v| v.0.clone()),
            new_expires_at: maybe_new.map(|v| v.1),
        })
    }

    pub async fn revoke_user_tokens(&self, user_id: u64) {
        let next_version = {
            let mut guard = self.inner.write().await;
            let updated = guard.users_by_id.get_mut(&user_id).map(|record| {
                record.token_version = record.token_version.saturating_add(1);
                (record.info.user_name.clone(), record.token_version)
            });
            if let Some((user_name, next_version)) = updated {
                if let Some(record_by_name) = guard.users_by_name.get_mut(&user_name) {
                    record_by_name.token_version = next_version;
                }
                Some(next_version)
            } else {
                None
            }
        };

        if let Some(token_version) = next_version {
            self.persistence
                .upsert_user_auth_version(user_id, token_version)
                .await;
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
        user_id: u64,
        token_version: u64,
        authority_id: u32,
    ) -> Result<(UserInfo, String, i64, String), String> {
        let guard = self.inner.read().await;
        let user = guard
            .users_by_id
            .get(&user_id)
            .ok_or_else(|| "用户不存在".to_string())?
            .clone();
        if user.token_version != token_version {
            return Err("登录已过期，请重新登录".to_string());
        }
        let updated_user = Self::effective_user_info(&user, authority_id)?;
        drop(guard);

        let (token, exp) =
            create_access_token(&self.config.auth, user_id, authority_id, token_version)?;
        let (refresh_token, _) =
            create_refresh_token(&self.config.auth, user_id, authority_id, token_version)?;
        Ok((updated_user, token, exp * 1000, refresh_token))
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
}
