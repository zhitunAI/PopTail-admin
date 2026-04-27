use super::seed::*;
use super::*;

impl AppState {
    pub async fn get_global_constraint(&self) -> GlobalConstraintRecord {
        let guard = self.inner.read().await;
        guard.global_constraint.clone()
    }

    pub async fn save_global_constraint(&self, content: String) -> GlobalConstraintRecord {
        let mut guard = self.inner.write().await;
        guard.global_constraint = GlobalConstraintRecord {
            content,
            updated_at: now_ts() * 1000,
        };
        guard.global_constraint.clone()
    }

    pub fn list_operation_logs(&self) -> PageResult<OperationLogInfo> {
        page_result(
            vec![
                OperationLogInfo {
                    id: 1,
                    ip: "127.0.0.1".to_string(),
                    method: "POST".to_string(),
                    path: "/base/login".to_string(),
                    status: 200,
                    latency: 12,
                },
                OperationLogInfo {
                    id: 2,
                    ip: "127.0.0.1".to_string(),
                    method: "POST".to_string(),
                    path: "/ai/moderation/decision".to_string(),
                    status: 200,
                    latency: 26,
                },
            ],
            Some(1),
            Some(50),
        )
    }

    pub fn list_login_logs(&self) -> PageResult<LoginLogInfo> {
        page_result(
            vec![
                LoginLogInfo {
                    id: 1,
                    username: "admin".to_string(),
                    ip: "127.0.0.1".to_string(),
                    status: true,
                    error_message: "登录成功".to_string(),
                },
                LoginLogInfo {
                    id: 2,
                    username: "disabled".to_string(),
                    ip: "127.0.0.1".to_string(),
                    status: false,
                    error_message: "用户被禁止登录".to_string(),
                },
            ],
            Some(1),
            Some(50),
        )
    }

    pub async fn dictionary_detail_tree(&self, dictionary_id: u64) -> Vec<DictionaryDetailInfo> {
        let guard = self.inner.read().await;
        guard
            .dictionary_details
            .get(&dictionary_id)
            .cloned()
            .unwrap_or_default()
    }

    pub async fn upsert_dictionary_detail(
        &self,
        input: crate::models::DictionaryDetailUpsertRequest,
    ) -> DictionaryDetailInfo {
        let mut guard = self.inner.write().await;
        let roots = guard
            .dictionary_details
            .entry(input.sys_dictionary_id)
            .or_insert_with(Vec::new);

        let mut flat = flatten_dictionary_nodes(roots);
        let next_id = flat.iter().map(|item| item.id).max().unwrap_or(0) + 1;
        let node_id = input.id.unwrap_or(next_id);

        if let Some(existing) = flat.iter_mut().find(|item| item.id == node_id) {
            existing.label = input.label;
            existing.value = input.value;
            existing.extend = input.extend;
            existing.level = input.level.max(1);
            existing.status = input.status;
            existing.sort = input.sort;
            existing.parent_id = input.parent_id;
        } else {
            flat.push(DictionaryDetailInfo {
                id: node_id,
                label: input.label,
                value: input.value,
                extend: input.extend,
                level: input.level.max(1),
                status: input.status,
                sort: input.sort,
                parent_id: input.parent_id,
                children: vec![],
            });
        }

        let target = flat
            .iter()
            .find(|item| item.id == node_id)
            .cloned()
            .unwrap_or(DictionaryDetailInfo {
                id: node_id,
                label: String::new(),
                value: String::new(),
                extend: String::new(),
                level: 1,
                status: true,
                sort: 1,
                parent_id: None,
                children: vec![],
            });

        *roots = rebuild_dictionary_tree(flat);
        target
    }

    pub fn list_error_logs(&self) -> PageResult<ErrorLogInfo> {
        page_result(
            vec![
                ErrorLogInfo {
                    id: 1,
                    error: "权限包刷新失败".to_string(),
                    path: "/casbin/getPolicyPathByAuthorityId".to_string(),
                    status: "已处理".to_string(),
                },
                ErrorLogInfo {
                    id: 2,
                    error: "导出模板配置缺失映射".to_string(),
                    path: "/sysExportTemplate/getSysExportTemplateList".to_string(),
                    status: "待处理".to_string(),
                },
            ],
            Some(1),
            Some(50),
        )
    }

    pub fn list_export_templates(&self) -> PageResult<ExportTemplateInfo> {
        page_result(
            vec![
                ExportTemplateInfo {
                    id: 1,
                    name: "接口导出模板".to_string(),
                    template_id: "api".to_string(),
                    desc: "接口列表导出模板".to_string(),
                },
                ExportTemplateInfo {
                    id: 2,
                    name: "用户导出模板".to_string(),
                    template_id: "user".to_string(),
                    desc: "用户列表导出模板".to_string(),
                },
            ],
            Some(1),
            Some(50),
        )
    }

    pub fn runtime_info(&self) -> RuntimeInfo {
        RuntimeInfo {
            os: std::env::consts::OS.to_string(),
            cpu_cores: std::thread::available_parallelism()
                .map(|value| value.get())
                .unwrap_or(1),
            rust_version: env!("CARGO_PKG_VERSION").to_string(),
            db_backend: std::env::var("POP_TAIL_DATABASE_URL")
                .ok()
                .filter(|value| !value.is_empty())
                .map(|value| {
                    if value.starts_with("postgres://") || value.starts_with("postgresql://") {
                        "postgres".to_string()
                    } else {
                        "sqlite".to_string()
                    }
                })
                .unwrap_or_else(|| "sqlite".to_string()),
            redis_enabled: std::env::var("POP_TAIL_REDIS_URL")
                .map(|value| !value.is_empty())
                .unwrap_or(false),
        }
    }

    pub async fn system_config_info(&self) -> SystemConfigInfo {
        let guard = self.inner.read().await;
        redact_system_config(guard.system_config.clone())
    }
}
