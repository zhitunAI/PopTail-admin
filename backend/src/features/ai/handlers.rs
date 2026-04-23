pub async fn post_ai_moderation_decision(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AiDecisionRequest>,
) -> Result<Json<ApiResponse<DecisionPayload>>, (StatusCode, Json<ApiResponse<serde_json::Value>>)>
{
    let (actor, effective_role) = match body.mode.as_str() {
        "system" => {
            let service_token = body.service_token.clone().ok_or((
                StatusCode::UNAUTHORIZED,
                Json(fail(serde_json::json!({}), "system模式缺少service token")),
            ))?;
            let service = state
                .resolve_service_identity(&service_token)
                .await
                .ok_or((
                    StatusCode::UNAUTHORIZED,
                    Json(fail(serde_json::json!({}), "service token无效")),
                ))?;
            let needed_scope = if body.target_is_user_or_crawler() {
                "user.violation.handle"
            } else {
                "article.review"
            };
            if !service.scopes.iter().any(|s| s == needed_scope) {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(fail(serde_json::json!({}), "service scope不足")),
                ));
            }
            (
                format!("service:{}", service.name),
                service.ai_role.to_string(),
            )
        }
        "delegated" => {
            let access_token = resolve_access_token(&headers).ok_or((
                StatusCode::UNAUTHORIZED,
                Json(fail(serde_json::json!({}), "未登录或非法访问，请登录")),
            ))?;

            let auth = state
                .authenticate_human(&access_token)
                .await
                .map_err(|err| {
                    (
                        StatusCode::UNAUTHORIZED,
                        Json(fail(serde_json::json!({}), &err)),
                    )
                })?;

            let delegation = body.delegation.clone().ok_or((
                StatusCode::BAD_REQUEST,
                Json(fail(serde_json::json!({}), "delegated模式缺少delegation")),
            ))?;

            if delegation.operator_user_id != auth.user.id {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(fail(serde_json::json!({}), "委托用户不匹配")),
                ));
            }
            if delegation.scope.is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(fail(serde_json::json!({}), "delegation scope为空")),
                ));
            }
            if !state
                .is_allowed(auth.user.authority_id, "/ai/moderation/decision", "POST")
                .await
            {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(fail(serde_json::json!({}), "权限不足")),
                ));
            }
            (
                format!("user:{}", auth.user.user_name),
                auth.user.authority_id.to_string(),
            )
        }
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(fail(serde_json::json!({}), "mode仅支持system/delegated")),
            ));
        }
    };

    let audit_id = Uuid::new_v4().to_string();
    state
        .append_audit(AuditEvent {
            audit_id: audit_id.clone(),
            actor,
            effective_role,
            target: body.article_id.clone(),
            action: body.action.clone(),
            reason: body.reason.clone(),
            workflow_run_id: Uuid::new_v4().to_string(),
            timestamp: now_ts(),
        })
        .await;

    Ok(Json(ok(
        DecisionPayload {
            accepted: true,
            audit_id,
        },
        "AI处置已记录",
    )))
}

trait DecisionTargetExt {
    fn target_is_user_or_crawler(&self) -> bool;
}

impl DecisionTargetExt for AiDecisionRequest {
    fn target_is_user_or_crawler(&self) -> bool {
        let lower = self.article_id.to_lowercase();
        lower.contains("user") || lower.contains("crawler")
    }
}
