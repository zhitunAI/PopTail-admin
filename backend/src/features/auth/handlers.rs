pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Response, (StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let _ = body.captcha;
    let _ = body.captcha_id;
    match state.login(&body.username, &body.password).await {
        Ok((user, token, expires_at, refresh_token)) => {
            let payload = ok(
                LoginPayload {
                    user,
                    token: token.clone(),
                    expires_at,
                    refresh_token: refresh_token.clone(),
                },
                "登录成功",
            );
            let mut response = Json(payload).into_response();
            append_access_cookie(
                response.headers_mut(),
                &token,
                state.config.auth.access_ttl_sec,
            );
            append_refresh_cookie(
                response.headers_mut(),
                &refresh_token,
                state.config.auth.refresh_ttl_sec,
            );
            Ok(response)
        }
        Err(err) => Err((
            StatusCode::UNAUTHORIZED,
            Json(fail(serde_json::json!({}), &err)),
        )),
    }
}

pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<RefreshRequest>,
) -> Result<Response, (StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let refresh_token = if body.refresh_token.trim().is_empty() {
        extract_cookie_value(
            headers.get("cookie").and_then(|v| v.to_str().ok()),
            REFRESH_TOKEN_COOKIE,
        )
        .unwrap_or_default()
    } else {
        body.refresh_token
    };

    if refresh_token.trim().is_empty() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(fail(serde_json::json!({}), "refresh token缺失")),
        ));
    }

    match state.refresh(&refresh_token).await {
        Ok((token, exp)) => {
            let payload = ok(
                serde_json::json!({
                    "token": token,
                    "expiresAt": exp * 1000
                }),
                "刷新成功",
            );
            let mut response = Json(payload).into_response();
            append_access_cookie(
                response.headers_mut(),
                &token,
                state.config.auth.access_ttl_sec,
            );
            Ok(response)
        }
        Err(err) => Err((
            StatusCode::UNAUTHORIZED,
            Json(fail(serde_json::json!({}), &err)),
        )),
    }
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, (StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    let token = resolve_access_token(&headers);

    if let Some(token) = token {
        let auth = state.authenticate_human(&token).await.map_err(|err| {
            (
                StatusCode::UNAUTHORIZED,
                Json(fail(serde_json::json!({}), &err)),
            )
        })?;

        state.revoke_session(&auth.session_id).await;
    }
    let mut response = Json(ok(serde_json::json!({}), "登出成功")).into_response();
    append_clear_auth_cookies(response.headers_mut());
    Ok(response)
}

pub async fn user_info(
    State(state): State<AppState>,
    headers: HeaderMap,
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

    let payload = ok(
        UserInfoPayload {
            user_info: auth.user,
        },
        "获取成功",
    );

    let mut response = Json(payload).into_response();
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

