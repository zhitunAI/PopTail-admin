use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::http::{Request, StatusCode};
use gaa_auth::{build_router, state::AppState};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn response_json(response: axum::response::Response) -> Value {
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read body failed");
    serde_json::from_slice(&body_bytes).expect("json parse failed")
}

async fn login_token(app: &axum::Router) -> String {
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
    payload["data"]["token"]
        .as_str()
        .expect("login token missing")
        .to_string()
}

#[tokio::test]
async fn switch_authority_rotates_token_and_refreshes_policy_package() {
    let state = AppState::seed().await.expect("seed failed");
    let app = build_router(state);
    let token = login_token(&app).await;

    let switch_request = Request::builder()
        .method("POST")
        .uri("/user/setUserAuthority")
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
    let rotated_header = switch_response
        .headers()
        .get("new-token")
        .and_then(|value| value.to_str().ok())
        .expect("new-token header missing")
        .to_string();
    assert!(
        switch_response.headers().get("new-expires-at").is_some(),
        "new-expires-at header missing"
    );
    let switch_payload = response_json(switch_response).await;
    assert_eq!(switch_payload["data"]["user"]["authorityId"], 9528);
    assert_eq!(switch_payload["data"]["token"], rotated_header);

    let user_request = Request::builder()
        .method("GET")
        .uri("/user/getUserInfo")
        .header("x-token", rotated_header.clone())
        .body(Body::empty())
        .expect("request build failed");
    let user_response = app
        .clone()
        .oneshot(user_request)
        .await
        .expect("request failed");
    assert_eq!(user_response.status(), StatusCode::OK);
    let user_payload = response_json(user_response).await;
    assert_eq!(user_payload["data"]["userInfo"]["authorityId"], 9528);

    let policy_request = Request::builder()
        .method("POST")
        .uri("/casbin/getPolicyPathByAuthorityId")
        .header("x-token", rotated_header)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "authorityId": 9528
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
    assert!(
        items
            .iter()
            .any(|item| { item["path"] == "/ai/moderation/decision" && item["method"] == "POST" }),
        "AI authority policy should be returned after switch"
    );
    assert!(
        items.iter().all(|item| item["path"] != "/base/logout"),
        "admin-only logout policy should not remain after switch"
    );
}
