pub async fn user_policy_paths(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AuthorityRequest>,
) -> Result<Response, (StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let token = resolve_access_token(&headers).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(fail(serde_json::json!({}), "未登录或非法访问，请登录")),
    ))?;

    let auth = state.authenticate_human(&token).await.map_err(|err| {
        (
            StatusCode::UNAUTHORIZED,
            Json(fail(serde_json::json!({}), &err)),
        )
    })?;

    if !state
        .is_allowed(
            auth.user.authority_id,
            "/casbin/getPolicyPathByAuthorityId",
            "POST",
        )
        .await
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(fail(serde_json::json!({}), "权限不足")),
        ));
    }

    let policies = state.get_policy_paths(body.authority_id).await;
    let mut response = Json(ok(policies, "获取成功")).into_response();
    if state.config.compatibility_refresh_headers {
        if let Some(token) = auth.new_token {
            response.headers_mut().insert(
                "new-token",
                HeaderValue::from_str(&token).unwrap_or_else(|_| HeaderValue::from_static("")),
            );
            append_access_cookie(
                response.headers_mut(),
                &token,
                state.config.auth.access_ttl_sec,
            );
        }
        if let Some(exp) = auth.new_expires_at {
            response.headers_mut().insert(
                "new-expires-at",
                HeaderValue::from_str(&exp.to_string())
                    .unwrap_or_else(|_| HeaderValue::from_static("0")),
            );
        }
    }
    Ok(response)
}

pub async fn user_policy_paths_set(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AuthorityPolicyUpdateRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    state
        .set_policy_paths(body.authority_id, body.policies)
        .await;
    Ok(Json(ok(serde_json::json!({}), "保存成功")))
}

pub async fn user_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<UserListRequest>,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::UserInfo>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_users(body).await, "获取成功")))
}

pub async fn authority_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<Vec<crate::models::AuthorityInfo>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_authorities().await, "获取成功")))
}

pub async fn authority_upsert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AuthorityUpsertRequest>,
) -> Result<
    Json<ApiResponse<crate::models::AuthorityInfo>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    let item = state.upsert_authority(body).await.map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            Json(fail(serde_json::json!({}), &err)),
        )
    })?;
    Ok(Json(ok(item, "保存成功")))
}

pub async fn authority_users(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AuthorityRequest>,
) -> Result<Json<ApiResponse<Vec<u64>>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        state.get_user_ids_by_authority(query.authority_id).await,
        "获取成功",
    )))
}

pub async fn authority_set_role_users(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AuthorityRoleUsersRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    state
        .set_role_users(body.authority_id, body.user_ids)
        .await
        .map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                Json(fail(serde_json::json!({}), &err)),
            )
        })?;
    Ok(Json(ok(serde_json::json!({}), "设置成功")))
}

pub async fn menu_tree(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<MenuTreePayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        MenuTreePayload {
            menus: state.list_menu_tree().await,
        },
        "获取成功",
    )))
}

pub async fn menu_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::MenuInfo>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    let menus = flatten_menus(&state.list_menu_tree().await);
    Ok(Json(ok(
        crate::models::PageResult {
            total: menus.len(),
            list: menus,
            page: 1,
            page_size: 999,
        },
        "获取成功",
    )))
}

pub async fn menu_upsert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<MenuUpsertRequest>,
) -> Result<
    Json<ApiResponse<crate::models::MenuInfo>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.upsert_menu(body).await, "保存成功")))
}

pub async fn menu_authority_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AuthorityRequest>,
) -> Result<
    Json<ApiResponse<MenuAuthorityPayload>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        MenuAuthorityPayload {
            menus: state.get_menu_authority(body.authority_id).await,
        },
        "获取成功",
    )))
}

pub async fn menu_authority_set(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<MenuAuthorityAssignmentRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    let menu_ids = body.menus.into_iter().map(|menu| menu.id).collect();
    state
        .set_menu_authority(body.authority_id, menu_ids)
        .await
        .map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                Json(fail(serde_json::json!({}), &err)),
            )
        })?;
    Ok(Json(ok(serde_json::json!({}), "添加成功")))
}

pub async fn menu_roles_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<crate::models::MenuIdRequest>,
) -> Result<Json<ApiResponse<MenuRoleIdsPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    let (authority_ids, default_router_authority_ids) = state.get_menu_roles(query.menu_id).await;
    Ok(Json(ok(
        MenuRoleIdsPayload {
            authority_ids,
            default_router_authority_ids,
        },
        "获取成功",
    )))
}

pub async fn menu_roles_set(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<MenuRoleUpdateRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    state
        .set_menu_roles(body.menu_id, body.authority_ids)
        .await
        .map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                Json(fail(serde_json::json!({}), &err)),
            )
        })?;
    Ok(Json(ok(serde_json::json!({}), "保存成功")))
}

pub async fn authority_btn_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AuthorityButtonMatrixRequest>,
) -> Result<
    Json<ApiResponse<AuthorityButtonMatrixSelection>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        AuthorityButtonMatrixSelection {
            selected: state
                .get_authority_buttons(body.authority_id, body.menu_id)
                .await,
        },
        "查询成功",
    )))
}

pub async fn authority_btn_set(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AuthorityButtonMatrixRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    state
        .set_authority_buttons(body.authority_id, body.menu_id, body.selected)
        .await
        .map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                Json(fail(serde_json::json!({}), &err)),
            )
        })?;
    Ok(Json(ok(serde_json::json!({}), "分配成功")))
}

