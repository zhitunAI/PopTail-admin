pub async fn upload_queue_replace(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<UploadQueueRequest>,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::UploadFileRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.replace_upload_queue(body).await, "保存成功")))
}

pub async fn upload_file_asset(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<
    Json<ApiResponse<UploadedFilePayload>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;

    let mut file_name = format!("upload-{}.bin", now_ts());
    let mut class_id = 0_u64;

    while let Some(field) = multipart.next_field().await.map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            Json(fail(
                serde_json::json!({}),
                &format!("读取上传字段失败: {err}"),
            )),
        )
    })? {
        let name = field.name().unwrap_or_default().to_string();
        if name == "classId" {
            let value = field.text().await.unwrap_or_default();
            class_id = value.parse::<u64>().unwrap_or(0);
            continue;
        }
        if name == "file" {
            if let Some(candidate) = field.file_name() {
                file_name = candidate.to_string();
            }
            let _ = field.bytes().await.map_err(|err| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(fail(
                        serde_json::json!({}),
                        &format!("读取上传文件失败: {err}"),
                    )),
                )
            })?;
        }
    }

    let key = format!("uploads/{}-{}", now_ts(), sanitize_upload_name(&file_name));
    let file = UploadedFileAsset {
        id: now_ts() as u64,
        class_id,
        key: key.clone(),
        name: file_name,
        url: format!("/{}", key),
    };

    Ok(Json(ok(UploadedFilePayload { file }, "上传成功")))
}

pub async fn upload_queue_complete(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::UploadFileRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.complete_upload_queue().await, "上传成功")))
}

pub async fn resume_upload_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::ResumeUploadRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.list_resume_uploads().await, "获取成功")))
}

pub async fn resume_upload_advance(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::ResumeUploadRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.advance_resume_uploads().await, "推进成功")))
}

pub async fn resume_upload_resume(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::PageResult<crate::models::ResumeUploadRecord>>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        state.resume_interrupted_upload().await,
        "恢复成功",
    )))
}

pub async fn scan_session_get(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<
    Json<ApiResponse<crate::models::ScanSessionRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(
        state.get_or_create_scan_session().await,
        "获取成功",
    )))
}

pub async fn scan_session_update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ScanSessionUpdateRequest>,
) -> Result<
    Json<ApiResponse<crate::models::ScanSessionRecord>>,
    (StatusCode, Json<ApiResponse<serde_json::Value>>),
> {
    require_auth(&state, &headers).await?;
    Ok(Json(ok(state.update_scan_session(body).await, "保存成功")))
}

