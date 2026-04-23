use axum::body::Body;
use axum::extract::{Multipart, Query, State};
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, HeaderValue, Request, StatusCode};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::{Json, response::Response};
use uuid::Uuid;

use crate::auth::{extract_cookie_value, now_ts};
use crate::models::{
    AiDecisionRequest, AnnouncementDataSourcePayload, AnnouncementIdRequest,
    AnnouncementIdsRequest, AnnouncementListRequest, AnnouncementRecord, AnnouncementUpsertRequest,
    ApiListRequest, ApiResponse, ApiTokenIssueRequest, ApiUpsertRequest,
    ArticleCategoryUpsertRequest, ArticleUpsertRequest, AuditEvent, AuthorityButtonMatrixRequest,
    AuthorityButtonMatrixSelection, AuthorityPolicyUpdateRequest, AuthorityRequest,
    AuthorityRoleUsersRequest, AuthorityUpsertRequest, AutoCodeRegistryUpsertRequest,
    ConsoleMenuUpsertRequest, CustomerIdRequest, CustomerListRequest, CustomerPayload,
    CustomerUpsertRequest, DecisionPayload, DictionaryDetailUpsertRequest, DictionaryUpsertRequest,
    EmailSendRequest, FrontendNavUpsertRequest, FrontendSettingsUpdateRequest,
    GlobalConstraintPayload, GlobalConstraintSaveRequest, IdRequest, LlmConfigUpsertRequest,
    LoginPayload, LoginRequest, McpServiceStatusPayload, McpTestRequest, McpToolListPayload,
    McpToolUpsertRequest, MemberUpsertRequest, MenuAuthorityAssignmentRequest,
    MenuAuthorityPayload, MenuRoleIdsPayload, MenuRoleUpdateRequest, MenuTreePayload,
    MenuUpsertRequest, PackageUpsertRequest, ParamUpsertRequest, PluginInstallRequest,
    PluginManifestUpsertRequest, ProfileUpdateRequest, RefreshRequest, ReleaseAppendRequest,
    RuntimeInfoPayload, ScanSessionUpdateRequest, SkillAssetCreateRequest, SkillAssetPayload,
    SkillAssetReadRequest, SkillAssetSaveRequest, SkillDeleteRequest, SkillDetailPayload,
    SkillDetailRequest, SkillListPayload, SkillSaveRequest, SkillToolListPayload,
    SwitchAuthorityPayload, SwitchAuthorityRequest, SystemConfigUpdateRequest, UploadQueueRequest,
    UploadedFileAsset, UploadedFilePayload, UserInfoPayload, UserListRequest, UserUpsertRequest,
    fail, ok,
};
use crate::state::AppState;

const ACCESS_TOKEN_COOKIE: &str = "gaa_access_token";
const REFRESH_TOKEN_COOKIE: &str = "gaa_refresh_token";

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

fn read_bool_env(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .map(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(default)
}

fn read_cookie_same_site() -> String {
    let value = std::env::var("GAA_COOKIE_SAMESITE").unwrap_or_else(|_| "Lax".to_string());
    match value.to_ascii_lowercase().as_str() {
        "strict" => "Strict".to_string(),
        "none" => "None".to_string(),
        _ => "Lax".to_string(),
    }
}

fn read_cookie_domain() -> Option<String> {
    std::env::var("GAA_COOKIE_DOMAIN")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn build_auth_cookie(name: &str, value: &str, max_age_sec: i64) -> String {
    let mut parts = vec![
        format!("{}={}", name, value),
        "Path=/".to_string(),
        "HttpOnly".to_string(),
        format!("Max-Age={}", max_age_sec.max(0)),
        format!("SameSite={}", read_cookie_same_site()),
    ];
    if read_bool_env("GAA_COOKIE_SECURE", is_production_env()) {
        parts.push("Secure".to_string());
    }
    if let Some(domain) = read_cookie_domain() {
        parts.push(format!("Domain={}", domain));
    }
    parts.join("; ")
}

fn append_cookie(headers: &mut HeaderMap, cookie: String) {
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        headers.append(SET_COOKIE, value);
    }
}

fn append_access_cookie(headers: &mut HeaderMap, token: &str, max_age_sec: i64) {
    append_cookie(
        headers,
        build_auth_cookie(ACCESS_TOKEN_COOKIE, token, max_age_sec),
    );
}

fn append_refresh_cookie(headers: &mut HeaderMap, token: &str, max_age_sec: i64) {
    append_cookie(
        headers,
        build_auth_cookie(REFRESH_TOKEN_COOKIE, token, max_age_sec),
    );
}

fn append_clear_auth_cookies(headers: &mut HeaderMap) {
    append_cookie(
        headers,
        build_auth_cookie(ACCESS_TOKEN_COOKIE, "deleted", 0),
    );
    append_cookie(
        headers,
        build_auth_cookie(REFRESH_TOKEN_COOKIE, "deleted", 0),
    );
}

fn resolve_access_token(headers: &HeaderMap) -> Option<String> {
    let header_token = crate::auth::extract_token(
        headers.get("x-token").and_then(|v| v.to_str().ok()),
        headers.get("authorization").and_then(|v| v.to_str().ok()),
    );
    header_token.or_else(|| {
        if read_bool_env("GAA_ALLOW_COOKIE_AUTH", false) {
            extract_cookie_value(
                headers.get("cookie").and_then(|v| v.to_str().ok()),
                ACCESS_TOKEN_COOKIE,
            )
        } else {
            None
        }
    })
}

pub async fn enforce_route_access(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let path = request.uri().path().to_string();
    let method = request.method().as_str().to_string();

    if is_public_route(&path) || method == "OPTIONS" {
        return Ok(next.run(request).await);
    }

    let token = resolve_access_token(request.headers())
        .ok_or_else(|| auth_error(StatusCode::UNAUTHORIZED, "未登录或非法访问，请登录"))?;
    let auth = state
        .authorize_human(&token)
        .await
        .map_err(|err| auth_error(StatusCode::UNAUTHORIZED, &err))?;

    if !state
        .is_allowed(auth.user.authority_id, &path, &method)
        .await
    {
        return Err(auth_error(StatusCode::FORBIDDEN, "权限不足"));
    }

    Ok(next.run(request).await)
}

fn is_public_route(path: &str) -> bool {
    matches!(path, "/healthz" | "/base/login" | "/base/refresh")
        || path.starts_with("/public/")
        || path == "/ai/moderation/decision"
}

fn auth_error(status: StatusCode, message: &str) -> Response {
    (status, Json(fail(serde_json::json!({}), message))).into_response()
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Response, (StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let _ = body.captcha;
    let _ = body.captcha_id;
    match state.login(&body.username, &body.password).await {
        Ok((user, token, expires_at, refresh_token)) => {
            let payload = ok(
                LoginPayload {
                    user,
                    token: token.clone(),
                    expires_at,
                    refresh_token: refresh_token.clone(),
                },
                "登录成功",
            );
            let mut response = Json(payload).into_response();
            append_access_cookie(
                response.headers_mut(),
                &token,
                state.config.auth.access_ttl_sec,
            );
            append_refresh_cookie(
                response.headers_mut(),
                &refresh_token,
                state.config.auth.refresh_ttl_sec,
            );
            Ok(response)
        }
        Err(err) => Err((
            StatusCode::UNAUTHORIZED,
            Json(fail(serde_json::json!({}), &err)),
        )),
    }
}

pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<RefreshRequest>,
) -> Result<Response, (StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let refresh_token = if body.refresh_token.trim().is_empty() {
        extract_cookie_value(
            headers.get("cookie").and_then(|v| v.to_str().ok()),
            REFRESH_TOKEN_COOKIE,
        )
        .unwrap_or_default()
    } else {
        body.refresh_token
    };

    if refresh_token.trim().is_empty() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(fail(serde_json::json!({}), "refresh token缺失")),
        ));
    }

    match state.refresh(&refresh_token).await {
        Ok((token, exp)) => {
            let payload = ok(
                serde_json::json!({
                    "token": token,
                    "expiresAt": exp * 1000
                }),
                "刷新成功",
            );
            let mut response = Json(payload).into_response();
            append_access_cookie(
                response.headers_mut(),
                &token,
                state.config.auth.access_ttl_sec,
            );
            Ok(response)
        }
        Err(err) => Err((
            StatusCode::UNAUTHORIZED,
            Json(fail(serde_json::json!({}), &err)),
        )),
    }
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, (StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let token = resolve_access_token(&headers);

    if let Some(token) = token {
        let auth = state.authenticate_human(&token).await.map_err(|err| {
            (
                StatusCode::UNAUTHORIZED,
                Json(fail(serde_json::json!({}), &err)),
            )
        })?;

        state.revoke_session(&auth.session_id).await;
    }
    let mut response = Json(ok(serde_json::json!({}), "登出成功")).into_response();
    append_clear_auth_cookies(response.headers_mut());
    Ok(response)
}

pub async fn user_info(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, (StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let token = resolve_access_token(&headers).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(fail(serde_json::json!({}), "未登录或非法访问，请登录")),
    ))?;

    let auth = state.authenticate_human(&token).await.map_err(|err| {
        (
            StatusCode::UNAUTHORIZED,
            Json(fail(serde_json::json!({}), &err)),
        )
    })?;

    let payload = ok(
        UserInfoPayload {
            user_info: auth.user,
        },
        "获取成功",
    );

    let mut response = Json(payload).into_response();
    if state.config.compatibility_refresh_headers {
        if let Some(token) = auth.new_token {
            response.headers_mut().insert(
                "new-token",
                HeaderValue::from_str(&token).unwrap_or_else(|_| HeaderValue::from_static("")),
            );
            append_access_cookie(
                response.headers_mut(),
                &token,
                state.config.auth.access_ttl_sec,
            );
        }
        if let Some(exp) = auth.new_expires_at {
            response.headers_mut().insert(
                "new-expires-at",
                HeaderValue::from_str(&exp.to_string())
                    .unwrap_or_else(|_| HeaderValue::from_static("0")),
            );
        }
    }

    Ok(response)
}

pub async fn user_policy_paths(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AuthorityRequest>,
) -> Result<Response, (StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let token = resolve_access_token(&headers).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(fail(serde_json::json!({}), "未登录或非法访问，请登录")),
    ))?;

    let auth = state.authenticate_human(&token).await.map_err(|err| {
        (
            StatusCode::UNAUTHORIZED,
            Json(fail(serde_json::json!({}), &err)),
        )
    })?;

    if !state
        .is_allowed(
            auth.user.authority_id,
            "/casbin/getPolicyPathByAuthorityId",
            "POST",
        )
        .await
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(fail(serde_json::json!({}), "权限不足")),
        ));
    }

    let policies = state.get_policy_paths(body.authority_id).await;
    let mut response = Json(ok(policies, "获取成功")).into_response();
    if state.config.compatibility_refresh_headers {
        if let Some(token) = auth.new_token {
            response.headers_mut().insert(
                "new-token",
                HeaderValue::from_str(&token).unwrap_or_else(|_| HeaderValue::from_static("")),
            );
            append_access_cookie(
                response.headers_mut(),
                &token,
                state.config.auth.access_ttl_sec,
            );
        }
        if let Some(exp) = auth.new_expires_at {
            response.headers_mut().insert(
                "new-expires-at",
                HeaderValue::from_str(&exp.to_string())
                    .unwrap_or_else(|_| HeaderValue::from_static("0")),
            );
        }
    }
    Ok(response)
}

pub async fn user_policy_paths_set(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AuthorityPolicyUpdateRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    state
        .set_policy_paths(body.authority_id, body.policies)
        .await;
    Ok(Json(ok(serde_json::json!({}), "保存成功")))
}

pub async fn user_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<UserListRequest>,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::UserInfo>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_users(body).await, "获取成功")))
}

pub async fn authority_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<Vec<crate::models::AuthorityInfo>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_authorities().await, "获取成功")))
}

pub async fn authority_upsert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AuthorityUpsertRequest>,
) -> Result<
    Json<ApiResponse<crate::models::AuthorityInfo>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    let item = state.upsert_authority(body).await.map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            Json(fail(serde_json::json!({}), &err)),
        )
    })?;
    Ok(Json(ok(item, "保存成功")))
}

pub async fn authority_users(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AuthorityRequest>,
) -> Result<Json<ApiResponse<Vec<u64>>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        state.get_user_ids_by_authority(query.authority_id).await,
        "获取成功",
    )))
}

pub async fn authority_set_role_users(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AuthorityRoleUsersRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    state
        .set_role_users(body.authority_id, body.user_ids)
        .await
        .map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                Json(fail(serde_json::json!({}), &err)),
            )
        })?;
    Ok(Json(ok(serde_json::json!({}), "设置成功")))
}

pub async fn menu_tree(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<MenuTreePayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        MenuTreePayload {
            menus: state.list_menu_tree().await,
        },
        "获取成功",
    )))
}

pub async fn menu_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::MenuInfo>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    let menus = flatten_menus(&state.list_menu_tree().await);
    Ok(Json(ok(
        crate::models::PageResult {
            total: menus.len(),
            list: menus,
            page: 1,
            page_size: 999,
        },
        "获取成功",
    )))
}

pub async fn menu_upsert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<MenuUpsertRequest>,
) -> Result<
    Json<ApiResponse<crate::models::MenuInfo>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.upsert_menu(body).await, "保存成功")))
}

pub async fn menu_authority_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AuthorityRequest>,
) -> Result<
    Json<ApiResponse<MenuAuthorityPayload>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        MenuAuthorityPayload {
            menus: state.get_menu_authority(body.authority_id).await,
        },
        "获取成功",
    )))
}

pub async fn menu_authority_set(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<MenuAuthorityAssignmentRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    let menu_ids = body.menus.into_iter().map(|menu| menu.id).collect();
    state
        .set_menu_authority(body.authority_id, menu_ids)
        .await
        .map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                Json(fail(serde_json::json!({}), &err)),
            )
        })?;
    Ok(Json(ok(serde_json::json!({}), "添加成功")))
}

pub async fn menu_roles_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<crate::models::MenuIdRequest>,
) -> Result<Json<ApiResponse<MenuRoleIdsPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    let (authority_ids, default_router_authority_ids) = state.get_menu_roles(query.menu_id).await;
    Ok(Json(ok(
        MenuRoleIdsPayload {
            authority_ids,
            default_router_authority_ids,
        },
        "获取成功",
    )))
}

pub async fn menu_roles_set(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<MenuRoleUpdateRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    state
        .set_menu_roles(body.menu_id, body.authority_ids)
        .await
        .map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                Json(fail(serde_json::json!({}), &err)),
            )
        })?;
    Ok(Json(ok(serde_json::json!({}), "保存成功")))
}

pub async fn authority_btn_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AuthorityButtonMatrixRequest>,
) -> Result<
    Json<ApiResponse<AuthorityButtonMatrixSelection>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        AuthorityButtonMatrixSelection {
            selected: state
                .get_authority_buttons(body.authority_id, body.menu_id)
                .await,
        },
        "查询成功",
    )))
}

pub async fn authority_btn_set(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AuthorityButtonMatrixRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    state
        .set_authority_buttons(body.authority_id, body.menu_id, body.selected)
        .await
        .map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                Json(fail(serde_json::json!({}), &err)),
            )
        })?;
    Ok(Json(ok(serde_json::json!({}), "分配成功")))
}

pub async fn api_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ApiListRequest>,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::ApiInfo>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_api_entries(body).await, "获取成功")))
}

pub async fn dictionary_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::DictionaryInfo>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_dictionaries().await, "获取成功")))
}

pub async fn api_upsert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ApiUpsertRequest>,
) -> Result<
    Json<ApiResponse<crate::models::ApiInfo>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.upsert_api(body).await, "保存成功")))
}

pub async fn dictionary_upsert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<DictionaryUpsertRequest>,
) -> Result<
    Json<ApiResponse<crate::models::DictionaryInfo>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.upsert_dictionary(body).await, "保存成功")))
}

pub async fn params_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::ParamInfo>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_params().await, "获取成功")))
}

pub async fn params_upsert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ParamUpsertRequest>,
) -> Result<
    Json<ApiResponse<crate::models::ParamInfo>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.upsert_param(body).await, "保存成功")))
}

pub async fn operation_logs(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::OperationLogInfo>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_operation_logs(), "获取成功")))
}

pub async fn login_logs(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::LoginLogInfo>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_login_logs(), "获取成功")))
}

pub async fn dictionary_detail_tree(
    State(state): State<AppState>,
    headers: HeaderMap,
    query: axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<
    Json<ApiResponse<Vec<crate::models::DictionaryDetailInfo>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    let dictionary_id = query
        .get("sysDictionaryID")
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(1);
    Ok(Json(ok(
        state.dictionary_detail_tree(dictionary_id).await,
        "获取成功",
    )))
}

pub async fn dictionary_detail_upsert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<DictionaryDetailUpsertRequest>,
) -> Result<
    Json<ApiResponse<crate::models::DictionaryDetailInfo>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        state.upsert_dictionary_detail(body).await,
        "保存成功",
    )))
}

pub async fn error_logs(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::ErrorLogInfo>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_error_logs(), "获取成功")))
}

pub async fn export_template_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::ExportTemplateInfo>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_export_templates(), "获取成功")))
}

pub async fn system_runtime(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<RuntimeInfoPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        RuntimeInfoPayload {
            server: state.runtime_info(),
        },
        "获取成功",
    )))
}

pub async fn system_config(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::SystemConfigInfo>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.system_config_info().await, "获取成功")))
}

pub async fn update_system_config(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SystemConfigUpdateRequest>,
) -> Result<
    Json<ApiResponse<crate::models::SystemConfigInfo>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.update_system_config(body).await, "保存成功")))
}

pub async fn llm_config_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::LlmConfigRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_llm_configs().await, "获取成功")))
}

pub async fn llm_config_save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<LlmConfigUpsertRequest>,
) -> Result<
    Json<ApiResponse<crate::models::LlmConfigRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.upsert_llm_config(body).await, "保存成功")))
}

pub async fn llm_config_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<IdRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    if state.delete_llm_config(body.id).await {
        Ok(Json(ok(serde_json::json!({}), "删除成功")))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(fail(serde_json::json!({}), "模型配置不存在")),
        ))
    }
}

pub async fn frontend_nav_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::FrontendNavRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_frontend_nav().await, "获取成功")))
}

pub async fn frontend_nav_save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<FrontendNavUpsertRequest>,
) -> Result<
    Json<ApiResponse<crate::models::FrontendNavRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.upsert_frontend_nav(body).await, "保存成功")))
}

pub async fn frontend_nav_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<IdRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    if state.delete_frontend_nav(body.id).await {
        Ok(Json(ok(serde_json::json!({}), "删除成功")))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(fail(serde_json::json!({}), "导航不存在")),
        ))
    }
}

pub async fn frontend_settings_get(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::FrontendSettingsRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.frontend_settings().await, "获取成功")))
}

pub async fn frontend_settings_save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<FrontendSettingsUpdateRequest>,
) -> Result<
    Json<ApiResponse<crate::models::FrontendSettingsRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        state.update_frontend_settings(body).await,
        "保存成功",
    )))
}

pub async fn public_frontend_settings_get(
    State(state): State<AppState>,
) -> Json<ApiResponse<crate::models::FrontendSettingsRecord>> {
    Json(ok(state.frontend_settings().await, "获取成功"))
}

pub async fn public_frontend_nav_list(
    State(state): State<AppState>,
) -> Json<ApiResponse<crate::models::PageResult<crate::models::FrontendNavRecord>>> {
    Json(ok(state.list_frontend_nav().await, "获取成功"))
}

pub async fn public_article_list(
    State(state): State<AppState>,
) -> Json<ApiResponse<crate::models::PageResult<crate::models::ArticleRecord>>> {
    Json(ok(state.list_articles().await, "获取成功"))
}

pub async fn article_category_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::ArticleCategoryRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_article_categories().await, "获取成功")))
}

pub async fn article_category_save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ArticleCategoryUpsertRequest>,
) -> Result<
    Json<ApiResponse<crate::models::ArticleCategoryRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        state.upsert_article_category(body).await,
        "保存成功",
    )))
}

pub async fn article_category_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<IdRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    if state.delete_article_category(body.id).await {
        Ok(Json(ok(serde_json::json!({}), "删除成功")))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(fail(serde_json::json!({}), "文章分类不存在")),
        ))
    }
}

pub async fn article_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::ArticleRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_articles().await, "获取成功")))
}

pub async fn article_save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ArticleUpsertRequest>,
) -> Result<
    Json<ApiResponse<crate::models::ArticleRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.upsert_article(body).await, "保存成功")))
}

pub async fn article_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<IdRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    if state.delete_article(body.id).await {
        Ok(Json(ok(serde_json::json!({}), "删除成功")))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(fail(serde_json::json!({}), "文章不存在")),
        ))
    }
}

pub async fn member_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::MemberRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_members().await, "获取成功")))
}

pub async fn member_save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<MemberUpsertRequest>,
) -> Result<
    Json<ApiResponse<crate::models::MemberRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.upsert_member(body).await, "保存成功")))
}

pub async fn member_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<IdRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    if state.delete_member(body.id).await {
        Ok(Json(ok(serde_json::json!({}), "删除成功")))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(fail(serde_json::json!({}), "会员不存在")),
        ))
    }
}

pub async fn console_menu_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::ConsoleMenuRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_console_menus().await, "获取成功")))
}

pub async fn console_menu_save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ConsoleMenuUpsertRequest>,
) -> Result<
    Json<ApiResponse<crate::models::ConsoleMenuRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.upsert_console_menu(body).await, "保存成功")))
}

pub async fn console_menu_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<IdRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    if state.delete_console_menu(body.id).await {
        Ok(Json(ok(serde_json::json!({}), "删除成功")))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(fail(serde_json::json!({}), "控制台菜单不存在")),
        ))
    }
}

pub async fn api_token_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::ApiTokenRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_api_tokens().await, "获取成功")))
}

pub async fn api_token_issue(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ApiTokenIssueRequest>,
) -> Result<
    Json<ApiResponse<crate::models::ApiTokenRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.issue_api_token(body).await, "保存成功")))
}

pub async fn api_token_clear(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    state.clear_api_tokens().await;
    Ok(Json(ok(serde_json::json!({}), "清空成功")))
}

pub async fn package_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::PackageRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_packages().await, "获取成功")))
}

pub async fn package_upsert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PackageUpsertRequest>,
) -> Result<
    Json<ApiResponse<crate::models::PackageRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.upsert_package(body).await, "保存成功")))
}

pub async fn plugin_manifest_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::PluginManifestRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_plugin_manifests().await, "获取成功")))
}

pub async fn plugin_manifest_upsert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PluginManifestUpsertRequest>,
) -> Result<
    Json<ApiResponse<crate::models::PluginManifestRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        state.upsert_plugin_manifest(body).await,
        "保存成功",
    )))
}

pub async fn plugin_install_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::PluginInstallRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_plugin_installs().await, "获取成功")))
}

pub async fn plugin_install_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PluginInstallRequest>,
) -> Result<
    Json<ApiResponse<crate::models::PluginInstallRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.install_plugin(body).await, "保存成功")))
}

pub async fn auto_code_registry_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::AutoCodeRegistryRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_auto_code_registry().await, "获取成功")))
}

pub async fn auto_code_registry_append(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AutoCodeRegistryUpsertRequest>,
) -> Result<
    Json<ApiResponse<crate::models::AutoCodeRegistryRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        state.append_auto_code_registry(body).await,
        "保存成功",
    )))
}

pub async fn auto_code_registry_clear(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    state.clear_auto_code_registry().await;
    Ok(Json(ok(serde_json::json!({}), "清空成功")))
}

pub async fn release_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::ReleaseRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_releases().await, "获取成功")))
}

pub async fn release_append(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ReleaseAppendRequest>,
) -> Result<
    Json<ApiResponse<crate::models::ReleaseRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.append_release(body).await, "保存成功")))
}

pub async fn email_test(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<EmailSendRequest>,
) -> Result<
    Json<ApiResponse<crate::models::EmailRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.send_email(body, "test").await, "发送成功")))
}

pub async fn email_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::EmailRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_email_records().await, "获取成功")))
}

pub async fn email_send(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<EmailSendRequest>,
) -> Result<
    Json<ApiResponse<crate::models::EmailRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.send_email(body, "send").await, "发送成功")))
}

pub async fn announcement_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    query: axum::extract::Query<AnnouncementListRequest>,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<AnnouncementRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        state.list_announcements(query.0).await,
        "获取成功",
    )))
}

pub async fn announcement_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    query: axum::extract::Query<AnnouncementIdRequest>,
) -> Result<Json<ApiResponse<AnnouncementRecord>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    let record = state.get_announcement(query.id).await.ok_or((
        StatusCode::NOT_FOUND,
        Json(fail(serde_json::json!({}), "公告不存在")),
    ))?;
    Ok(Json(ok(record, "获取成功")))
}

pub async fn announcement_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AnnouncementUpsertRequest>,
) -> Result<Json<ApiResponse<AnnouncementRecord>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.upsert_announcement(body).await, "创建成功")))
}

pub async fn announcement_update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AnnouncementUpsertRequest>,
) -> Result<Json<ApiResponse<AnnouncementRecord>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.upsert_announcement(body).await, "更新成功")))
}

pub async fn announcement_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    query: axum::extract::Query<AnnouncementIdRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    if state.delete_announcement(query.id).await {
        Ok(Json(ok(serde_json::json!({}), "删除成功")))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(fail(serde_json::json!({}), "公告不存在")),
        ))
    }
}

pub async fn announcement_delete_many(
    State(state): State<AppState>,
    headers: HeaderMap,
    query: axum::extract::Query<AnnouncementIdsRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    let deleted = state.delete_announcements(query.ids.clone()).await;
    Ok(Json(ok(
        serde_json::json!({ "deleted": deleted }),
        "删除成功",
    )))
}

pub async fn announcement_data_source(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<AnnouncementDataSourcePayload>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.announcement_data_source().await, "获取成功")))
}

pub async fn upload_queue_replace(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<UploadQueueRequest>,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::UploadFileRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.replace_upload_queue(body).await, "保存成功")))
}

pub async fn upload_file_asset(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<
    Json<ApiResponse<UploadedFilePayload>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;

    let mut file_name = format!("upload-{}.bin", now_ts());
    let mut class_id = 0_u64;

    while let Some(field) = multipart.next_field().await.map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            Json(fail(
                serde_json::json!({}),
                &format!("读取上传字段失败: {err}"),
            )),
        )
    })? {
        let name = field.name().unwrap_or_default().to_string();
        if name == "classId" {
            let value = field.text().await.unwrap_or_default();
            class_id = value.parse::<u64>().unwrap_or(0);
            continue;
        }
        if name == "file" {
            if let Some(candidate) = field.file_name() {
                file_name = candidate.to_string();
            }
            let _ = field.bytes().await.map_err(|err| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(fail(
                        serde_json::json!({}),
                        &format!("读取上传文件失败: {err}"),
                    )),
                )
            })?;
        }
    }

    let key = format!("uploads/{}-{}", now_ts(), sanitize_upload_name(&file_name));
    let file = UploadedFileAsset {
        id: now_ts() as u64,
        class_id,
        key: key.clone(),
        name: file_name,
        url: format!("/{}", key),
    };

    Ok(Json(ok(UploadedFilePayload { file }, "上传成功")))
}

pub async fn upload_queue_complete(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::UploadFileRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.complete_upload_queue().await, "上传成功")))
}

pub async fn resume_upload_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::ResumeUploadRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_resume_uploads().await, "获取成功")))
}

pub async fn resume_upload_advance(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::ResumeUploadRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.advance_resume_uploads().await, "推进成功")))
}

pub async fn resume_upload_resume(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::ResumeUploadRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        state.resume_interrupted_upload().await,
        "恢复成功",
    )))
}

pub async fn scan_session_get(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::ScanSessionRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        state.get_or_create_scan_session().await,
        "获取成功",
    )))
}

pub async fn scan_session_update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ScanSessionUpdateRequest>,
) -> Result<
    Json<ApiResponse<crate::models::ScanSessionRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.update_scan_session(body).await, "保存成功")))
}

pub async fn customer_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    query: axum::extract::Query<CustomerListRequest>,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::CustomerRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_customers(query.0).await, "获取成功")))
}

pub async fn customer_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    query: axum::extract::Query<CustomerIdRequest>,
) -> Result<Json<ApiResponse<CustomerPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    let customer = state.get_customer(query.id).await.ok_or((
        StatusCode::NOT_FOUND,
        Json(fail(serde_json::json!({}), "客户不存在")),
    ))?;
    Ok(Json(ok(CustomerPayload { customer }, "获取成功")))
}

pub async fn customer_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CustomerUpsertRequest>,
) -> Result<Json<ApiResponse<CustomerPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    let auth = require_auth(&state, &headers).await?;
    let customer = state.upsert_customer(body, auth.user.id).await;
    Ok(Json(ok(CustomerPayload { customer }, "创建成功")))
}

pub async fn customer_update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CustomerUpsertRequest>,
) -> Result<Json<ApiResponse<CustomerPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    let auth = require_auth(&state, &headers).await?;
    let customer = state.upsert_customer(body, auth.user.id).await;
    Ok(Json(ok(CustomerPayload { customer }, "更新成功")))
}

pub async fn customer_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CustomerIdRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    if state.delete_customer(body.id).await {
        Ok(Json(ok(serde_json::json!({}), "删除成功")))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(fail(serde_json::json!({}), "客户不存在")),
        ))
    }
}

pub async fn mcp_tool_upsert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<McpToolUpsertRequest>,
) -> Result<
    Json<ApiResponse<crate::models::McpToolRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.save_mcp_tool(body).await, "保存成功")))
}

pub async fn mcp_service_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<McpServiceStatusPayload>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.mcp_service_status().await, "获取成功")))
}

pub async fn mcp_service_start(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<McpServiceStatusPayload>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.start_mcp_service().await, "启动成功")))
}

pub async fn mcp_service_stop(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<McpServiceStatusPayload>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.stop_mcp_service().await, "停用成功")))
}

pub async fn mcp_tool_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<McpToolListPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        McpToolListPayload {
            tools: state.list_mcp_tools().await,
        },
        "获取成功",
    )))
}

pub async fn mcp_tool_test(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<McpTestRequest>,
) -> Result<
    Json<ApiResponse<crate::models::McpTestResult>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    let result = state.test_mcp_tool(body).await.map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            Json(fail(serde_json::json!({}), &err)),
        )
    })?;
    Ok(Json(ok(result, "执行成功")))
}

pub async fn skill_tools(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<SkillToolListPayload>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        SkillToolListPayload {
            tools: state.list_skill_tools().await,
        },
        "获取成功",
    )))
}

pub async fn skill_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<SkillListPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        SkillListPayload {
            skills: state.list_skills().await,
        },
        "获取成功",
    )))
}

pub async fn skill_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SkillDetailRequest>,
) -> Result<Json<ApiResponse<SkillDetailPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    let skill = state.get_skill_detail(&body.name).await.ok_or((
        StatusCode::NOT_FOUND,
        Json(fail(serde_json::json!({}), "技能不存在")),
    ))?;
    Ok(Json(ok(SkillDetailPayload { skill }, "获取成功")))
}

pub async fn skill_save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SkillSaveRequest>,
) -> Result<Json<ApiResponse<SkillDetailPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    let skill = state.save_skill(body).await;
    Ok(Json(ok(SkillDetailPayload { skill }, "保存成功")))
}

pub async fn skill_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SkillDeleteRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    if state.delete_skill(&body.name).await {
        Ok(Json(ok(serde_json::json!({}), "删除成功")))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(fail(serde_json::json!({}), "技能不存在")),
        ))
    }
}

async fn create_skill_asset_by_kind(
    state: &AppState,
    headers: &HeaderMap,
    kind: &str,
    body: SkillAssetCreateRequest,
) -> Result<Json<ApiResponse<SkillAssetPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(state, headers).await?;
    let file = state
        .create_skill_asset(&body.skill_name, kind, &body.name)
        .await
        .map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                Json(fail(serde_json::json!({}), &err)),
            )
        })?;
    Ok(Json(ok(SkillAssetPayload { file }, "创建成功")))
}

async fn get_skill_asset_by_kind(
    state: &AppState,
    headers: &HeaderMap,
    kind: &str,
    body: SkillAssetReadRequest,
) -> Result<Json<ApiResponse<SkillAssetPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(state, headers).await?;
    let file = state
        .get_skill_asset(&body.skill_name, kind, &body.name)
        .await
        .map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                Json(fail(serde_json::json!({}), &err)),
            )
        })?;
    Ok(Json(ok(SkillAssetPayload { file }, "获取成功")))
}

async fn save_skill_asset_by_kind(
    state: &AppState,
    headers: &HeaderMap,
    kind: &str,
    body: SkillAssetSaveRequest,
) -> Result<Json<ApiResponse<SkillAssetPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(state, headers).await?;
    let file = state
        .save_skill_asset(&body.skill_name, kind, &body.name, body.content)
        .await
        .map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                Json(fail(serde_json::json!({}), &err)),
            )
        })?;
    Ok(Json(ok(SkillAssetPayload { file }, "保存成功")))
}

pub async fn skill_script_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SkillAssetCreateRequest>,
) -> Result<Json<ApiResponse<SkillAssetPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    create_skill_asset_by_kind(&state, &headers, "script", body).await
}

pub async fn skill_script_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SkillAssetReadRequest>,
) -> Result<Json<ApiResponse<SkillAssetPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    get_skill_asset_by_kind(&state, &headers, "script", body).await
}

pub async fn skill_script_save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SkillAssetSaveRequest>,
) -> Result<Json<ApiResponse<SkillAssetPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    save_skill_asset_by_kind(&state, &headers, "script", body).await
}

pub async fn skill_resource_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SkillAssetCreateRequest>,
) -> Result<Json<ApiResponse<SkillAssetPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    create_skill_asset_by_kind(&state, &headers, "resource", body).await
}

pub async fn skill_resource_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SkillAssetReadRequest>,
) -> Result<Json<ApiResponse<SkillAssetPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    get_skill_asset_by_kind(&state, &headers, "resource", body).await
}

pub async fn skill_resource_save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SkillAssetSaveRequest>,
) -> Result<Json<ApiResponse<SkillAssetPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    save_skill_asset_by_kind(&state, &headers, "resource", body).await
}

pub async fn skill_reference_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SkillAssetCreateRequest>,
) -> Result<Json<ApiResponse<SkillAssetPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    create_skill_asset_by_kind(&state, &headers, "reference", body).await
}

pub async fn skill_reference_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SkillAssetReadRequest>,
) -> Result<Json<ApiResponse<SkillAssetPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    get_skill_asset_by_kind(&state, &headers, "reference", body).await
}

pub async fn skill_reference_save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SkillAssetSaveRequest>,
) -> Result<Json<ApiResponse<SkillAssetPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    save_skill_asset_by_kind(&state, &headers, "reference", body).await
}

pub async fn skill_template_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SkillAssetCreateRequest>,
) -> Result<Json<ApiResponse<SkillAssetPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    create_skill_asset_by_kind(&state, &headers, "template", body).await
}

pub async fn skill_template_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SkillAssetReadRequest>,
) -> Result<Json<ApiResponse<SkillAssetPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    get_skill_asset_by_kind(&state, &headers, "template", body).await
}

pub async fn skill_template_save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SkillAssetSaveRequest>,
) -> Result<Json<ApiResponse<SkillAssetPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    save_skill_asset_by_kind(&state, &headers, "template", body).await
}

pub async fn global_constraint_get(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<GlobalConstraintPayload>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        GlobalConstraintPayload {
            global_constraint: state.get_global_constraint().await,
        },
        "获取成功",
    )))
}

pub async fn global_constraint_save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<GlobalConstraintSaveRequest>,
) -> Result<
    Json<ApiResponse<GlobalConstraintPayload>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        GlobalConstraintPayload {
            global_constraint: state.save_global_constraint(body.content).await,
        },
        "保存成功",
    )))
}

pub async fn update_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ProfileUpdateRequest>,
) -> Result<Json<ApiResponse<UserInfoPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    let token = resolve_access_token(&headers).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(fail(serde_json::json!({}), "未登录或非法访问，请登录")),
    ))?;

    let auth = state.authenticate_human(&token).await.map_err(|err| {
        (
            StatusCode::UNAUTHORIZED,
            Json(fail(serde_json::json!({}), &err)),
        )
    })?;

    let user = state
        .update_profile(auth.user.id, body)
        .await
        .map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                Json(fail(serde_json::json!({}), &err)),
            )
        })?;

    Ok(Json(ok(UserInfoPayload { user_info: user }, "保存成功")))
}

pub async fn user_upsert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<UserUpsertRequest>,
) -> Result<Json<ApiResponse<UserInfoPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    let user = state.upsert_user(body).await.map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            Json(fail(serde_json::json!({}), &err)),
        )
    })?;
    Ok(Json(ok(UserInfoPayload { user_info: user }, "保存成功")))
}

pub async fn switch_authority(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SwitchAuthorityRequest>,
) -> Result<Response, (StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let token = resolve_access_token(&headers).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(fail(serde_json::json!({}), "未登录或非法访问，请登录")),
    ))?;

    let auth = state.authenticate_human(&token).await.map_err(|err| {
        (
            StatusCode::UNAUTHORIZED,
            Json(fail(serde_json::json!({}), &err)),
        )
    })?;

    if !state
        .is_allowed(auth.user.authority_id, "/user/setUserAuthority", "POST")
        .await
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(fail(serde_json::json!({}), "权限不足")),
        ));
    }

    let (user, new_token, expires_at) = state
        .switch_authority(&auth.session_id, auth.user.id, body.authority_id)
        .await
        .map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                Json(fail(serde_json::json!({}), &err)),
            )
        })?;

    let payload = ok(
        SwitchAuthorityPayload {
            user,
            token: new_token.clone(),
            expires_at,
        },
        "修改成功",
    );
    let mut response = Json(payload).into_response();
    if state.config.compatibility_refresh_headers {
        response.headers_mut().insert(
            "new-token",
            HeaderValue::from_str(&new_token).unwrap_or_else(|_| HeaderValue::from_static("")),
        );
        response.headers_mut().insert(
            "new-expires-at",
            HeaderValue::from_str(&(expires_at / 1000).to_string())
                .unwrap_or_else(|_| HeaderValue::from_static("0")),
        );
    }
    append_access_cookie(
        response.headers_mut(),
        &new_token,
        state.config.auth.access_ttl_sec,
    );
    Ok(response)
}

pub async fn post_ai_moderation_decision(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AiDecisionRequest>,
) -> Result<Json<ApiResponse<DecisionPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    let (actor, effective_role) = match body.mode.as_str() {
        "system" => {
            let service_token = body.service_token.clone().ok_or((
                StatusCode::UNAUTHORIZED,
                Json(fail(serde_json::json!({}), "system模式缺少service token")),
            ))?;
            let service = state
                .resolve_service_identity(&service_token)
                .await
                .ok_or((
                    StatusCode::UNAUTHORIZED,
                    Json(fail(serde_json::json!({}), "service token无效")),
                ))?;
            let needed_scope = if body.target_is_user_or_crawler() {
                "user.violation.handle"
            } else {
                "article.review"
            };
            if !service.scopes.iter().any(|s| s == needed_scope) {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(fail(serde_json::json!({}), "service scope不足")),
                ));
            }
            (
                format!("service:{}", service.name),
                service.ai_role.to_string(),
            )
        }
        "delegated" => {
            let access_token = resolve_access_token(&headers).ok_or((
                StatusCode::UNAUTHORIZED,
                Json(fail(serde_json::json!({}), "未登录或非法访问，请登录")),
            ))?;

            let auth = state
                .authenticate_human(&access_token)
                .await
                .map_err(|err| {
                    (
                        StatusCode::UNAUTHORIZED,
                        Json(fail(serde_json::json!({}), &err)),
                    )
                })?;

            let delegation = body.delegation.clone().ok_or((
                StatusCode::BAD_REQUEST,
                Json(fail(serde_json::json!({}), "delegated模式缺少delegation")),
            ))?;

            if delegation.operator_user_id != auth.user.id {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(fail(serde_json::json!({}), "委托用户不匹配")),
                ));
            }
            if delegation.scope.is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(fail(serde_json::json!({}), "delegation scope为空")),
                ));
            }
            if !state
                .is_allowed(auth.user.authority_id, "/ai/moderation/decision", "POST")
                .await
            {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(fail(serde_json::json!({}), "权限不足")),
                ));
            }
            (
                format!("user:{}", auth.user.user_name),
                auth.user.authority_id.to_string(),
            )
        }
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(fail(serde_json::json!({}), "mode仅支持system/delegated")),
            ));
        }
    };

    let audit_id = Uuid::new_v4().to_string();
    state
        .append_audit(AuditEvent {
            audit_id: audit_id.clone(),
            actor,
            effective_role,
            target: body.article_id.clone(),
            action: body.action.clone(),
            reason: body.reason.clone(),
            workflow_run_id: Uuid::new_v4().to_string(),
            timestamp: now_ts(),
        })
        .await;

    Ok(Json(ok(
        DecisionPayload {
            accepted: true,
            audit_id,
        },
        "AI处置已记录",
    )))
}

trait DecisionTargetExt {
    fn target_is_user_or_crawler(&self) -> bool;
}

impl DecisionTargetExt for AiDecisionRequest {
    fn target_is_user_or_crawler(&self) -> bool {
        let lower = self.article_id.to_lowercase();
        lower.contains("user") || lower.contains("crawler")
    }
}

async fn require_auth(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::state::Authenticated, (StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let token = resolve_access_token(headers).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(fail(serde_json::json!({}), "未登录或非法访问，请登录")),
    ))?;

    state.authenticate_human(&token).await.map_err(|err| {
        (
            StatusCode::UNAUTHORIZED,
            Json(fail(serde_json::json!({}), &err)),
        )
    })
}

fn flatten_menus(items: &[crate::models::MenuInfo]) -> Vec<crate::models::MenuInfo> {
    let mut out = Vec::new();
    for item in items {
        out.push(item.clone());
        out.extend(flatten_menus(&item.children));
    }
    out
}

fn sanitize_upload_name(name: &str) -> String {
    let sanitized = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if sanitized.is_empty() {
        "file.bin".to_string()
    } else {
        sanitized
    }
}

fn is_production_env() -> bool {
    std::env::var("GAA_ENV")
        .or_else(|_| std::env::var("RUST_ENV"))
        .or_else(|_| std::env::var("APP_ENV"))
        .map(|value| matches!(value.to_ascii_lowercase().as_str(), "prod" | "production"))
        .unwrap_or(false)
}
