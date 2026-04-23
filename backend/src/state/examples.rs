use super::seed::*;
use super::*;

impl AppState {
    pub async fn replace_upload_queue(
        &self,
        input: crate::models::UploadQueueRequest,
    ) -> PageResult<UploadFileRecord> {
        let mut guard = self.inner.write().await;
        let mode = input.mode.unwrap_or_else(|| "replace".to_string());

        if mode == "inspect" {
            return page_result(guard.upload_files.clone(), Some(1), Some(50));
        }

        let incoming = input
            .files
            .into_iter()
            .map(|item| UploadFileRecord {
                id: 0,
                name: item.name,
                size: item.size,
                status: "待上传".to_string(),
            })
            .collect::<Vec<_>>();

        guard.upload_files = if mode == "append" {
            let mut merged = guard.upload_files.clone();
            for item in incoming {
                if let Some(existing) = merged
                    .iter_mut()
                    .find(|existing| existing.name == item.name && existing.size == item.size)
                {
                    if existing.status != "上传完成" {
                        existing.status = "待上传".to_string();
                    }
                } else {
                    merged.push(item);
                }
            }
            merged
        } else {
            incoming
        };

        for (index, item) in guard.upload_files.iter_mut().enumerate() {
            item.id = (index + 1) as u64;
        }

        page_result(guard.upload_files.clone(), Some(1), Some(50))
    }

    pub async fn complete_upload_queue(&self) -> PageResult<UploadFileRecord> {
        let mut guard = self.inner.write().await;
        for item in &mut guard.upload_files {
            item.status = "上传完成".to_string();
        }
        page_result(guard.upload_files.clone(), Some(1), Some(50))
    }

    pub async fn list_resume_uploads(&self) -> PageResult<ResumeUploadRecord> {
        let guard = self.inner.read().await;
        page_result(guard.resume_uploads.clone(), Some(1), Some(50))
    }

    pub async fn advance_resume_uploads(&self) -> PageResult<ResumeUploadRecord> {
        let mut guard = self.inner.write().await;
        for item in &mut guard.resume_uploads {
            if item.progress < 100 {
                item.progress = (item.progress + 12).min(100);
                item.status = if item.progress >= 100 {
                    "已完成".to_string()
                } else {
                    "上传中".to_string()
                };
            }
        }
        page_result(guard.resume_uploads.clone(), Some(1), Some(50))
    }

    pub async fn resume_interrupted_upload(&self) -> PageResult<ResumeUploadRecord> {
        let mut guard = self.inner.write().await;
        if let Some(item) = guard.resume_uploads.first_mut() {
            item.status = "已恢复续传".to_string();
        }
        page_result(guard.resume_uploads.clone(), Some(1), Some(50))
    }

    pub async fn get_or_create_scan_session(&self) -> ScanSessionRecord {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard.scan_sessions.first().cloned() {
            return existing;
        }
        let created = ScanSessionRecord {
            session_id: format!("scan-{}", now_ts()),
            status: "待扫码".to_string(),
        };
        guard.scan_sessions.push(created.clone());
        created
    }

    pub async fn update_scan_session(
        &self,
        input: crate::models::ScanSessionUpdateRequest,
    ) -> ScanSessionRecord {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard
            .scan_sessions
            .iter_mut()
            .find(|item| item.session_id == input.session_id)
        {
            existing.status = input.status;
            return existing.clone();
        }

        let created = ScanSessionRecord {
            session_id: input.session_id,
            status: input.status,
        };
        guard.scan_sessions.push(created.clone());
        created
    }

    pub async fn list_customers(
        &self,
        input: crate::models::CustomerListRequest,
    ) -> PageResult<CustomerRecord> {
        let guard = self.inner.read().await;
        let keyword = input.keyword.unwrap_or_default().trim().to_lowercase();
        let rows = guard
            .customers
            .iter()
            .filter(|item| {
                keyword.is_empty()
                    || item.customer_name.to_lowercase().contains(&keyword)
                    || item.customer_phone_data.contains(&keyword)
                    || item.customer_level.to_lowercase().contains(&keyword)
                    || item.customer_status.to_lowercase().contains(&keyword)
            })
            .cloned()
            .collect();
        page_result(rows, input.page, input.page_size)
    }

    pub async fn get_customer(&self, id: u64) -> Option<CustomerRecord> {
        let guard = self.inner.read().await;
        guard.customers.iter().find(|item| item.id == id).cloned()
    }

    pub async fn upsert_customer(
        &self,
        input: crate::models::CustomerUpsertRequest,
        actor_user_id: u64,
    ) -> CustomerRecord {
        let mut guard = self.inner.write().await;
        let now = now_ts() * 1000;
        if let Some(existing) = guard
            .customers
            .iter_mut()
            .find(|item| item.id == input.id.unwrap_or(0))
        {
            existing.customer_name = input.customer_name;
            existing.customer_phone_data = input.customer_phone_data;
            existing.customer_level = non_empty(input.customer_level, &existing.customer_level);
            existing.customer_status = non_empty(input.customer_status, &existing.customer_status);
            existing.remark = input.remark.unwrap_or_default();
            existing.updated_at = now;
            return existing.clone();
        }

        let next_id = guard
            .customers
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            + 1;
        let created = CustomerRecord {
            id: next_id,
            created_at: now,
            updated_at: now,
            customer_name: input.customer_name,
            customer_phone_data: input.customer_phone_data,
            sys_user_id: input.sys_user_id.unwrap_or(actor_user_id),
            customer_level: non_empty(input.customer_level, "A"),
            customer_status: non_empty(input.customer_status, "跟进中"),
            remark: input.remark.unwrap_or_default(),
        };
        guard.customers.insert(0, created.clone());
        created
    }

    pub async fn delete_customer(&self, id: u64) -> bool {
        let mut guard = self.inner.write().await;
        let before = guard.customers.len();
        guard.customers.retain(|item| item.id != id);
        before != guard.customers.len()
    }

    pub async fn save_mcp_tool(&self, input: crate::models::McpToolUpsertRequest) -> McpToolRecord {
        let mut guard = self.inner.write().await;
        let now = now_ts() * 1000;
        if let Some(existing) = guard
            .mcp_tools
            .iter_mut()
            .find(|item| item.name.eq_ignore_ascii_case(&input.name))
        {
            existing.description = input.description;
            existing.params = sanitize_mcp_params(input.params);
            existing.response = sanitize_mcp_outputs(input.response);
            existing.updated_at = now;
            return existing.clone();
        }

        let next_id = guard
            .mcp_tools
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            + 1;
        let created = McpToolRecord {
            id: next_id,
            created_at: now,
            updated_at: now,
            name: input.name.trim().to_string(),
            description: input.description.trim().to_string(),
            params: sanitize_mcp_params(input.params),
            response: sanitize_mcp_outputs(input.response),
        };
        guard.mcp_tools.insert(0, created.clone());
        created
    }

    pub async fn list_mcp_tools(&self) -> Vec<McpToolDescriptor> {
        let guard = self.inner.read().await;
        guard
            .mcp_tools
            .iter()
            .cloned()
            .map(build_mcp_descriptor)
            .collect()
    }

    pub async fn mcp_service_status(&self) -> McpServiceStatusPayload {
        let guard = self.inner.read().await;
        let state = if guard.mcp_service_running {
            "running"
        } else {
            "stopped"
        };
        McpServiceStatusPayload {
            managed: true,
            state: state.to_string(),
            reachable: guard.mcp_service_running,
            base_url: "http://127.0.0.1:8889/mcp".to_string(),
            health_url: "http://127.0.0.1:8889/healthz".to_string(),
            started_at: guard.mcp_service_started_at,
            last_error: guard.mcp_service_last_error.clone(),
            message: if guard.mcp_service_running {
                format!(
                    "MCP 独立服务已启动，当前注册 {} 个工具",
                    guard.mcp_tools.len()
                )
            } else {
                "MCP 独立服务未启动".to_string()
            },
        }
    }

    pub async fn start_mcp_service(&self) -> McpServiceStatusPayload {
        let mut guard = self.inner.write().await;
        guard.mcp_service_running = true;
        guard.mcp_service_started_at = Some(now_ts() * 1000);
        guard.mcp_service_last_error.clear();
        drop(guard);
        self.mcp_service_status().await
    }

    pub async fn stop_mcp_service(&self) -> McpServiceStatusPayload {
        let mut guard = self.inner.write().await;
        guard.mcp_service_running = false;
        drop(guard);
        self.mcp_service_status().await
    }

    pub async fn test_mcp_tool(
        &self,
        input: crate::models::McpTestRequest,
    ) -> Result<McpTestResult, String> {
        let guard = self.inner.read().await;
        let tool = guard
            .mcp_tools
            .iter()
            .find(|item| item.name == input.name)
            .cloned()
            .ok_or_else(|| "工具不存在".to_string())?;
        let args = match input.args {
            serde_json::Value::Null => serde_json::json!({}),
            other => other,
        };
        let object = args
            .as_object()
            .ok_or_else(|| "args 必须是 JSON 对象".to_string())?;
        let mut missing = vec![];
        for param in tool.params.iter().filter(|item| item.required) {
            if !object.contains_key(&param.name) {
                missing.push(param.name.clone());
            }
        }
        if !missing.is_empty() {
            return Err(format!("缺少必填参数: {}", missing.join(", ")));
        }

        let validation = if object.is_empty() {
            "empty payload"
        } else {
            "schema accepted"
        };
        Ok(McpTestResult {
            tool: tool.name.clone(),
            accepted: true,
            validation: validation.to_string(),
            executed_at: now_ts() * 1000,
            output: serde_json::json!({
                "tool": tool.name,
                "echo": object,
                "responseTypes": tool.response.iter().map(|item| item.type_name.clone()).collect::<Vec<_>>(),
                "serverState": if guard.mcp_service_running { "running" } else { "stopped" },
            }),
        })
    }

    pub async fn list_skill_tools(&self) -> Vec<SkillToolDefinition> {
        vec![
            SkillToolDefinition {
                key: "shell".to_string(),
                label: "Shell".to_string(),
                summary: "执行本地命令、脚本和校验流程".to_string(),
            },
            SkillToolDefinition {
                key: "filesystem".to_string(),
                label: "Filesystem".to_string(),
                summary: "读取、写入和组织工作区文件".to_string(),
            },
            SkillToolDefinition {
                key: "http".to_string(),
                label: "HTTP".to_string(),
                summary: "拉取外部文档、状态页和在线资产".to_string(),
            },
        ]
    }

    pub async fn list_skills(&self) -> Vec<SkillSummary> {
        let guard = self.inner.read().await;
        guard
            .skills
            .iter()
            .map(|item| SkillSummary {
                name: item.name.clone(),
                description: item.description.clone(),
                enabled: item.enabled,
                tags: item.tags.clone(),
                updated_at: item.updated_at,
            })
            .collect()
    }

    pub async fn get_skill_detail(&self, name: &str) -> Option<SkillRecord> {
        let guard = self.inner.read().await;
        guard.skills.iter().find(|item| item.name == name).cloned()
    }

    pub async fn save_skill(&self, input: crate::models::SkillSaveRequest) -> SkillRecord {
        let mut guard = self.inner.write().await;
        let now = now_ts() * 1000;
        if let Some(existing) = guard.skills.iter_mut().find(|item| item.name == input.name) {
            existing.description = input.description;
            existing.allowed_tools = input.allowed_tools;
            existing.context = input.context;
            existing.agent = input.agent;
            existing.markdown = input.markdown;
            existing.enabled = input.enabled;
            existing.tags = sanitize_tags(input.tags);
            existing.updated_at = now;
            return existing.clone();
        }

        let created = SkillRecord {
            name: input.name,
            description: input.description,
            allowed_tools: input.allowed_tools,
            context: input.context,
            agent: input.agent,
            markdown: input.markdown,
            enabled: input.enabled,
            tags: sanitize_tags(input.tags),
            created_at: now,
            updated_at: now,
            scripts: vec![],
            resources: vec![],
            references: vec![],
            templates: vec![],
        };
        guard.skills.insert(0, created.clone());
        created
    }

    pub async fn delete_skill(&self, name: &str) -> bool {
        let mut guard = self.inner.write().await;
        let before = guard.skills.len();
        guard.skills.retain(|item| item.name != name);
        before != guard.skills.len()
    }

    pub async fn create_skill_asset(
        &self,
        skill_name: &str,
        kind: &str,
        name: &str,
    ) -> Result<SkillAssetRecord, String> {
        let mut guard = self.inner.write().await;
        let skill = guard
            .skills
            .iter_mut()
            .find(|item| item.name == skill_name)
            .ok_or_else(|| "技能不存在".to_string())?;
        let now = now_ts() * 1000;
        let target = select_skill_assets(skill, kind)?;
        if target.iter().any(|item| item.name == name) {
            return Err("文件已存在".to_string());
        }
        let file = SkillAssetRecord {
            name: name.to_string(),
            content: String::new(),
            updated_at: now,
        };
        target.push(file.clone());
        skill.updated_at = now;
        Ok(file)
    }

    pub async fn get_skill_asset(
        &self,
        skill_name: &str,
        kind: &str,
        name: &str,
    ) -> Result<SkillAssetRecord, String> {
        let guard = self.inner.read().await;
        let skill = guard
            .skills
            .iter()
            .find(|item| item.name == skill_name)
            .ok_or_else(|| "技能不存在".to_string())?;
        select_skill_assets_readonly(skill, kind)?
            .iter()
            .find(|item| item.name == name)
            .cloned()
            .ok_or_else(|| "文件不存在".to_string())
    }

    pub async fn save_skill_asset(
        &self,
        skill_name: &str,
        kind: &str,
        name: &str,
        content: String,
    ) -> Result<SkillAssetRecord, String> {
        let mut guard = self.inner.write().await;
        let skill = guard
            .skills
            .iter_mut()
            .find(|item| item.name == skill_name)
            .ok_or_else(|| "技能不存在".to_string())?;
        let now = now_ts() * 1000;
        let updated = {
            let target = select_skill_assets(skill, kind)?;
            if let Some(existing) = target.iter_mut().find(|item| item.name == name) {
                existing.content = content.clone();
                existing.updated_at = now;
                Some(existing.clone())
            } else {
                None
            }
        };
        if let Some(existing) = updated {
            skill.updated_at = now;
            return Ok(existing);
        }
        let created = SkillAssetRecord {
            name: name.to_string(),
            content,
            updated_at: now,
        };
        let target = select_skill_assets(skill, kind)?;
        target.push(created.clone());
        skill.updated_at = now;
        Ok(created)
    }
}
