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
async fn update_profile_without_password_keeps_existing_login_password() {
    let state = AppState::seed().await.expect("seed failed");
    let app = build_router(state);
    let (token, _) = login_tokens(&app).await;

    let update_request = Request::builder()
        .method("POST")
        .uri("/user/updateProfile")
        .header(CONTENT_TYPE, "application/json")
        .header("x-token", token.clone())
        .body(Body::from(
            json!({
                "nickName": "资料已更新",
                "phone": "13800138001",
                "email": "admin-updated@gaa.local"
            })
            .to_string(),
        ))
        .expect("request build failed");
    let update_response = app
        .clone()
        .oneshot(update_request)
        .await
        .expect("request failed");
    assert_eq!(update_response.status(), StatusCode::OK);

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
    let login_response = app.oneshot(login_request).await.expect("request failed");
    assert_eq!(login_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn update_profile_with_password_changes_login_password() {
    let state = AppState::seed().await.expect("seed failed");
    let app = build_router(state);
    let (token, _) = login_tokens(&app).await;

    let update_request = Request::builder()
        .method("POST")
        .uri("/user/updateProfile")
        .header(CONTENT_TYPE, "application/json")
        .header("x-token", token.clone())
        .body(Body::from(
            json!({
                "nickName": "资料已更新",
                "phone": "13800138001",
                "email": "admin-updated@gaa.local",
                "password": "654321"
            })
            .to_string(),
        ))
        .expect("request build failed");
    let update_response = app
        .clone()
        .oneshot(update_request)
        .await
        .expect("request failed");
    assert_eq!(update_response.status(), StatusCode::OK);

    let old_login_request = Request::builder()
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
    let old_login_response = app
        .clone()
        .oneshot(old_login_request)
        .await
        .expect("request failed");
    assert_eq!(old_login_response.status(), StatusCode::UNAUTHORIZED);

    let stale_user_request = Request::builder()
        .method("GET")
        .uri("/user/getUserInfo")
        .header("x-token", token.clone())
        .body(Body::empty())
        .expect("request build failed");
    let stale_user_response = app
        .clone()
        .oneshot(stale_user_request)
        .await
        .expect("request failed");
    assert_eq!(stale_user_response.status(), StatusCode::UNAUTHORIZED);

    let new_login_request = Request::builder()
        .method("POST")
        .uri("/base/login")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "username": "admin",
                "password": "654321"
            })
            .to_string(),
        ))
        .expect("request build failed");
    let new_login_response = app
        .oneshot(new_login_request)
        .await
        .expect("request failed");
    assert_eq!(new_login_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn logout_revokes_current_session() {
    let state = AppState::seed().await.expect("seed failed");
    let app = build_router(state);
    let (token, refresh_token) = login_tokens(&app).await;

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
    let user_response = app
        .clone()
        .oneshot(user_request)
        .await
        .expect("request failed");
    assert_eq!(user_response.status(), StatusCode::UNAUTHORIZED);

    let refresh_request = Request::builder()
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
    let refresh_response = app.oneshot(refresh_request).await.expect("request failed");
    assert_eq!(refresh_response.status(), StatusCode::UNAUTHORIZED);
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
    assert!(payload["data"]["refreshToken"].as_str().is_some());
}

#[tokio::test]
async fn refresh_after_switch_authority_keeps_switched_role() {
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
    let switch_payload = response_json(switch_response).await;
    let switched_refresh_token = switch_payload["data"]["refreshToken"]
        .as_str()
        .expect("switched refresh token missing")
        .to_string();

    let refresh_request = Request::builder()
        .method("POST")
        .uri("/base/refresh")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "refreshToken": switched_refresh_token
            })
            .to_string(),
        ))
        .expect("request build failed");
    let refresh_response = app
        .clone()
        .oneshot(refresh_request)
        .await
        .expect("request failed");
    assert_eq!(refresh_response.status(), StatusCode::OK);
    let refresh_payload = response_json(refresh_response).await;
    let refreshed_access_token = refresh_payload["data"]["token"]
        .as_str()
        .expect("refreshed access token missing");

    let user_request = Request::builder()
        .method("GET")
        .uri("/user/getUserInfo")
        .header("x-token", refreshed_access_token)
        .body(Body::empty())
        .expect("request build failed");
    let user_response = app.oneshot(user_request).await.expect("request failed");
    assert_eq!(user_response.status(), StatusCode::OK);
    let user_payload = response_json(user_response).await;
    assert_eq!(user_payload["data"]["userInfo"]["authorityId"], json!(9528));
}

#[tokio::test]
async fn switched_role_can_still_logout() {
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
    let switch_payload = response_json(switch_response).await;
    let switched_access_token = switch_payload["data"]["token"]
        .as_str()
        .expect("switched access token missing")
        .to_string();
    let switched_refresh_token = switch_payload["data"]["refreshToken"]
        .as_str()
        .expect("switched refresh token missing")
        .to_string();

    let logout_request = Request::builder()
        .method("POST")
        .uri("/base/logout")
        .header("x-token", switched_access_token)
        .body(Body::empty())
        .expect("request build failed");
    let logout_response = app
        .clone()
        .oneshot(logout_request)
        .await
        .expect("request failed");
    assert_eq!(logout_response.status(), StatusCode::OK);

    let refresh_request = Request::builder()
        .method("POST")
        .uri("/base/refresh")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "refreshToken": switched_refresh_token
            })
            .to_string(),
        ))
        .expect("request build failed");
    let refresh_response = app.oneshot(refresh_request).await.expect("request failed");
    assert_eq!(refresh_response.status(), StatusCode::UNAUTHORIZED);
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
async fn single_point_login_invalidates_previous_session_when_multipoint_disabled() {
    let state = AppState::seed_with_config(crate::state::AppConfig {
        multipoint_enabled: false,
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
async fn multipoint_login_keeps_previous_session_when_enabled() {
    let state = AppState::seed_with_config(crate::state::AppConfig {
        multipoint_enabled: true,
        ..crate::state::AppConfig::default()
    })
    .await
    .expect("seed failed");
    let app = build_router(state);

    let (first_token, _) = login_tokens(&app).await;
    let (second_token, _) = login_tokens(&app).await;

    for token in [first_token, second_token] {
        let request = Request::builder()
            .method("GET")
            .uri("/user/getUserInfo")
            .header("x-token", token)
            .body(Body::empty())
            .expect("request build failed");
        let response = app.clone().oneshot(request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
    }
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
async fn get_authority_buttons_batch_returns_multiple_menu_results() {
    let state = AppState::seed().await.expect("seed failed");
    let app = build_router(state);
    let (token, _) = login_tokens(&app).await;

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

    let batch_request = Request::builder()
        .method("POST")
        .uri("/authorityBtn/getAuthorityBtns")
        .header(CONTENT_TYPE, "application/json")
        .header("x-token", token)
        .body(Body::from(
            json!({
                "authorityId": 9528,
                "menuIDs": [21, 22]
            })
            .to_string(),
        ))
        .expect("request build failed");
    let batch_response = app.oneshot(batch_request).await.expect("request failed");
    assert_eq!(batch_response.status(), StatusCode::OK);
    let payload = response_json(batch_response).await;
    let items = payload["data"].as_array().expect("data should be an array");
    assert_eq!(items.len(), 2);

    let menu_21 = items
        .iter()
        .find(|item| item["menuID"] == 21)
        .expect("menu 21 should exist");
    let menu_22 = items
        .iter()
        .find(|item| item["menuID"] == 22)
        .expect("menu 22 should exist");

    assert!(menu_21["selected"].is_array());
    assert_eq!(menu_22["selected"], json!([2201, 2202]));
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
async fn save_menus_updates_multiple_menu_sorts_in_one_request() {
    let state = AppState::seed().await.expect("seed failed");
    let app = build_router(state);
    let (token, _) = login_tokens(&app).await;

    let request = Request::builder()
        .method("POST")
        .uri("/menu/saveMenus")
        .header(CONTENT_TYPE, "application/json")
        .header("x-token", token.clone())
        .body(Body::from(
            json!({
                "menus": [
                    {
                        "ID": 1,
                        "parentId": 0,
                        "name": "dashboard",
                        "path": "/dashboard",
                        "component": "views/DashboardView.vue",
                        "sort": 20,
                        "hidden": false,
                        "meta": {
                            "title": "仪表盘1",
                            "icon": "layout-dashboard"
                        }
                    },
                    {
                        "ID": 3,
                        "parentId": 0,
                        "name": "profile",
                        "path": "/profile",
                        "component": "views/ProfileView.vue",
                        "sort": 10,
                        "hidden": false,
                        "meta": {
                            "title": "个人中心",
                            "icon": "user-circle"
                        }
                    }
                ]
            })
            .to_string(),
        ))
        .expect("request build failed");

    let response = app.clone().oneshot(request).await.expect("request failed");
    assert_eq!(response.status(), StatusCode::OK);
    let payload = response_json(response).await;
    let saved_items = payload["data"].as_array().expect("data should be an array");
    assert_eq!(saved_items.len(), 2);

    let list_request = Request::builder()
        .method("POST")
        .uri("/menu/getMenuList")
        .header(CONTENT_TYPE, "application/json")
        .header("x-token", token)
        .body(Body::from("{}"))
        .expect("request build failed");
    let list_response = app.oneshot(list_request).await.expect("request failed");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_payload = response_json(list_response).await;
    let items = list_payload["data"]["List"]
        .as_array()
        .expect("menu list should be an array");

    let dashboard = items
        .iter()
        .find(|item| item["ID"] == 1)
        .expect("dashboard menu should exist");
    let profile = items
        .iter()
        .find(|item| item["ID"] == 3)
        .expect("profile menu should exist");

    assert_eq!(dashboard["sort"], 20);
    assert_eq!(profile["sort"], 10);
}

#[tokio::test]
async fn delete_menu_removes_menu_and_descendants() {
    let state = AppState::seed().await.expect("seed failed");
    let app = build_router(state);
    let (token, _) = login_tokens(&app).await;

    let delete_request = Request::builder()
        .method("POST")
        .uri("/menu/deleteBaseMenu")
        .header(CONTENT_TYPE, "application/json")
        .header("x-token", token.clone())
        .body(Body::from(json!({ "ID": 29 }).to_string()))
        .expect("request build failed");
    let delete_response = app
        .clone()
        .oneshot(delete_request)
        .await
        .expect("request failed");
    assert_eq!(delete_response.status(), StatusCode::OK);

    let list_request = Request::builder()
        .method("POST")
        .uri("/menu/getMenuList")
        .header(CONTENT_TYPE, "application/json")
        .header("x-token", token)
        .body(Body::from("{}"))
        .expect("request build failed");
    let list_response = app.oneshot(list_request).await.expect("request failed");
    assert_eq!(list_response.status(), StatusCode::OK);
    let payload = response_json(list_response).await;
    let items = payload["data"]["List"]
        .as_array()
        .expect("menu list should be an array");
    assert!(
        items.iter().all(|item| item["ID"] != 29 && item["ID"] != 291 && item["ID"] != 295),
        "deleted menu and children should be removed"
    );
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
                "databaseUrl": "postgres://pop_tail:gaa@postgres:5432/gaa",
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
        "postgres://pop_tail:****@postgres:5432/gaa"
    );
}

#[tokio::test]
async fn enabling_multipoint_via_system_config_keeps_existing_sessions_alive() {
    let state = AppState::seed().await.expect("seed failed");
    let app = build_router(state);
    let (admin_token, _) = login_tokens(&app).await;

    let config_request = Request::builder()
        .method("POST")
        .uri("/system/setSystemConfig")
        .header(CONTENT_TYPE, "application/json")
        .header("x-token", admin_token.clone())
        .body(Body::from(
            json!({
                "bindAddress": "0.0.0.0:8888",
                "databaseUrl": "postgres://pop_tail:gaa@postgres:5432/gaa",
                "redisUrl": "redis://redis:6379",
                "multipointEnabled": true,
                "compatibilityRefreshHeaders": true
            })
            .to_string(),
        ))
        .expect("request build failed");
    let config_response = app
        .clone()
        .oneshot(config_request)
        .await
        .expect("request failed");
    assert_eq!(config_response.status(), StatusCode::OK);

    let (first_token, _) = login_tokens(&app).await;
    let (second_token, _) = login_tokens(&app).await;

    for token in [first_token, second_token] {
        let request = Request::builder()
            .method("GET")
            .uri("/user/getUserInfo")
            .header("x-token", token)
            .body(Body::empty())
            .expect("request build failed");
        let response = app.clone().oneshot(request).await.expect("request failed");
        assert_eq!(response.status(), StatusCode::OK);
    }
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
