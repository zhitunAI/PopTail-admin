use super::*;

pub(super) fn redact_system_config(mut config: SystemConfigInfo) -> SystemConfigInfo {
    config.database_url = redact_url_secret(&config.database_url);
    config.redis_url = redact_url_secret(&config.redis_url);
    config
}

pub(super) fn redact_url_secret(url: &str) -> String {
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

pub(super) fn read_service_token() -> Result<String, String> {
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

pub(super) fn read_bootstrap_admin_password() -> Result<String, String> {
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

pub(super) fn read_default_user_password() -> Result<String, String> {
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

pub(super) fn is_strong_runtime_secret(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.len() >= 32 && trimmed != "svc-moderation-token"
}

pub(super) fn is_strong_password(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.len() >= 12 && trimmed != "123456"
}

pub(super) fn is_production_env() -> bool {
    std::env::var("GAA_ENV")
        .or_else(|_| std::env::var("RUST_ENV"))
        .or_else(|_| std::env::var("APP_ENV"))
        .map(|value| matches!(value.to_ascii_lowercase().as_str(), "prod" | "production"))
        .unwrap_or(false)
}

pub(super) fn build_active_sessions_index(sessions: &[SessionRecord]) -> HashMap<u64, String> {
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

pub(super) fn seeded_dictionary_details() -> HashMap<u64, Vec<DictionaryDetailInfo>> {
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

pub(super) fn seeded_authorities() -> Vec<AuthorityInfo> {
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

pub(super) fn flatten_authorities(items: &[AuthorityInfo]) -> Vec<AuthorityInfo> {
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

pub(super) fn rebuild_authority_tree(mut items: Vec<AuthorityInfo>) -> Vec<AuthorityInfo> {
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

pub(super) fn seeded_menu_tree() -> Vec<MenuInfo> {
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

pub(super) fn seeded_authority_menu_ids(menus: &[MenuInfo]) -> HashMap<u32, Vec<u64>> {
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

pub(super) fn seeded_authority_button_ids(
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

pub(super) fn seeded_apis() -> Vec<ApiInfo> {
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

pub(super) fn seeded_dictionaries() -> Vec<DictionaryInfo> {
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

pub(super) fn seeded_system_config(
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

pub(super) fn seeded_api_tokens() -> Vec<ApiTokenRecord> {
    vec![ApiTokenRecord {
        id: 1,
        name: "moderation-service".to_string(),
        scope: "article.review".to_string(),
        ttl: "24h".to_string(),
        status: "已登记".to_string(),
    }]
}

pub(super) fn seeded_packages() -> Vec<PackageRecord> {
    vec![PackageRecord {
        id: 1,
        name: "feature-bundle".to_string(),
        kind: "package".to_string(),
        output: "/modules/feature-bundle".to_string(),
        summary: "用户列表、详情表单、角色切换、导出模板，以及开发完成后的 Tauri 2 桌面端重构。"
            .to_string(),
    }]
}

pub(super) fn seeded_plugin_manifests() -> Vec<PluginManifestRecord> {
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

pub(super) fn seeded_plugin_installs() -> Vec<PluginInstallRecord> {
    vec![]
}

pub(super) fn seeded_auto_code_registry() -> Vec<AutoCodeRegistryRecord> {
    vec![]
}

pub(super) fn seeded_releases() -> Vec<ReleaseRecord> {
    vec![ReleaseRecord {
        id: 1,
        created_at: now_ts() * 1000,
        note: "记录当前前后端页面与权限恢复批次。".to_string(),
    }]
}

pub(super) fn seeded_email_records() -> Vec<EmailRecord> {
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

pub(super) fn seeded_announcements() -> Vec<AnnouncementRecord> {
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

pub(super) fn seeded_llm_configs() -> Vec<LlmConfigRecord> {
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

pub(super) fn seeded_frontend_nav() -> Vec<FrontendNavRecord> {
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

pub(super) fn seeded_frontend_settings() -> FrontendSettingsRecord {
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

pub(super) fn seeded_article_categories() -> Vec<ArticleCategoryRecord> {
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

pub(super) fn seeded_articles() -> Vec<ArticleRecord> {
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

pub(super) fn seeded_members() -> Vec<MemberRecord> {
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

pub(super) fn seeded_console_menus() -> Vec<ConsoleMenuRecord> {
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

pub(super) fn seeded_resume_uploads() -> Vec<ResumeUploadRecord> {
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

pub(super) fn seeded_customers() -> Vec<CustomerRecord> {
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

pub(super) fn seeded_mcp_tools() -> Vec<McpToolRecord> {
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

pub(super) fn seeded_skills() -> Vec<SkillRecord> {
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

pub(super) fn seeded_global_constraint() -> GlobalConstraintRecord {
    GlobalConstraintRecord {
        content: "所有新增工作流必须保留 Rust + Tauri 2 的后续路线，不回退旧 Gin/Web 架构。"
            .to_string(),
        updated_at: now_ts() * 1000,
    }
}

pub(super) fn flatten_menus_for_state(items: &[MenuInfo]) -> Vec<MenuInfo> {
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

pub(super) fn dedupe_u64(mut items: Vec<u64>) -> Vec<u64> {
    items.sort_unstable();
    items.dedup();
    items
}

pub(super) fn sanitize_mcp_params(
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

pub(super) fn sanitize_mcp_outputs(
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

pub(super) fn sanitize_announcement_attachments(
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

pub(super) fn build_mcp_descriptor(tool: McpToolRecord) -> McpToolDescriptor {
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

pub(super) fn sanitize_tags(tags: Vec<String>) -> Vec<String> {
    tags.into_iter()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .take(8)
        .collect()
}

pub(super) fn select_skill_assets<'a>(
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

pub(super) fn select_skill_assets_readonly<'a>(
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

pub(super) fn non_empty(value: Option<String>, fallback: &str) -> String {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

pub(super) fn rebuild_menu_tree(mut items: Vec<MenuInfo>) -> Vec<MenuInfo> {
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

pub(super) fn flatten_dictionary_nodes(
    nodes: &[DictionaryDetailInfo],
) -> Vec<DictionaryDetailInfo> {
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

pub(super) fn rebuild_dictionary_tree(
    mut nodes: Vec<DictionaryDetailInfo>,
) -> Vec<DictionaryDetailInfo> {
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

pub(super) fn ttl_from(expiry_ts: i64) -> u64 {
    expiry_ts.saturating_sub(now_ts()).max(1) as u64
}

pub(super) fn read_bool_env(key: &str, default: bool) -> bool {
    match std::env::var(key) {
        Ok(value) => matches!(
            value.as_str(),
            "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
        ),
        Err(_) => default,
    }
}

pub(super) fn next_id(ids: impl Iterator<Item = u64>) -> u64 {
    ids.max().unwrap_or(0) + 1
}

pub(super) fn timestamp_ms_string() -> String {
    (now_ts() * 1000).to_string()
}

pub(super) fn page_result<T: Clone>(
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

pub(super) fn menu_node(
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

pub(super) fn default_menu_buttons(
    menu_id: u64,
    menu_name: &str,
) -> Vec<crate::models::MenuButtonInfo> {
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
