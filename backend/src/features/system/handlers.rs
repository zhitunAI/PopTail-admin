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

