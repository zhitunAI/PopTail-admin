pub async fn update_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ProfileUpdateRequest>,
) -> Result<Json<ApiResponse<UserInfoPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
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

    let user = state
        .update_profile(auth.user.id, body)
        .await
        .map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                Json(fail(serde_json::json!({}), &err)),
            )
        })?;

    Ok(Json(ok(UserInfoPayload { user_info: user }, "保存成功")))
}

pub async fn user_upsert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<UserUpsertRequest>,
) -> Result<Json<ApiResponse<UserInfoPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    require_auth(&state, &headers).await?;
    let user = state.upsert_user(body).await.map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            Json(fail(serde_json::json!({}), &err)),
        )
    })?;
    Ok(Json(ok(UserInfoPayload { user_info: user }, "保存成功")))
}

pub async fn switch_authority(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SwitchAuthorityRequest>,
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
        .is_allowed(auth.user.authority_id, "/user/setUserAuthority", "POST")
        .await
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(fail(serde_json::json!({}), "权限不足")),
        ));
    }

    let (user, new_token, expires_at) = state
        .switch_authority(&auth.session_id, auth.user.id, body.authority_id)
        .await
        .map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                Json(fail(serde_json::json!({}), &err)),
            )
        })?;

    let payload = ok(
        SwitchAuthorityPayload {
            user,
            token: new_token.clone(),
            expires_at,
        },
        "修改成功",
    );
    let mut response = Json(payload).into_response();
    if state.config.compatibility_refresh_headers {
        response.headers_mut().insert(
            "new-token",
            HeaderValue::from_str(&new_token).unwrap_or_else(|_| HeaderValue::from_static("")),
        );
        response.headers_mut().insert(
            "new-expires-at",
            HeaderValue::from_str(&(expires_at / 1000).to_string())
                .unwrap_or_else(|_| HeaderValue::from_static("0")),
        );
    }
    append_access_cookie(
        response.headers_mut(),
        &new_token,
        state.config.auth.access_ttl_sec,
    );
    Ok(response)
}

