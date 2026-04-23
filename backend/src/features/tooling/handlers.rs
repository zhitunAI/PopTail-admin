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

