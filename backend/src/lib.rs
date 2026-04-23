pub mod auth;
pub mod casbin_port;
pub mod handlers;
pub mod models;
pub mod session_registry;
pub mod state;
pub mod storage;

use axum::Router;
use axum::http::Method;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderName, HeaderValue};
use axum::middleware;
use axum::routing::{get, post};
use handlers::{
    announcement_create, announcement_data_source, announcement_delete, announcement_delete_many,
    announcement_get, announcement_list, announcement_update, api_list, api_token_clear,
    api_token_issue, api_token_list, api_upsert, article_category_delete, article_category_list,
    article_category_save, article_delete, article_list, article_save, authority_btn_get,
    authority_btn_set, authority_list, authority_set_role_users, authority_upsert, authority_users,
    auto_code_registry_append, auto_code_registry_clear, auto_code_registry_list,
    console_menu_delete, console_menu_list, console_menu_save, customer_create, customer_delete,
    customer_get, customer_list, customer_update, dictionary_detail_tree, dictionary_detail_upsert,
    dictionary_list, dictionary_upsert, email_list, email_send, email_test, enforce_route_access,
    error_logs, export_template_list, frontend_nav_delete, frontend_nav_list, frontend_nav_save,
    frontend_settings_get, frontend_settings_save, global_constraint_get, global_constraint_save,
    health, llm_config_delete, llm_config_list, llm_config_save, login, login_logs, logout,
    mcp_service_start, mcp_service_status, mcp_service_stop, mcp_tool_list, mcp_tool_test,
    mcp_tool_upsert, member_delete, member_list, member_save, menu_authority_get,
    menu_authority_set, menu_list, menu_roles_get, menu_roles_set, menu_tree, menu_upsert,
    operation_logs, package_list, package_upsert, params_list, params_upsert,
    plugin_install_create, plugin_install_list, plugin_manifest_list, plugin_manifest_upsert,
    post_ai_moderation_decision, public_article_list, public_frontend_nav_list,
    public_frontend_settings_get, refresh, release_append, release_list, resume_upload_advance,
    resume_upload_list, resume_upload_resume, scan_session_get, scan_session_update, skill_delete,
    skill_detail, skill_list, skill_reference_create, skill_reference_get, skill_reference_save,
    skill_resource_create, skill_resource_get, skill_resource_save, skill_save,
    skill_script_create, skill_script_get, skill_script_save, skill_template_create,
    skill_template_get, skill_template_save, skill_tools, switch_authority, system_config,
    system_runtime, update_profile, update_system_config, upload_file_asset, upload_queue_complete,
    upload_queue_replace, user_info, user_list, user_policy_paths, user_policy_paths_set,
    user_upsert,
};
use state::AppState;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub fn build_router(state: AppState) -> Router {
    let allowed_origins = [
        HeaderValue::from_static("http://127.0.0.1:5666"),
        HeaderValue::from_static("http://localhost:5666"),
        HeaderValue::from_static("http://127.0.0.1:3000"),
        HeaderValue::from_static("http://localhost:3000"),
    ];
    let allowed_headers = [
        AUTHORIZATION,
        CONTENT_TYPE,
        HeaderName::from_static("x-token"),
        HeaderName::from_static("x-csrf-token"),
    ];
    let allowed_methods = [
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::OPTIONS,
    ];

    Router::new()
        .route("/healthz", get(health))
        .route("/base/login", post(login))
        .route("/base/refresh", post(refresh))
        .route("/base/logout", post(logout))
        .route("/user/getUserList", post(user_list))
        .route("/user/setUserAuthority", post(switch_authority))
        .route("/user/switchAuthority", post(switch_authority))
        .route("/user/getUserInfo", get(user_info))
        .route("/user/updateProfile", post(update_profile))
        .route("/user/saveUser", post(user_upsert))
        .route("/authority/getAuthorityList", post(authority_list))
        .route("/authority/saveAuthority", post(authority_upsert))
        .route("/authority/getUsersByAuthority", get(authority_users))
        .route("/authority/setRoleUsers", post(authority_set_role_users))
        .route("/authorityBtn/getAuthorityBtn", post(authority_btn_get))
        .route("/authorityBtn/setAuthorityBtn", post(authority_btn_set))
        .route("/menu/getMenu", post(menu_tree))
        .route("/menu/getMenuList", post(menu_list))
        .route("/menu/saveMenu", post(menu_upsert))
        .route("/menu/getMenuAuthority", post(menu_authority_get))
        .route("/menu/addMenuAuthority", post(menu_authority_set))
        .route("/menu/getMenuRoles", get(menu_roles_get))
        .route("/menu/setMenuRoles", post(menu_roles_set))
        .route("/api/getApiList", post(api_list))
        .route("/api/saveApi", post(api_upsert))
        .route("/sysDictionary/getSysDictionaryList", get(dictionary_list))
        .route("/sysDictionary/saveSysDictionary", post(dictionary_upsert))
        .route(
            "/sysDictionaryDetail/getDictionaryTreeList",
            get(dictionary_detail_tree),
        )
        .route(
            "/sysDictionaryDetail/saveSysDictionaryDetail",
            post(dictionary_detail_upsert),
        )
        .route("/sysParams/getSysParamsList", get(params_list))
        .route("/sysParams/saveSysParams", post(params_upsert))
        .route(
            "/sysOperationRecord/getSysOperationRecordList",
            get(operation_logs),
        )
        .route("/sysLoginLog/getLoginLogList", get(login_logs))
        .route("/sysError/getSysErrorList", get(error_logs))
        .route(
            "/sysExportTemplate/getSysExportTemplateList",
            get(export_template_list),
        )
        .route("/system/getServerInfo", post(system_runtime))
        .route("/system/getSystemConfig", post(system_config))
        .route("/system/setSystemConfig", post(update_system_config))
        .route("/tool/api-token/list", get(api_token_list))
        .route("/tool/api-token/issue", post(api_token_issue))
        .route("/tool/api-token/clear", post(api_token_clear))
        .route("/tool/llm-config/list", get(llm_config_list))
        .route("/tool/llm-config/save", post(llm_config_save))
        .route("/tool/llm-config/delete", post(llm_config_delete))
        .route("/tool/package/list", get(package_list))
        .route("/tool/package/save", post(package_upsert))
        .route("/tool/plugin-manifest/list", get(plugin_manifest_list))
        .route("/tool/plugin-manifest/save", post(plugin_manifest_upsert))
        .route("/tool/plugin-install/list", get(plugin_install_list))
        .route("/tool/plugin-install/save", post(plugin_install_create))
        .route("/tool/auto-code/list", get(auto_code_registry_list))
        .route("/tool/auto-code/save", post(auto_code_registry_append))
        .route("/tool/auto-code/clear", post(auto_code_registry_clear))
        .route("/tool/release/list", get(release_list))
        .route("/tool/release/save", post(release_append))
        .route("/email/getEmailList", get(email_list))
        .route("/email/emailTest", post(email_test))
        .route("/email/sendEmail", post(email_send))
        .route("/info/getInfoDataSource", get(announcement_data_source))
        .route("/info/getInfoList", get(announcement_list))
        .route("/info/findInfo", get(announcement_get))
        .route("/info/createInfo", post(announcement_create))
        .route(
            "/info/updateInfo",
            post(announcement_update).put(announcement_update),
        )
        .route(
            "/info/deleteInfo",
            axum::routing::delete(announcement_delete),
        )
        .route(
            "/info/deleteInfoByIds",
            axum::routing::delete(announcement_delete_many),
        )
        .route("/frontend/nav/list", get(frontend_nav_list))
        .route("/frontend/nav/save", post(frontend_nav_save))
        .route("/frontend/nav/delete", post(frontend_nav_delete))
        .route("/frontend/settings/get", get(frontend_settings_get))
        .route("/frontend/settings/save", post(frontend_settings_save))
        .route(
            "/frontend/article-category/list",
            get(article_category_list),
        )
        .route(
            "/frontend/article-category/save",
            post(article_category_save),
        )
        .route(
            "/frontend/article-category/delete",
            post(article_category_delete),
        )
        .route("/frontend/article/list", get(article_list))
        .route("/frontend/article/save", post(article_save))
        .route("/frontend/article/delete", post(article_delete))
        .route("/frontend/member/list", get(member_list))
        .route("/frontend/member/save", post(member_save))
        .route("/frontend/member/delete", post(member_delete))
        .route("/frontend/console-menu/list", get(console_menu_list))
        .route("/frontend/console-menu/save", post(console_menu_save))
        .route("/frontend/console-menu/delete", post(console_menu_delete))
        .route("/fileUploadAndDownload/upload", post(upload_file_asset))
        .route(
            "/public/frontend/settings",
            get(public_frontend_settings_get),
        )
        .route("/public/frontend/nav", get(public_frontend_nav_list))
        .route("/public/frontend/articles", get(public_article_list))
        .route("/example/upload/list", post(upload_queue_replace))
        .route("/example/upload/complete", post(upload_queue_complete))
        .route("/example/resume/list", get(resume_upload_list))
        .route("/example/resume/advance", post(resume_upload_advance))
        .route("/example/resume/recover", post(resume_upload_resume))
        .route("/example/scan/session", get(scan_session_get))
        .route("/example/scan/session", post(scan_session_update))
        .route("/customer/customerList", get(customer_list))
        .route(
            "/customer/customer",
            get(customer_get)
                .post(customer_create)
                .put(customer_update)
                .delete(customer_delete),
        )
        .route("/autoCode/mcp", post(mcp_tool_upsert))
        .route("/autoCode/mcpStatus", post(mcp_service_status))
        .route("/autoCode/mcpStart", post(mcp_service_start))
        .route("/autoCode/mcpStop", post(mcp_service_stop))
        .route("/autoCode/mcpList", post(mcp_tool_list))
        .route("/autoCode/mcpTest", post(mcp_tool_test))
        .route("/skills/getTools", get(skill_tools))
        .route("/skills/getSkillList", post(skill_list))
        .route("/skills/getSkillDetail", post(skill_detail))
        .route("/skills/saveSkill", post(skill_save))
        .route("/skills/deleteSkill", post(skill_delete))
        .route("/skills/createScript", post(skill_script_create))
        .route("/skills/getScript", post(skill_script_get))
        .route("/skills/saveScript", post(skill_script_save))
        .route("/skills/createResource", post(skill_resource_create))
        .route("/skills/getResource", post(skill_resource_get))
        .route("/skills/saveResource", post(skill_resource_save))
        .route("/skills/createReference", post(skill_reference_create))
        .route("/skills/getReference", post(skill_reference_get))
        .route("/skills/saveReference", post(skill_reference_save))
        .route("/skills/createTemplate", post(skill_template_create))
        .route("/skills/getTemplate", post(skill_template_get))
        .route("/skills/saveTemplate", post(skill_template_save))
        .route("/skills/getGlobalConstraint", post(global_constraint_get))
        .route("/skills/saveGlobalConstraint", post(global_constraint_save))
        .route(
            "/casbin/getPolicyPathByAuthorityId",
            post(user_policy_paths),
        )
        .route(
            "/casbin/setPolicyPathByAuthorityId",
            post(user_policy_paths_set),
        )
        .route("/ai/moderation/decision", post(post_ai_moderation_decision))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            enforce_route_access,
        ))
        .layer(
            CorsLayer::new()
                .allow_methods(allowed_methods)
                .allow_headers(allowed_headers)
                .allow_origin(allowed_origins)
                .allow_credentials(true),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::header::CONTENT_TYPE;
    use axum::http::{Request, StatusCode};
    use serde_json::{Value, json};
    use tower::ServiceExt;

    async fn response_json(response: axum::response::Response) -> Value {
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read body failed");
        serde_json::from_slice(&body_bytes).expect("json parse failed")
    }

    async fn login_tokens(app: &Router) -> (String, String) {
        let login_request = Request::builder()
            .method("POST")
            .uri("/base/login")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "username": "admin",
                    "password": "123456"
                })
                .to_string(),
            ))
            .expect("request build failed");
        let login_response = app
            .clone()
            .oneshot(login_request)
            .await
            .expect("request failed");
        assert_eq!(login_response.status(), StatusCode::OK);
        let payload = response_json(login_response).await;
        let access_token = payload["data"]["token"]
            .as_str()
            .expect("access token missing")
            .to_string();
        let refresh_token = payload["data"]["refreshToken"]
            .as_str()
            .expect("refresh token missing")
            .to_string();
        (access_token, refresh_token)
    }

    #[tokio::test]
    async fn login_success_returns_token() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);

        let request = Request::builder()
            .method("POST")
            .uri("/base/login")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "username": "admin",
                    "password": "123456"
                })
                .to_string(),
            ))
            .expect("request build failed");

        let response = app.oneshot(request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn login_disabled_user_rejected() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);

        let request = Request::builder()
            .method("POST")
            .uri("/base/login")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "username": "disabled",
                    "password": "123456"
                })
                .to_string(),
            ))
            .expect("request build failed");

        let response = app.oneshot(request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn get_user_info_requires_valid_token() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);

        let request = Request::builder()
            .method("GET")
            .uri("/user/getUserInfo")
            .body(Body::empty())
            .expect("request build failed");
        let response = app.clone().oneshot(request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let (token, _) = login_tokens(&app).await;

        let user_request = Request::builder()
            .method("GET")
            .uri("/user/getUserInfo")
            .header("x-token", token)
            .body(Body::empty())
            .expect("request build failed");
        let user_response = app.oneshot(user_request).await.expect("request failed");
        assert_eq!(user_response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn save_user_replaces_authorities_with_payload_values() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let save_request = Request::builder()
            .method("POST")
            .uri("/user/saveUser")
            .header(CONTENT_TYPE, "application/json")
            .header("x-token", token.clone())
            .body(Body::from(
                json!({
                    "ID": 1,
                    "userName": "admin",
                    "nickName": "GAA管理员",
                    "authorityId": 888,
                    "authorityIds": [888],
                    "phone": "13800138000",
                    "email": "admin@gaa.local",
                    "enable": 1
                })
                .to_string(),
            ))
            .expect("request build failed");
        let save_response = app
            .clone()
            .oneshot(save_request)
            .await
            .expect("request failed");
        assert_eq!(save_response.status(), StatusCode::OK);
        let save_payload = response_json(save_response).await;
        let save_authorities = save_payload["data"]["userInfo"]["authorities"]
            .as_array()
            .expect("save authorities missing");
        assert_eq!(save_authorities.len(), 1);
        assert_eq!(save_authorities[0]["authorityId"], json!(888));

        let list_request = Request::builder()
            .method("POST")
            .uri("/user/getUserList")
            .header(CONTENT_TYPE, "application/json")
            .header("x-token", token)
            .body(Body::from(
                json!({
                    "page": 1,
                    "pageSize": 20
                })
                .to_string(),
            ))
            .expect("request build failed");
        let list_response = app.oneshot(list_request).await.expect("request failed");
        assert_eq!(list_response.status(), StatusCode::OK);
        let list_payload = response_json(list_response).await;
        let list_authorities = list_payload["data"]["List"][0]["authorities"]
            .as_array()
            .expect("list authorities missing");
        assert_eq!(list_authorities.len(), 1);
        assert_eq!(list_authorities[0]["authorityId"], json!(888));
    }

    #[tokio::test]
    async fn refresh_returns_new_access_token() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (_, refresh_token) = login_tokens(&app).await;

        let request = Request::builder()
            .method("POST")
            .uri("/base/refresh")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "refreshToken": refresh_token
                })
                .to_string(),
            ))
            .expect("request build failed");
        let response = app.oneshot(request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
        let payload = response_json(response).await;
        assert!(payload["data"]["token"].as_str().is_some());
    }

    #[tokio::test]
    async fn logout_revokes_current_session() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let logout_request = Request::builder()
            .method("POST")
            .uri("/base/logout")
            .header("x-token", token.clone())
            .body(Body::empty())
            .expect("request build failed");
        let logout_response = app
            .clone()
            .oneshot(logout_request)
            .await
            .expect("request failed");
        assert_eq!(logout_response.status(), StatusCode::OK);

        let user_request = Request::builder()
            .method("GET")
            .uri("/user/getUserInfo")
            .header("x-token", token)
            .body(Body::empty())
            .expect("request build failed");
        let user_response = app.oneshot(user_request).await.expect("request failed");
        assert_eq!(user_response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn policy_query_is_business_compatible() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let policy_request = Request::builder()
            .method("POST")
            .uri("/casbin/getPolicyPathByAuthorityId")
            .header("x-token", token)
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "authorityId": 888
                })
                .to_string(),
            ))
            .expect("request build failed");
        let policy_response = app.oneshot(policy_request).await.expect("request failed");
        assert_eq!(policy_response.status(), StatusCode::OK);
        let payload = response_json(policy_response).await;
        let items = payload["data"].as_array().expect("policy array missing");
        assert!(
            items
                .iter()
                .any(|item| item["path"] == "/user/getUserInfo" && item["method"] == "GET"),
            "legacy-compatible policy path missing"
        );
    }

    #[tokio::test]
    async fn user_info_emits_rolling_refresh_header_when_near_expiry() {
        let mut state = AppState::seed().await.expect("seed failed");
        state.config.auth.refresh_buffer_sec = state.config.auth.access_ttl_sec;
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let request = Request::builder()
            .method("GET")
            .uri("/user/getUserInfo")
            .header("x-token", token)
            .body(Body::empty())
            .expect("request build failed");
        let response = app.oneshot(request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().get("new-token").is_some());
        assert!(response.headers().get("new-expires-at").is_some());
    }

    #[tokio::test]
    async fn ai_system_mode_accepts_service_identity() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);

        let request = Request::builder()
            .method("POST")
            .uri("/ai/moderation/decision")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "mode": "system",
                    "articleId": "article-100",
                    "action": "approve",
                    "reason": "policy-safe",
                    "serviceToken": "svc-moderation-token"
                })
                .to_string(),
            ))
            .expect("request build failed");
        let response = app.oneshot(request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
        let payload = response_json(response).await;
        assert_eq!(payload["data"]["accepted"], true);
        assert!(payload["data"]["auditId"].as_str().is_some());
    }

    #[tokio::test]
    async fn ai_delegated_mode_rejects_wrong_operator() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let request = Request::builder()
            .method("POST")
            .uri("/ai/moderation/decision")
            .header("x-token", token)
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "mode": "delegated",
                    "articleId": "user-violation-42",
                    "action": "ban",
                    "reason": "abuse pattern",
                    "delegation": {
                        "operatorUserId": 999,
                        "scope": "user.violation.handle"
                    }
                })
                .to_string(),
            ))
            .expect("request build failed");
        let response = app.oneshot(request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn switch_authority_updates_user_and_token() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let switch_request = Request::builder()
            .method("POST")
            .uri("/user/switchAuthority")
            .header("x-token", token)
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "authorityId": 9528
                })
                .to_string(),
            ))
            .expect("request build failed");
        let switch_response = app
            .clone()
            .oneshot(switch_request)
            .await
            .expect("request failed");
        assert_eq!(switch_response.status(), StatusCode::OK);
        let payload = response_json(switch_response).await;
        assert_eq!(payload["data"]["user"]["authorityId"], 9528);
        assert!(payload["data"]["token"].as_str().is_some());
    }

    #[tokio::test]
    async fn non_admin_role_cannot_access_system_config() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let switch_request = Request::builder()
            .method("POST")
            .uri("/user/switchAuthority")
            .header("x-token", token)
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "authorityId": 9528
                })
                .to_string(),
            ))
            .expect("request build failed");
        let switch_response = app
            .clone()
            .oneshot(switch_request)
            .await
            .expect("request failed");
        assert_eq!(switch_response.status(), StatusCode::OK);
        let payload = response_json(switch_response).await;
        let ai_token = payload["data"]["token"]
            .as_str()
            .expect("switched token missing");

        let config_request = Request::builder()
            .method("POST")
            .uri("/system/getSystemConfig")
            .header("x-token", ai_token)
            .body(Body::empty())
            .expect("request build failed");
        let config_response = app.oneshot(config_request).await.expect("request failed");
        assert_eq!(config_response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn multipoint_login_invalidates_previous_session_when_enabled() {
        let state = AppState::seed_with_config(crate::state::AppConfig {
            multipoint_enabled: true,
            ..crate::state::AppConfig::default()
        })
        .await
        .expect("seed failed");
        let app = build_router(state);

        let (first_token, _) = login_tokens(&app).await;
        let (second_token, _) = login_tokens(&app).await;

        let stale_request = Request::builder()
            .method("GET")
            .uri("/user/getUserInfo")
            .header("x-token", first_token)
            .body(Body::empty())
            .expect("request build failed");
        let stale_response = app
            .clone()
            .oneshot(stale_request)
            .await
            .expect("request failed");
        assert_eq!(stale_response.status(), StatusCode::UNAUTHORIZED);

        let current_request = Request::builder()
            .method("GET")
            .uri("/user/getUserInfo")
            .header("x-token", second_token)
            .body(Body::empty())
            .expect("request build failed");
        let current_response = app.oneshot(current_request).await.expect("request failed");
        assert_eq!(current_response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn admin_list_endpoints_return_data() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let user_request = Request::builder()
            .method("POST")
            .uri("/user/getUserList")
            .header("x-token", token.clone())
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({"page":1,"pageSize":10}).to_string()))
            .expect("request build failed");
        let user_response = app
            .clone()
            .oneshot(user_request)
            .await
            .expect("request failed");
        assert_eq!(user_response.status(), StatusCode::OK);

        let authority_request = Request::builder()
            .method("POST")
            .uri("/authority/getAuthorityList")
            .header("x-token", token.clone())
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from("{}"))
            .expect("request build failed");
        let authority_response = app
            .clone()
            .oneshot(authority_request)
            .await
            .expect("request failed");
        assert_eq!(authority_response.status(), StatusCode::OK);

        let menu_request = Request::builder()
            .method("POST")
            .uri("/menu/getMenu")
            .header("x-token", token.clone())
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from("{}"))
            .expect("request build failed");
        let menu_response = app
            .clone()
            .oneshot(menu_request)
            .await
            .expect("request failed");
        assert_eq!(menu_response.status(), StatusCode::OK);

        let api_request = Request::builder()
            .method("POST")
            .uri("/api/getApiList")
            .header("x-token", token)
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({"page":1,"pageSize":10}).to_string()))
            .expect("request build failed");
        let api_response = app.oneshot(api_request).await.expect("request failed");
        assert_eq!(api_response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn email_and_announcement_endpoints_round_trip() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let email_request = Request::builder()
            .method("POST")
            .uri("/email/sendEmail")
            .header("x-token", token.clone())
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "to":"notice@gaa.local",
                    "subject":"Parity Check",
                    "body":"Announcement route enabled"
                })
                .to_string(),
            ))
            .expect("request build failed");
        let email_response = app
            .clone()
            .oneshot(email_request)
            .await
            .expect("request failed");
        assert_eq!(email_response.status(), StatusCode::OK);

        let datasource_request = Request::builder()
            .method("GET")
            .uri("/info/getInfoDataSource")
            .header("x-token", token.clone())
            .body(Body::empty())
            .expect("request build failed");
        let datasource_response = app
            .clone()
            .oneshot(datasource_request)
            .await
            .expect("request failed");
        assert_eq!(datasource_response.status(), StatusCode::OK);

        let create_request = Request::builder()
            .method("POST")
            .uri("/info/createInfo")
            .header("x-token", token.clone())
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "title":"Rust parity bulletin",
                    "content":"Plugin views restored",
                    "userID":1,
                    "attachments":[{"name":"matrix.md","url":"/docs/page-parity-matrix.md"}]
                })
                .to_string(),
            ))
            .expect("request build failed");
        let create_response = app
            .clone()
            .oneshot(create_request)
            .await
            .expect("request failed");
        assert_eq!(create_response.status(), StatusCode::OK);

        let list_request = Request::builder()
            .method("GET")
            .uri("/info/getInfoList?page=1&pageSize=10")
            .header("x-token", token)
            .body(Body::empty())
            .expect("request build failed");
        let list_response = app.oneshot(list_request).await.expect("request failed");
        assert_eq!(list_response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn save_user_updates_enable_state() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let request = Request::builder()
            .method("POST")
            .uri("/user/saveUser")
            .header(CONTENT_TYPE, "application/json")
            .header("x-token", token)
            .body(Body::from(
                json!({
                    "ID": 2,
                    "userName": "disabled",
                    "nickName": "禁用账户已恢复",
                    "authorityId": 888,
                    "phone": "13900139000",
                    "email": "disabled@gaa.local",
                    "enable": 1
                })
                .to_string(),
            ))
            .expect("request build failed");

        let response = app.oneshot(request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
        let payload = response_json(response).await;
        assert_eq!(payload["data"]["userInfo"]["enable"], 1);
        assert_eq!(payload["data"]["userInfo"]["nickName"], "禁用账户已恢复");
    }

    #[tokio::test]
    async fn save_dictionary_detail_appends_child_node() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let save_request = Request::builder()
            .method("POST")
            .uri("/sysDictionaryDetail/saveSysDictionaryDetail")
            .header(CONTENT_TYPE, "application/json")
            .header("x-token", token.clone())
            .body(Body::from(
                json!({
                    "sysDictionaryID": 2,
                    "label": "代理异常",
                    "value": "proxy_drift",
                    "extend": "crawler",
                    "level": 2,
                    "status": true,
                    "sort": 3,
                    "parentID": 21
                })
                .to_string(),
            ))
            .expect("request build failed");
        let save_response = app
            .clone()
            .oneshot(save_request)
            .await
            .expect("request failed");
        assert_eq!(save_response.status(), StatusCode::OK);

        let list_request = Request::builder()
            .method("GET")
            .uri("/sysDictionaryDetail/getDictionaryTreeList?sysDictionaryID=2")
            .header("x-token", token)
            .body(Body::empty())
            .expect("request build failed");
        let list_response = app.oneshot(list_request).await.expect("request failed");
        assert_eq!(list_response.status(), StatusCode::OK);
        let payload = response_json(list_response).await;
        let node_json = serde_json::to_string(&payload["data"]).expect("serialize payload failed");
        assert!(node_json.contains("proxy_drift"));
    }

    #[tokio::test]
    async fn save_authority_updates_default_router() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let request = Request::builder()
            .method("POST")
            .uri("/authority/saveAuthority")
            .header(CONTENT_TYPE, "application/json")
            .header("x-token", token)
            .body(Body::from(
                json!({
                    "ID": 2,
                    "authorityId": 9528,
                    "authorityName": "AI运营角色",
                    "defaultRouter": "authorities",
                    "parentId": 0
                })
                .to_string(),
            ))
            .expect("request build failed");

        let response = app.oneshot(request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
        let payload = response_json(response).await;
        assert_eq!(payload["data"]["defaultRouter"], "authorities");
    }

    #[tokio::test]
    async fn save_authority_updates_status_without_explicit_id() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let request = Request::builder()
            .method("POST")
            .uri("/authority/saveAuthority")
            .header(CONTENT_TYPE, "application/json")
            .header("x-token", token)
            .body(Body::from(
                json!({
                    "authorityId": 9528,
                    "authorityName": "AI运营角色",
                    "defaultRouter": "dashboard",
                    "parentId": 0,
                    "enable": 2,
                    "status": 2
                })
                .to_string(),
            ))
            .expect("request build failed");

        let response = app.oneshot(request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
        let payload = response_json(response).await;
        assert_eq!(payload["data"]["authorityId"], 9528);
        assert_eq!(payload["data"]["enable"], 2);
        assert_eq!(payload["data"]["status"], 2);
    }

    #[tokio::test]
    async fn authority_permission_endpoints_round_trip() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let get_menu_request = Request::builder()
            .method("POST")
            .uri("/menu/getMenuAuthority")
            .header(CONTENT_TYPE, "application/json")
            .header("x-token", token.clone())
            .body(Body::from(json!({ "authorityId": 9528 }).to_string()))
            .expect("request build failed");
        let get_menu_response = app
            .clone()
            .oneshot(get_menu_request)
            .await
            .expect("request failed");
        assert_eq!(get_menu_response.status(), StatusCode::OK);

        let set_button_request = Request::builder()
            .method("POST")
            .uri("/authorityBtn/setAuthorityBtn")
            .header(CONTENT_TYPE, "application/json")
            .header("x-token", token.clone())
            .body(Body::from(
                json!({
                    "authorityId": 9528,
                    "menuID": 22,
                    "selected": [2201, 2202]
                })
                .to_string(),
            ))
            .expect("request build failed");
        let set_button_response = app
            .clone()
            .oneshot(set_button_request)
            .await
            .expect("request failed");
        assert_eq!(set_button_response.status(), StatusCode::OK);

        let get_button_request = Request::builder()
            .method("POST")
            .uri("/authorityBtn/getAuthorityBtn")
            .header(CONTENT_TYPE, "application/json")
            .header("x-token", token.clone())
            .body(Body::from(
                json!({
                    "authorityId": 9528,
                    "menuID": 22
                })
                .to_string(),
            ))
            .expect("request build failed");
        let get_button_response = app
            .clone()
            .oneshot(get_button_request)
            .await
            .expect("request failed");
        assert_eq!(get_button_response.status(), StatusCode::OK);
        let payload = response_json(get_button_response).await;
        assert_eq!(payload["data"]["selected"], json!([2201, 2202]));

        let menu_role_request = Request::builder()
            .method("GET")
            .uri("/menu/getMenuRoles?menuId=22")
            .header("x-token", token)
            .body(Body::empty())
            .expect("request build failed");
        let menu_role_response = app
            .oneshot(menu_role_request)
            .await
            .expect("request failed");
        assert_eq!(menu_role_response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn save_menu_updates_title() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let request = Request::builder()
            .method("POST")
            .uri("/menu/saveMenu")
            .header(CONTENT_TYPE, "application/json")
            .header("x-token", token)
            .body(Body::from(
                json!({
                    "ID": 23,
                    "parentId": 2,
                    "name": "menus",
                    "path": "/system/menus",
                    "component": "views/admin/MenusView.vue",
                    "sort": 23,
                    "hidden": false,
                    "meta": {
                        "title": "菜单维护",
                        "icon": "menu"
                    }
                })
                .to_string(),
            ))
            .expect("request build failed");

        let response = app.oneshot(request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
        let payload = response_json(response).await;
        assert_eq!(payload["data"]["meta"]["title"], "菜单维护");
    }

    #[tokio::test]
    async fn save_api_updates_description() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let request = Request::builder()
            .method("POST")
            .uri("/api/saveApi")
            .header(CONTENT_TYPE, "application/json")
            .header("x-token", token)
            .body(Body::from(
                json!({
                    "ID": 6,
                    "path": "/api/getApiList",
                    "apiGroup": "接口",
                    "description": "接口清单分页查询",
                    "method": "POST"
                })
                .to_string(),
            ))
            .expect("request build failed");

        let response = app.oneshot(request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
        let payload = response_json(response).await;
        assert_eq!(payload["data"]["description"], "接口清单分页查询");
    }

    #[tokio::test]
    async fn save_dictionary_updates_status() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let request = Request::builder()
            .method("POST")
            .uri("/sysDictionary/saveSysDictionary")
            .header(CONTENT_TYPE, "application/json")
            .header("x-token", token)
            .body(Body::from(
                json!({
                    "ID": 2,
                    "name": "违规类型",
                    "type": "violation_type",
                    "status": false,
                    "desc": "违规分级字典"
                })
                .to_string(),
            ))
            .expect("request build failed");

        let response = app.oneshot(request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
        let payload = response_json(response).await;
        assert_eq!(payload["data"]["status"], false);
    }

    #[tokio::test]
    async fn set_system_config_updates_redis_url() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let request = Request::builder()
            .method("POST")
            .uri("/system/setSystemConfig")
            .header(CONTENT_TYPE, "application/json")
            .header("x-token", token)
            .body(Body::from(
                json!({
                    "bindAddress": "0.0.0.0:8888",
                    "databaseUrl": "postgres://gaa:gaa@postgres:5432/gaa",
                    "redisUrl": "redis://redis:6379",
                    "multipointEnabled": true,
                    "compatibilityRefreshHeaders": true
                })
                .to_string(),
            ))
            .expect("request build failed");

        let response = app.oneshot(request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
        let payload = response_json(response).await;
        assert_eq!(payload["data"]["redisUrl"], "redis://redis:6379");
        assert_eq!(
            payload["data"]["databaseUrl"],
            "postgres://gaa:****@postgres:5432/gaa"
        );
    }

    #[tokio::test]
    async fn issue_api_token_appends_registry_item() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let request = Request::builder()
            .method("POST")
            .uri("/tool/api-token/issue")
            .header(CONTENT_TYPE, "application/json")
            .header("x-token", token)
            .body(Body::from(
                json!({
                    "name": "crawler-guard",
                    "scope": "user.violation.handle",
                    "ttl": "7d"
                })
                .to_string(),
            ))
            .expect("request build failed");
        let response = app.oneshot(request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
        let payload = response_json(response).await;
        assert_eq!(payload["data"]["name"], "crawler-guard");
    }

    #[tokio::test]
    async fn save_package_updates_output_path() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let request = Request::builder()
            .method("POST")
            .uri("/tool/package/save")
            .header(CONTENT_TYPE, "application/json")
            .header("x-token", token)
            .body(Body::from(
                json!({
                    "name": "feature-bundle",
                    "kind": "package",
                    "output": "/modules/parity-bundle",
                    "summary": "全页面恢复与 Tauri 2 后续重构"
                })
                .to_string(),
            ))
            .expect("request build failed");
        let response = app.oneshot(request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
        let payload = response_json(response).await;
        assert_eq!(payload["data"]["output"], "/modules/parity-bundle");
    }

    #[tokio::test]
    async fn save_plugin_manifest_updates_saved_at() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let request = Request::builder()
            .method("POST")
            .uri("/tool/plugin-manifest/save")
            .header(CONTENT_TYPE, "application/json")
            .header("x-token", token)
            .body(Body::from(
                json!({
                    "pluginName": "ops-toolkit",
                    "menuGroup": "风控工具",
                    "menuIds": [29, 307],
                    "apiIds": [6],
                    "dictionaryIds": [1, 2]
                })
                .to_string(),
            ))
            .expect("request build failed");
        let response = app.oneshot(request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
        let payload = response_json(response).await;
        assert_eq!(payload["data"]["menuGroup"], "风控工具");
        assert!(payload["data"]["savedAt"].as_i64().unwrap_or_default() > 0);
    }

    #[tokio::test]
    async fn plugin_install_save_records_target() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let request = Request::builder()
            .method("POST")
            .uri("/tool/plugin-install/save")
            .header(CONTENT_TYPE, "application/json")
            .header("x-token", token)
            .body(Body::from(
                json!({
                    "fileName": "ops-toolkit.zip",
                    "kind": "full",
                    "target": "docker",
                    "manifest": "菜单、接口、字典"
                })
                .to_string(),
            ))
            .expect("request build failed");
        let response = app.oneshot(request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
        let payload = response_json(response).await;
        assert_eq!(payload["data"]["target"], "docker");
    }

    #[tokio::test]
    async fn auto_code_registry_save_persists_entity_name() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let request = Request::builder()
            .method("POST")
            .uri("/tool/auto-code/save")
            .header(CONTENT_TYPE, "application/json")
            .header("x-token", token)
            .body(Body::from(
                json!({
                    "payload": {
                        "entity": "RiskPolicy",
                        "table": "risk_policies",
                        "module": "compliance",
                        "outputs": ["API", "FORM"]
                    }
                })
                .to_string(),
            ))
            .expect("request build failed");
        let response = app.oneshot(request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
        let payload = response_json(response).await;
        assert_eq!(payload["data"]["payload"]["entity"], "RiskPolicy");
    }

    #[tokio::test]
    async fn release_save_appends_note() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let request = Request::builder()
            .method("POST")
            .uri("/tool/release/save")
            .header(CONTENT_TYPE, "application/json")
            .header("x-token", token)
            .body(Body::from(
                json!({
                    "note": "完成自动代码与插件安装台账后端化"
                })
                .to_string(),
            ))
            .expect("request build failed");
        let response = app.oneshot(request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
        let payload = response_json(response).await;
        assert_eq!(payload["data"]["note"], "完成自动代码与插件安装台账后端化");
    }

    #[tokio::test]
    async fn upload_queue_complete_marks_items_done() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let seed_request = Request::builder()
            .method("POST")
            .uri("/example/upload/list")
            .header(CONTENT_TYPE, "application/json")
            .header("x-token", token.clone())
            .body(Body::from(
                json!({
                    "files": [
                        { "name": "report.xlsx", "size": "128 KB" }
                    ]
                })
                .to_string(),
            ))
            .expect("request build failed");
        let _ = app
            .clone()
            .oneshot(seed_request)
            .await
            .expect("request failed");

        let complete_request = Request::builder()
            .method("POST")
            .uri("/example/upload/complete")
            .header(CONTENT_TYPE, "application/json")
            .header("x-token", token)
            .body(Body::from("{}"))
            .expect("request build failed");
        let response = app.oneshot(complete_request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
        let payload = response_json(response).await;
        assert_eq!(payload["data"]["List"][0]["status"], "上传完成");
    }

    #[tokio::test]
    async fn resume_upload_recover_updates_first_row() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let request = Request::builder()
            .method("POST")
            .uri("/example/resume/recover")
            .header(CONTENT_TYPE, "application/json")
            .header("x-token", token)
            .body(Body::from("{}"))
            .expect("request build failed");
        let response = app.oneshot(request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
        let payload = response_json(response).await;
        assert_eq!(payload["data"]["List"][0]["status"], "已恢复续传");
    }

    #[tokio::test]
    async fn scan_session_update_persists_status() {
        let state = AppState::seed().await.expect("seed failed");
        let app = build_router(state);
        let (token, _) = login_tokens(&app).await;

        let get_request = Request::builder()
            .method("GET")
            .uri("/example/scan/session")
            .header("x-token", token.clone())
            .body(Body::empty())
            .expect("request build failed");
        let get_response = app
            .clone()
            .oneshot(get_request)
            .await
            .expect("request failed");
        let get_payload = response_json(get_response).await;
        let session_id = get_payload["data"]["sessionId"]
            .as_str()
            .expect("session id missing");

        let update_request = Request::builder()
            .method("POST")
            .uri("/example/scan/session")
            .header(CONTENT_TYPE, "application/json")
            .header("x-token", token)
            .body(Body::from(
                json!({
                    "sessionId": session_id,
                    "status": "上传完成"
                })
                .to_string(),
            ))
            .expect("request build failed");
        let update_response = app.oneshot(update_request).await.expect("request failed");
        assert_eq!(update_response.status(), StatusCode::OK);
        let update_payload = response_json(update_response).await;
        assert_eq!(update_payload["data"]["status"], "上传完成");
    }
}
