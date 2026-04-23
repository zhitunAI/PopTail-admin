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
async fn customer_crud_round_trip_matches_expected_contract() {
    let state = AppState::seed().await.expect("seed failed");
    let app = build_router(state);
    let token = login_token(&app).await;

    let create_request = Request::builder()
        .method("POST")
        .uri("/customer/customer")
        .header("x-token", token.clone())
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "customerName": "极光供应链",
                "customerPhoneData": "400-100-2000",
                "customerLevel": "S",
                "customerStatus": "重点跟进",
                "remark": "需要联调发货看板"
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
    let create_payload = response_json(create_response).await;
    let customer_id = create_payload["data"]["customer"]["ID"]
        .as_u64()
        .expect("created customer id missing");
    assert_eq!(
        create_payload["data"]["customer"]["customerName"],
        "极光供应链"
    );
    assert_eq!(create_payload["data"]["customer"]["customerLevel"], "S");

    let list_request = Request::builder()
        .method("GET")
        .uri("/customer/customerList?page=1&pageSize=10&keyword=%E6%9E%81%E5%85%89")
        .header("x-token", token.clone())
        .body(Body::empty())
        .expect("request build failed");
    let list_response = app
        .clone()
        .oneshot(list_request)
        .await
        .expect("request failed");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_payload = response_json(list_response).await;
    let rows = list_payload["data"]["List"]
        .as_array()
        .expect("customer list missing");
    assert!(
        rows.iter()
            .any(|item| item["ID"].as_u64() == Some(customer_id)
                && item["customerName"] == "极光供应链"),
        "created customer should be returned by keyword filter"
    );

    let update_request = Request::builder()
        .method("PUT")
        .uri("/customer/customer")
        .header("x-token", token.clone())
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "ID": customer_id,
                "customerName": "极光供应链",
                "customerPhoneData": "400-100-2001",
                "customerLevel": "A+",
                "customerStatus": "已签约",
                "remark": "合同已完成"
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
    let update_payload = response_json(update_response).await;
    assert_eq!(
        update_payload["data"]["customer"]["customerPhoneData"],
        "400-100-2001"
    );
    assert_eq!(
        update_payload["data"]["customer"]["customerStatus"],
        "已签约"
    );

    let get_request = Request::builder()
        .method("GET")
        .uri(format!("/customer/customer?ID={customer_id}"))
        .header("x-token", token.clone())
        .body(Body::empty())
        .expect("request build failed");
    let get_response = app
        .clone()
        .oneshot(get_request)
        .await
        .expect("request failed");
    assert_eq!(get_response.status(), StatusCode::OK);
    let get_payload = response_json(get_response).await;
    assert_eq!(get_payload["data"]["customer"]["remark"], "合同已完成");

    let delete_request = Request::builder()
        .method("DELETE")
        .uri("/customer/customer")
        .header("x-token", token.clone())
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "ID": customer_id }).to_string()))
        .expect("request build failed");
    let delete_response = app
        .clone()
        .oneshot(delete_request)
        .await
        .expect("request failed");
    assert_eq!(delete_response.status(), StatusCode::OK);

    let missing_request = Request::builder()
        .method("GET")
        .uri(format!("/customer/customer?ID={customer_id}"))
        .header("x-token", token)
        .body(Body::empty())
        .expect("request build failed");
    let missing_response = app
        .clone()
        .oneshot(missing_request)
        .await
        .expect("request failed");
    assert_eq!(missing_response.status(), StatusCode::NOT_FOUND);
}
