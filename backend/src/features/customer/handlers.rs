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

