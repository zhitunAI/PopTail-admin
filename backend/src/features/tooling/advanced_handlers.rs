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

