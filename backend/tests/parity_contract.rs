use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::http::{Request, StatusCode};
use pop_tail_auth::{build_router, state::AppState};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn response_json(response: axum::response::Response) -> Value {
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read body failed");
    serde_json::from_slice(&body_bytes).expect("json parse failed")
}

#[tokio::test]
async fn login_response_shape_matches_legacy_contract() {
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
    let payload = response_json(response).await;

    assert!(payload.get("code").is_some());
    assert!(payload.get("msg").is_some());
    let data = payload["data"].as_object().expect("data should be object");
    assert!(data.contains_key("user"));
    assert!(data.contains_key("token"));
    assert!(data.contains_key("expiresAt"));
    assert!(data.contains_key("refreshToken"));

    let user = data["user"].as_object().expect("user should be object");
    assert!(user.contains_key("ID"));
    assert!(user.contains_key("uuid"));
    assert!(user.contains_key("userName"));
    assert!(user.contains_key("nickName"));
    assert!(user.contains_key("authorityId"));
    assert!(user.contains_key("authority"));
    assert!(user.contains_key("authorities"));
    let authority = user["authority"]
        .as_object()
        .expect("authority should be object");
    assert!(authority.contains_key("authorityId"));
    assert!(authority.contains_key("authorityName"));
    assert!(authority.contains_key("defaultRouter"));
}

#[tokio::test]
async fn user_info_and_policy_shapes_match_legacy_contract() {
    let state = AppState::seed().await.expect("seed failed");
    let app = build_router(state);

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
    let login_payload = response_json(login_response).await;
    let token = login_payload["data"]["token"]
        .as_str()
        .expect("token missing")
        .to_string();

    let user_request = Request::builder()
        .method("GET")
        .uri("/user/getUserInfo")
        .header("x-token", token.clone())
        .body(Body::empty())
        .expect("request build failed");
    let user_response = app
        .clone()
        .oneshot(user_request)
        .await
        .expect("request failed");
    assert_eq!(user_response.status(), StatusCode::OK);
    let user_payload = response_json(user_response).await;
    assert!(user_payload.get("code").is_some());
    assert!(user_payload.get("msg").is_some());
    let user_info = user_payload["data"]["userInfo"]
        .as_object()
        .expect("userInfo should be object");
    assert!(user_info.contains_key("userName"));
    assert!(user_info.contains_key("authority"));

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
    let policy_response = app
        .clone()
        .oneshot(policy_request)
        .await
        .expect("request failed");
    assert_eq!(policy_response.status(), StatusCode::OK);
    let policy_payload = response_json(policy_response).await;
    let items = policy_payload["data"]
        .as_array()
        .expect("policy data should be array");
    assert!(!items.is_empty(), "policy list should not be empty");
    let first = items[0].as_object().expect("policy item should be object");
    assert!(first.contains_key("path"));
    assert!(first.contains_key("method"));
}
