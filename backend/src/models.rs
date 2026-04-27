use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthorityInfo {
    #[serde(rename = "ID")]
    pub id: u64,
    #[serde(rename = "authorityId")]
    pub authority_id: u32,
    #[serde(rename = "authorityName")]
    pub authority_name: String,
    #[serde(rename = "defaultRouter")]
    pub default_router: String,
    #[serde(rename = "parentId")]
    pub parent_id: u32,
    pub enable: u8,
    pub status: u8,
    pub children: Vec<AuthorityInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserInfo {
    #[serde(rename = "ID")]
    pub id: u64,
    pub uuid: Uuid,
    #[serde(rename = "userName")]
    pub user_name: String,
    #[serde(rename = "nickName")]
    pub nick_name: String,
    #[serde(rename = "authorityId")]
    pub authority_id: u32,
    pub authority: AuthorityInfo,
    pub authorities: Vec<AuthorityInfo>,
    #[serde(rename = "headerImg")]
    pub header_img: String,
    pub phone: String,
    pub email: String,
    pub enable: u8,
}

#[derive(Clone, Debug)]
pub struct UserRecord {
    pub info: UserInfo,
    pub password: String,
    pub token_version: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CasbinInfo {
    pub path: String,
    pub method: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub msg: String,
    pub data: T,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PageResult<T> {
    #[serde(rename = "List")]
    pub list: Vec<T>,
    #[serde(rename = "Total")]
    pub total: usize,
    #[serde(rename = "Page")]
    pub page: usize,
    #[serde(rename = "PageSize")]
    pub page_size: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginPayload {
    pub user: UserInfo,
    pub token: String,
    #[serde(rename = "expiresAt")]
    pub expires_at: i64,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserInfoPayload {
    #[serde(rename = "userInfo")]
    pub user_info: UserInfo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BootstrapPayload {
    #[serde(rename = "userInfo")]
    pub user_info: UserInfo,
    #[serde(rename = "policyPaths")]
    pub policy_paths: Vec<CasbinInfo>,
    pub menus: Vec<MenuInfo>,
    #[serde(rename = "authorityButtons")]
    pub authority_buttons: Vec<AuthorityButtonMatrixBatchSelectionItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecisionPayload {
    pub accepted: bool,
    #[serde(rename = "auditId")]
    pub audit_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SwitchAuthorityPayload {
    pub user: UserInfo,
    pub token: String,
    #[serde(rename = "expiresAt")]
    pub expires_at: i64,
    #[serde(rename = "refreshToken", skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MenuMeta {
    pub title: String,
    pub icon: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MenuButtonInfo {
    #[serde(rename = "ID")]
    pub id: u64,
    #[serde(rename = "sysBaseMenuID")]
    pub sys_base_menu_id: u64,
    pub name: String,
    pub desc: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MenuParameterInfo {
    #[serde(rename = "ID")]
    pub id: u64,
    #[serde(rename = "sysBaseMenuID")]
    pub sys_base_menu_id: u64,
    #[serde(rename = "type")]
    pub type_name: String,
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MenuInfo {
    #[serde(rename = "ID")]
    pub id: u64,
    #[serde(rename = "parentId")]
    pub parent_id: u64,
    pub name: String,
    pub path: String,
    pub component: String,
    pub sort: i64,
    pub hidden: bool,
    pub meta: MenuMeta,
    #[serde(rename = "menuBtn", default)]
    pub menu_btn: Vec<MenuButtonInfo>,
    #[serde(default)]
    pub parameters: Vec<MenuParameterInfo>,
    pub children: Vec<MenuInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MenuTreePayload {
    pub menus: Vec<MenuInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiInfo {
    #[serde(rename = "ID")]
    pub id: u64,
    pub path: String,
    #[serde(rename = "apiGroup")]
    pub api_group: String,
    pub description: String,
    pub method: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DictionaryInfo {
    #[serde(rename = "ID")]
    pub id: u64,
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub status: bool,
    pub desc: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParamInfo {
    #[serde(rename = "ID")]
    pub id: u64,
    pub key: String,
    pub value: String,
    pub desc: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperationLogInfo {
    #[serde(rename = "ID")]
    pub id: u64,
    pub ip: String,
    pub method: String,
    pub path: String,
    pub status: i64,
    pub latency: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginLogInfo {
    #[serde(rename = "ID")]
    pub id: u64,
    pub username: String,
    pub ip: String,
    pub status: bool,
    #[serde(rename = "errorMessage")]
    pub error_message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DictionaryDetailInfo {
    #[serde(rename = "ID")]
    pub id: u64,
    pub label: String,
    pub value: String,
    pub extend: String,
    pub level: u32,
    pub status: bool,
    pub sort: i64,
    #[serde(rename = "parentID")]
    pub parent_id: Option<u64>,
    pub children: Vec<DictionaryDetailInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorLogInfo {
    #[serde(rename = "ID")]
    pub id: u64,
    pub error: String,
    pub path: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportTemplateInfo {
    #[serde(rename = "ID")]
    pub id: u64,
    pub name: String,
    pub template_id: String,
    pub desc: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeInfo {
    pub os: String,
    #[serde(rename = "cpuCores")]
    pub cpu_cores: usize,
    #[serde(rename = "rustVersion")]
    pub rust_version: String,
    #[serde(rename = "dbBackend")]
    pub db_backend: String,
    #[serde(rename = "redisEnabled")]
    pub redis_enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeInfoPayload {
    pub server: RuntimeInfo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemConfigInfo {
    #[serde(rename = "bindAddress")]
    pub bind_address: String,
    #[serde(rename = "databaseUrl")]
    pub database_url: String,
    #[serde(rename = "redisUrl")]
    pub redis_url: String,
    #[serde(rename = "multipointEnabled")]
    pub multipoint_enabled: bool,
    #[serde(rename = "compatibilityRefreshHeaders")]
    pub compatibility_refresh_headers: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LlmConfigRecord {
    #[serde(rename = "ID")]
    pub id: u64,
    pub provider: String,
    pub model: String,
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    #[serde(rename = "apiKey")]
    pub api_key: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LlmConfigUpsertRequest {
    #[serde(rename = "ID", default)]
    pub id: Option<u64>,
    pub provider: String,
    pub model: String,
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    #[serde(rename = "apiKey")]
    pub api_key: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrontendNavRecord {
    #[serde(rename = "ID")]
    pub id: u64,
    pub title: String,
    pub path: String,
    pub icon: String,
    pub order: i64,
    pub visible: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FrontendNavUpsertRequest {
    #[serde(rename = "ID", default)]
    pub id: Option<u64>,
    pub title: String,
    pub path: String,
    pub icon: String,
    pub order: i64,
    pub visible: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArticleCategoryRecord {
    #[serde(rename = "ID")]
    pub id: u64,
    pub name: String,
    pub slug: String,
    pub sort: i64,
    pub status: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ArticleCategoryUpsertRequest {
    #[serde(rename = "ID", default)]
    pub id: Option<u64>,
    pub name: String,
    pub slug: String,
    pub sort: i64,
    pub status: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArticleRecord {
    #[serde(rename = "ID")]
    pub id: u64,
    #[serde(rename = "categoryId")]
    pub category_id: u64,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub status: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ArticleUpsertRequest {
    #[serde(rename = "ID", default)]
    pub id: Option<u64>,
    #[serde(rename = "categoryId")]
    pub category_id: u64,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemberRecord {
    #[serde(rename = "ID")]
    pub id: u64,
    pub email: String,
    pub nickname: String,
    pub provider: String,
    pub status: String,
    #[serde(rename = "lastLogin")]
    pub last_login: String,
    #[serde(rename = "consolePath")]
    pub console_path: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MemberUpsertRequest {
    #[serde(rename = "ID", default)]
    pub id: Option<u64>,
    pub email: String,
    pub nickname: String,
    pub provider: String,
    pub status: String,
    #[serde(rename = "lastLogin", default)]
    pub last_login: Option<String>,
    #[serde(rename = "consolePath")]
    pub console_path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsoleMenuRecord {
    #[serde(rename = "ID")]
    pub id: u64,
    pub title: String,
    pub group: String,
    pub path: String,
    pub icon: String,
    pub order: i64,
    pub visible: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConsoleMenuUpsertRequest {
    #[serde(rename = "ID", default)]
    pub id: Option<u64>,
    pub title: String,
    pub group: String,
    pub path: String,
    pub icon: String,
    pub order: i64,
    pub visible: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct IdRequest {
    #[serde(rename = "ID")]
    pub id: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrontendSettingsRecord {
    #[serde(rename = "logoUrl")]
    pub logo_url: String,
    #[serde(rename = "siteName")]
    pub site_name: String,
    #[serde(rename = "siteSlogan")]
    pub site_slogan: String,
    #[serde(rename = "siteDescription")]
    pub site_description: String,
    #[serde(rename = "recordNumber")]
    pub record_number: String,
    #[serde(rename = "smtpHost")]
    pub smtp_host: String,
    #[serde(rename = "smtpPort")]
    pub smtp_port: u16,
    #[serde(rename = "smtpUser")]
    pub smtp_user: String,
    #[serde(rename = "smtpPassword")]
    pub smtp_password: String,
    #[serde(rename = "mailFrom")]
    pub mail_from: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FrontendSettingsUpdateRequest {
    #[serde(rename = "logoUrl")]
    pub logo_url: String,
    #[serde(rename = "siteName")]
    pub site_name: String,
    #[serde(rename = "siteSlogan")]
    pub site_slogan: String,
    #[serde(rename = "siteDescription")]
    pub site_description: String,
    #[serde(rename = "recordNumber")]
    pub record_number: String,
    #[serde(rename = "smtpHost")]
    pub smtp_host: String,
    #[serde(rename = "smtpPort")]
    pub smtp_port: u16,
    #[serde(rename = "smtpUser")]
    pub smtp_user: String,
    #[serde(rename = "smtpPassword")]
    pub smtp_password: String,
    #[serde(rename = "mailFrom")]
    pub mail_from: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub captcha: Option<String>,
    #[serde(rename = "captchaId")]
    pub captcha_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RefreshRequest {
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SwitchAuthorityRequest {
    #[serde(rename = "authorityId")]
    pub authority_id: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UserListRequest {
    #[serde(default)]
    pub page: Option<usize>,
    #[serde(rename = "pageSize", default)]
    pub page_size: Option<usize>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(rename = "nickName", default)]
    pub nick_name: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProfileUpdateRequest {
    #[serde(rename = "nickName")]
    pub nick_name: String,
    pub phone: String,
    pub email: String,
    #[serde(default)]
    pub password: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UserUpsertRequest {
    #[serde(rename = "ID", default)]
    pub id: Option<u64>,
    #[serde(rename = "userName")]
    pub user_name: String,
    #[serde(rename = "nickName")]
    pub nick_name: String,
    #[serde(rename = "authorityId")]
    pub authority_id: u32,
    #[serde(rename = "authorityIds", default)]
    pub authority_ids: Vec<u32>,
    pub phone: String,
    pub email: String,
    pub enable: u8,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AuthorityUpsertRequest {
    #[serde(rename = "ID", default)]
    pub id: Option<u64>,
    #[serde(rename = "authorityId")]
    pub authority_id: u32,
    #[serde(rename = "authorityName")]
    pub authority_name: String,
    #[serde(rename = "defaultRouter")]
    pub default_router: String,
    #[serde(rename = "parentId")]
    pub parent_id: u32,
    #[serde(default = "default_enabled_status")]
    pub enable: u8,
    #[serde(default = "default_enabled_status")]
    pub status: u8,
}

fn default_enabled_status() -> u8 {
    1
}

#[derive(Clone, Debug, Deserialize)]
pub struct AuthorityRequest {
    #[serde(rename = "authorityId")]
    pub authority_id: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AuthorityPolicyUpdateRequest {
    #[serde(rename = "authorityId")]
    pub authority_id: u32,
    #[serde(default)]
    pub policies: Vec<CasbinInfo>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AuthorityRoleUsersRequest {
    #[serde(rename = "authorityId")]
    pub authority_id: u32,
    #[serde(rename = "userIds", default)]
    pub user_ids: Vec<u64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AuthorityButtonMatrixRequest {
    #[serde(rename = "menuID")]
    pub menu_id: u64,
    #[serde(rename = "authorityId")]
    pub authority_id: u32,
    #[serde(default)]
    pub selected: Vec<u64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AuthorityButtonMatrixBatchRequest {
    #[serde(rename = "authorityId")]
    pub authority_id: u32,
    #[serde(rename = "menuIDs", default)]
    pub menu_ids: Vec<u64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AuthorityButtonMatrixSelection {
    pub selected: Vec<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthorityButtonMatrixBatchSelectionItem {
    #[serde(rename = "menuID")]
    pub menu_id: u64,
    pub selected: Vec<u64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ApiListRequest {
    #[serde(default)]
    pub page: Option<usize>,
    #[serde(rename = "pageSize", default)]
    pub page_size: Option<usize>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "apiGroup", default)]
    pub api_group: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ParamUpsertRequest {
    #[serde(rename = "ID", default)]
    pub id: Option<u64>,
    pub key: String,
    pub value: String,
    pub desc: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DictionaryDetailUpsertRequest {
    #[serde(rename = "ID", default)]
    pub id: Option<u64>,
    #[serde(rename = "sysDictionaryID")]
    pub sys_dictionary_id: u64,
    pub label: String,
    pub value: String,
    pub extend: String,
    pub level: u32,
    pub status: bool,
    pub sort: i64,
    #[serde(rename = "parentID", default)]
    pub parent_id: Option<u64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MenuUpsertRequest {
    #[serde(rename = "ID", default)]
    pub id: Option<u64>,
    #[serde(rename = "parentId")]
    pub parent_id: u64,
    pub name: String,
    pub path: String,
    pub component: String,
    pub sort: i64,
    pub hidden: bool,
    pub meta: MenuMeta,
    #[serde(rename = "menuBtn", default)]
    pub menu_btn: Vec<MenuButtonInfo>,
    #[serde(default)]
    pub parameters: Vec<MenuParameterInfo>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MenuBatchUpsertRequest {
    pub menus: Vec<MenuUpsertRequest>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MenuAuthorityAssignmentRequest {
    #[serde(rename = "authorityId")]
    pub authority_id: u32,
    pub menus: Vec<MenuInfo>,
}

#[derive(Clone, Debug, Serialize)]
pub struct MenuAuthorityPayload {
    pub menus: Vec<MenuInfo>,
}

#[derive(Clone, Debug, Serialize)]
pub struct MenuRoleIdsPayload {
    #[serde(rename = "authorityIds")]
    pub authority_ids: Vec<u32>,
    #[serde(rename = "defaultRouterAuthorityIds")]
    pub default_router_authority_ids: Vec<u32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MenuRoleUpdateRequest {
    #[serde(rename = "menuId")]
    pub menu_id: u64,
    #[serde(rename = "authorityIds", default)]
    pub authority_ids: Vec<u32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MenuIdRequest {
    #[serde(rename = "menuId")]
    pub menu_id: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ApiUpsertRequest {
    #[serde(rename = "ID", default)]
    pub id: Option<u64>,
    pub path: String,
    #[serde(rename = "apiGroup")]
    pub api_group: String,
    pub description: String,
    pub method: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DictionaryUpsertRequest {
    #[serde(rename = "ID", default)]
    pub id: Option<u64>,
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub status: bool,
    pub desc: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SystemConfigUpdateRequest {
    #[serde(rename = "bindAddress")]
    pub bind_address: String,
    #[serde(rename = "databaseUrl")]
    pub database_url: String,
    #[serde(rename = "redisUrl")]
    pub redis_url: String,
    #[serde(rename = "multipointEnabled")]
    pub multipoint_enabled: bool,
    #[serde(rename = "compatibilityRefreshHeaders")]
    pub compatibility_refresh_headers: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiTokenRecord {
    #[serde(rename = "ID")]
    pub id: u64,
    pub name: String,
    pub scope: String,
    pub ttl: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ApiTokenIssueRequest {
    pub name: String,
    pub scope: String,
    pub ttl: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackageRecord {
    #[serde(rename = "ID")]
    pub id: u64,
    pub name: String,
    pub kind: String,
    pub output: String,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PackageUpsertRequest {
    #[serde(rename = "ID", default)]
    pub id: Option<u64>,
    pub name: String,
    pub kind: String,
    pub output: String,
    pub summary: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginManifestRecord {
    #[serde(rename = "ID")]
    pub id: u64,
    #[serde(rename = "pluginName")]
    pub plugin_name: String,
    #[serde(rename = "menuGroup")]
    pub menu_group: String,
    #[serde(rename = "menuIds")]
    pub menu_ids: Vec<u64>,
    #[serde(rename = "apiIds")]
    pub api_ids: Vec<u64>,
    #[serde(rename = "dictionaryIds")]
    pub dictionary_ids: Vec<u64>,
    #[serde(rename = "savedAt")]
    pub saved_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PluginManifestUpsertRequest {
    #[serde(rename = "pluginName")]
    pub plugin_name: String,
    #[serde(rename = "menuGroup")]
    pub menu_group: String,
    #[serde(rename = "menuIds")]
    pub menu_ids: Vec<u64>,
    #[serde(rename = "apiIds")]
    pub api_ids: Vec<u64>,
    #[serde(rename = "dictionaryIds")]
    pub dictionary_ids: Vec<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginInstallRecord {
    #[serde(rename = "ID")]
    pub id: u64,
    pub name: String,
    pub kind: String,
    pub target: String,
    pub manifest: String,
    pub status: String,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PluginInstallRequest {
    #[serde(rename = "fileName")]
    pub file_name: String,
    pub kind: String,
    pub target: String,
    pub manifest: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutoCodeRegistryRecord {
    #[serde(rename = "ID")]
    pub id: u64,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AutoCodeRegistryUpsertRequest {
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReleaseRecord {
    #[serde(rename = "ID")]
    pub id: u64,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    pub note: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ReleaseAppendRequest {
    pub note: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmailRecord {
    #[serde(rename = "ID")]
    pub id: u64,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub mode: String,
    pub status: String,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmailPresetRecord {
    #[serde(rename = "ID")]
    pub id: u64,
    pub name: String,
    pub description: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EmailSendRequest {
    #[serde(default)]
    pub to: Option<String>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EmailPresetListRequest {
    #[serde(default)]
    pub page: Option<usize>,
    #[serde(rename = "pageSize", default)]
    pub page_size: Option<usize>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EmailPresetUpsertRequest {
    #[serde(rename = "ID", default)]
    pub id: Option<u64>,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub to: String,
    pub subject: String,
    pub body: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnnouncementAttachment {
    pub name: String,
    pub url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnnouncementRecord {
    #[serde(rename = "ID")]
    pub id: u64,
    #[serde(rename = "CreatedAt")]
    pub created_at: i64,
    #[serde(rename = "UpdatedAt")]
    pub updated_at: i64,
    pub title: String,
    pub content: String,
    #[serde(rename = "userID")]
    pub user_id: u64,
    pub attachments: Vec<AnnouncementAttachment>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AnnouncementListRequest {
    #[serde(default)]
    pub page: Option<usize>,
    #[serde(rename = "pageSize", default)]
    pub page_size: Option<usize>,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AnnouncementUpsertRequest {
    #[serde(rename = "ID", default)]
    pub id: Option<u64>,
    pub title: String,
    pub content: String,
    #[serde(rename = "userID")]
    pub user_id: u64,
    #[serde(default)]
    pub attachments: Vec<AnnouncementAttachment>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AnnouncementIdRequest {
    #[serde(rename = "ID")]
    pub id: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AnnouncementIdsRequest {
    #[serde(rename = "IDs", default)]
    pub ids: Vec<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelectOption {
    pub label: String,
    pub value: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnnouncementDataSourcePayload {
    #[serde(rename = "userID")]
    pub user_id: Vec<SelectOption>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadFileRecord {
    #[serde(rename = "ID")]
    pub id: u64,
    pub name: String,
    pub size: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadedFileAsset {
    #[serde(rename = "ID")]
    pub id: u64,
    #[serde(rename = "classId")]
    pub class_id: u64,
    pub key: String,
    pub name: String,
    pub url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadedFilePayload {
    pub file: UploadedFileAsset,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UploadQueueRequest {
    #[serde(default)]
    pub files: Vec<UploadQueueItem>,
    #[serde(default)]
    pub mode: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UploadQueueItem {
    pub name: String,
    pub size: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResumeUploadRecord {
    #[serde(rename = "ID")]
    pub id: u64,
    pub name: String,
    pub progress: u32,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanSessionRecord {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ScanSessionUpdateRequest {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomerRecord {
    #[serde(rename = "ID")]
    pub id: u64,
    #[serde(rename = "CreatedAt")]
    pub created_at: i64,
    #[serde(rename = "UpdatedAt")]
    pub updated_at: i64,
    #[serde(rename = "customerName")]
    pub customer_name: String,
    #[serde(rename = "customerPhoneData")]
    pub customer_phone_data: String,
    #[serde(rename = "sysUserId")]
    pub sys_user_id: u64,
    #[serde(rename = "customerLevel")]
    pub customer_level: String,
    #[serde(rename = "customerStatus")]
    pub customer_status: String,
    pub remark: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CustomerListRequest {
    #[serde(default)]
    pub page: Option<usize>,
    #[serde(rename = "pageSize", default)]
    pub page_size: Option<usize>,
    #[serde(default)]
    pub keyword: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CustomerUpsertRequest {
    #[serde(rename = "ID", default)]
    pub id: Option<u64>,
    #[serde(rename = "customerName")]
    pub customer_name: String,
    #[serde(rename = "customerPhoneData")]
    pub customer_phone_data: String,
    #[serde(rename = "sysUserId", default)]
    pub sys_user_id: Option<u64>,
    #[serde(rename = "customerLevel", default)]
    pub customer_level: Option<String>,
    #[serde(rename = "customerStatus", default)]
    pub customer_status: Option<String>,
    #[serde(default)]
    pub remark: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CustomerIdRequest {
    #[serde(rename = "ID")]
    pub id: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomerPayload {
    pub customer: CustomerRecord,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpToolParam {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default)]
    pub required: bool,
    #[serde(rename = "defaultValue", default)]
    pub default_value: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpToolOutput {
    #[serde(rename = "type")]
    pub type_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpToolRecord {
    #[serde(rename = "ID")]
    pub id: u64,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    pub name: String,
    pub description: String,
    pub params: Vec<McpToolParam>,
    pub response: Vec<McpToolOutput>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct McpToolUpsertRequest {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub params: Vec<McpToolParam>,
    #[serde(default)]
    pub response: Vec<McpToolOutput>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpSchemaProperty {
    #[serde(rename = "type")]
    pub type_name: String,
    pub description: String,
    #[serde(rename = "defaultValue", skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpInputSchema {
    #[serde(rename = "type")]
    pub type_name: String,
    pub properties: std::collections::BTreeMap<String, McpSchemaProperty>,
    pub required: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpToolDescriptor {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: McpInputSchema,
    pub response: Vec<McpToolOutput>,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpToolListPayload {
    pub tools: Vec<McpToolDescriptor>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpServiceStatusPayload {
    pub managed: bool,
    pub state: String,
    pub reachable: bool,
    #[serde(rename = "baseURL")]
    pub base_url: String,
    #[serde(rename = "healthURL")]
    pub health_url: String,
    #[serde(rename = "startedAt")]
    pub started_at: Option<i64>,
    #[serde(rename = "lastError")]
    pub last_error: String,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct McpTestRequest {
    pub name: String,
    #[serde(default)]
    pub args: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpTestResult {
    pub tool: String,
    pub accepted: bool,
    pub validation: String,
    #[serde(rename = "executedAt")]
    pub executed_at: i64,
    pub output: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillAssetRecord {
    pub name: String,
    pub content: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillRecord {
    pub name: String,
    pub description: String,
    #[serde(rename = "allowedTools")]
    pub allowed_tools: String,
    pub context: String,
    pub agent: String,
    pub markdown: String,
    pub enabled: bool,
    pub tags: Vec<String>,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    pub scripts: Vec<SkillAssetRecord>,
    pub resources: Vec<SkillAssetRecord>,
    pub references: Vec<SkillAssetRecord>,
    pub templates: Vec<SkillAssetRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillSummary {
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub tags: Vec<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillListPayload {
    pub skills: Vec<SkillSummary>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillDetailPayload {
    pub skill: SkillRecord,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SkillDetailRequest {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SkillSaveRequest {
    pub name: String,
    pub description: String,
    #[serde(rename = "allowedTools", default)]
    pub allowed_tools: String,
    #[serde(default)]
    pub context: String,
    #[serde(default)]
    pub agent: String,
    #[serde(default)]
    pub markdown: String,
    #[serde(default = "default_enabled_true")]
    pub enabled: bool,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_enabled_true() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize)]
pub struct SkillDeleteRequest {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SkillAssetCreateRequest {
    #[serde(rename = "skillName")]
    pub skill_name: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SkillAssetReadRequest {
    #[serde(rename = "skillName")]
    pub skill_name: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SkillAssetSaveRequest {
    #[serde(rename = "skillName")]
    pub skill_name: String,
    pub name: String,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillAssetPayload {
    pub file: SkillAssetRecord,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillToolDefinition {
    pub key: String,
    pub label: String,
    pub summary: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillToolListPayload {
    pub tools: Vec<SkillToolDefinition>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlobalConstraintRecord {
    pub content: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlobalConstraintPayload {
    #[serde(rename = "globalConstraint")]
    pub global_constraint: GlobalConstraintRecord,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GlobalConstraintSaveRequest {
    pub content: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AiDecisionRequest {
    pub mode: String,
    #[serde(rename = "articleId")]
    pub article_id: String,
    pub action: String,
    pub reason: String,
    #[serde(rename = "serviceToken")]
    pub service_token: Option<String>,
    pub delegation: Option<DelegationInput>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DelegationInput {
    #[serde(rename = "operatorUserId")]
    pub operator_user_id: u64,
    pub scope: String,
}

#[derive(Clone, Debug)]
pub struct UserAuthStateRecord {
    pub user_id: u64,
    pub token_version: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct AuditEvent {
    #[serde(rename = "auditId")]
    pub audit_id: String,
    pub actor: String,
    #[serde(rename = "effectiveRole")]
    pub effective_role: String,
    pub target: String,
    pub action: String,
    pub reason: String,
    #[serde(rename = "workflowRunId")]
    pub workflow_run_id: String,
    pub timestamp: i64,
}

#[derive(Clone, Debug)]
pub struct ServiceIdentity {
    pub token: String,
    pub name: String,
    pub scopes: Vec<String>,
    pub ai_role: u32,
}

pub fn ok<T>(data: T, msg: &str) -> ApiResponse<T> {
    ApiResponse {
        code: 0,
        msg: msg.to_string(),
        data,
    }
}

pub fn fail<T>(data: T, msg: &str) -> ApiResponse<T> {
    ApiResponse {
        code: 7,
        msg: msg.to_string(),
        data,
    }
}
