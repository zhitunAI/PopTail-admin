use axum::body::{Body, to_bytes};
use axum::http::header::CONTENT_TYPE;
use axum::http::{Request, StatusCode};
use pop_tail_auth::{build_router, state::AppState};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn response_json(response: axum::response::Response) -> Value {
    let body_bytes = to_bytes(response.into_body(), usize::MAX)
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
        .expect("token missing")
        .to_string()
}

#[tokio::test]
async fn upload_queue_supports_replace_append_and_inspect() {
    let state = AppState::seed().await.expect("seed failed");
    let app = build_router(state);
    let token = login_token(&app).await;

    let replace_request = Request::builder()
        .method("POST")
        .uri("/example/upload/list")
        .header("x-token", token.clone())
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "mode": "replace",
                "files": [
                    { "name": "alpha.png", "size": "512 KB" }
                ]
            })
            .to_string(),
        ))
        .expect("request build failed");
    let replace_response = app
        .clone()
        .oneshot(replace_request)
        .await
        .expect("request failed");
    assert_eq!(replace_response.status(), StatusCode::OK);

    let append_request = Request::builder()
        .method("POST")
        .uri("/example/upload/list")
        .header("x-token", token.clone())
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "mode": "append",
                "files": [
                    { "name": "beta.png", "size": "1 MB" }
                ]
            })
            .to_string(),
        ))
        .expect("request build failed");
    let append_response = app
        .clone()
        .oneshot(append_request)
        .await
        .expect("request failed");
    assert_eq!(append_response.status(), StatusCode::OK);
    let append_payload = response_json(append_response).await;
    assert_eq!(
        append_payload["data"]["Total"]
            .as_u64()
            .expect("total missing"),
        2
    );

    let inspect_request = Request::builder()
        .method("POST")
        .uri("/example/upload/list")
        .header("x-token", token)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "mode": "inspect",
                "files": []
            })
            .to_string(),
        ))
        .expect("request build failed");
    let inspect_response = app.oneshot(inspect_request).await.expect("request failed");
    assert_eq!(inspect_response.status(), StatusCode::OK);
    let inspect_payload = response_json(inspect_response).await;
    let rows = inspect_payload["data"]["List"]
        .as_array()
        .expect("list missing");
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().any(|item| item["name"] == "alpha.png"));
    assert!(rows.iter().any(|item| item["name"] == "beta.png"));
}
