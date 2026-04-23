pub async fn frontend_nav_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::FrontendNavRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_frontend_nav().await, "获取成功")))
}

pub async fn frontend_nav_save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<FrontendNavUpsertRequest>,
) -> Result<
    Json<ApiResponse<crate::models::FrontendNavRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.upsert_frontend_nav(body).await, "保存成功")))
}

pub async fn frontend_nav_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<IdRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    if state.delete_frontend_nav(body.id).await {
        Ok(Json(ok(serde_json::json!({}), "删除成功")))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(fail(serde_json::json!({}), "导航不存在")),
        ))
    }
}

pub async fn frontend_settings_get(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::FrontendSettingsRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.frontend_settings().await, "获取成功")))
}

pub async fn frontend_settings_save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<FrontendSettingsUpdateRequest>,
) -> Result<
    Json<ApiResponse<crate::models::FrontendSettingsRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        state.update_frontend_settings(body).await,
        "保存成功",
    )))
}

pub async fn public_frontend_settings_get(
    State(state): State<AppState>,
) -> Json<ApiResponse<crate::models::FrontendSettingsRecord>> {
    Json(ok(state.frontend_settings().await, "获取成功"))
}

pub async fn public_frontend_nav_list(
    State(state): State<AppState>,
) -> Json<ApiResponse<crate::models::PageResult<crate::models::FrontendNavRecord>>> {
    Json(ok(state.list_frontend_nav().await, "获取成功"))
}

pub async fn public_article_list(
    State(state): State<AppState>,
) -> Json<ApiResponse<crate::models::PageResult<crate::models::ArticleRecord>>> {
    Json(ok(state.list_articles().await, "获取成功"))
}

pub async fn article_category_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::ArticleCategoryRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_article_categories().await, "获取成功")))
}

pub async fn article_category_save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ArticleCategoryUpsertRequest>,
) -> Result<
    Json<ApiResponse<crate::models::ArticleCategoryRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        state.upsert_article_category(body).await,
        "保存成功",
    )))
}

pub async fn article_category_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<IdRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    if state.delete_article_category(body.id).await {
        Ok(Json(ok(serde_json::json!({}), "删除成功")))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(fail(serde_json::json!({}), "文章分类不存在")),
        ))
    }
}

pub async fn article_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::ArticleRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_articles().await, "获取成功")))
}

pub async fn article_save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ArticleUpsertRequest>,
) -> Result<
    Json<ApiResponse<crate::models::ArticleRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.upsert_article(body).await, "保存成功")))
}

pub async fn article_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<IdRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    if state.delete_article(body.id).await {
        Ok(Json(ok(serde_json::json!({}), "删除成功")))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(fail(serde_json::json!({}), "文章不存在")),
        ))
    }
}

pub async fn member_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::MemberRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_members().await, "获取成功")))
}

pub async fn member_save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<MemberUpsertRequest>,
) -> Result<
    Json<ApiResponse<crate::models::MemberRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.upsert_member(body).await, "保存成功")))
}

pub async fn member_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<IdRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    if state.delete_member(body.id).await {
        Ok(Json(ok(serde_json::json!({}), "删除成功")))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(fail(serde_json::json!({}), "会员不存在")),
        ))
    }
}

pub async fn console_menu_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::ConsoleMenuRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_console_menus().await, "获取成功")))
}

pub async fn console_menu_save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ConsoleMenuUpsertRequest>,
) -> Result<
    Json<ApiResponse<crate::models::ConsoleMenuRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.upsert_console_menu(body).await, "保存成功")))
}

pub async fn console_menu_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<IdRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    if state.delete_console_menu(body.id).await {
        Ok(Json(ok(serde_json::json!({}), "删除成功")))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(fail(serde_json::json!({}), "控制台菜单不存在")),
        ))
    }
}

