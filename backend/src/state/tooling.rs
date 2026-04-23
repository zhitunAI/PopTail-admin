use super::seed::*;
use super::*;

impl AppState {
    pub async fn list_api_tokens(&self) -> PageResult<ApiTokenRecord> {
        let guard = self.inner.read().await;
        page_result(guard.api_tokens.clone(), Some(1), Some(50))
    }

    pub async fn issue_api_token(
        &self,
        input: crate::models::ApiTokenIssueRequest,
    ) -> ApiTokenRecord {
        let mut guard = self.inner.write().await;
        let next_id = guard
            .api_tokens
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            + 1;
        let created = ApiTokenRecord {
            id: next_id,
            name: input.name,
            scope: input.scope,
            ttl: input.ttl,
            status: "已登记".to_string(),
        };
        guard.api_tokens.insert(0, created.clone());
        guard.api_tokens.truncate(20);
        created
    }

    pub async fn clear_api_tokens(&self) {
        let mut guard = self.inner.write().await;
        guard.api_tokens.clear();
    }

    pub async fn list_packages(&self) -> PageResult<PackageRecord> {
        let guard = self.inner.read().await;
        page_result(guard.packages.clone(), Some(1), Some(50))
    }

    pub async fn upsert_package(
        &self,
        input: crate::models::PackageUpsertRequest,
    ) -> PackageRecord {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard
            .packages
            .iter_mut()
            .find(|item| input.id == Some(item.id) || item.name == input.name)
        {
            existing.kind = input.kind;
            existing.output = input.output;
            existing.summary = input.summary;
            return existing.clone();
        }

        let next_id = guard.packages.iter().map(|item| item.id).max().unwrap_or(0) + 1;
        let created = PackageRecord {
            id: next_id,
            name: input.name,
            kind: input.kind,
            output: input.output,
            summary: input.summary,
        };
        guard.packages.insert(0, created.clone());
        guard.packages.truncate(20);
        created
    }

    pub async fn list_plugin_manifests(&self) -> PageResult<PluginManifestRecord> {
        let guard = self.inner.read().await;
        page_result(guard.plugin_manifests.clone(), Some(1), Some(20))
    }

    pub async fn upsert_plugin_manifest(
        &self,
        input: crate::models::PluginManifestUpsertRequest,
    ) -> PluginManifestRecord {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard
            .plugin_manifests
            .iter_mut()
            .find(|item| item.plugin_name == input.plugin_name)
        {
            existing.menu_group = input.menu_group;
            existing.menu_ids = input.menu_ids;
            existing.api_ids = input.api_ids;
            existing.dictionary_ids = input.dictionary_ids;
            existing.saved_at = now_ts() * 1000;
            return existing.clone();
        }

        let next_id = guard
            .plugin_manifests
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            + 1;
        let created = PluginManifestRecord {
            id: next_id,
            plugin_name: input.plugin_name,
            menu_group: input.menu_group,
            menu_ids: input.menu_ids,
            api_ids: input.api_ids,
            dictionary_ids: input.dictionary_ids,
            saved_at: now_ts() * 1000,
        };
        guard.plugin_manifests.insert(0, created.clone());
        guard.plugin_manifests.truncate(20);
        created
    }

    pub async fn list_plugin_installs(&self) -> PageResult<PluginInstallRecord> {
        let guard = self.inner.read().await;
        page_result(guard.plugin_installs.clone(), Some(1), Some(20))
    }

    pub async fn install_plugin(
        &self,
        input: crate::models::PluginInstallRequest,
    ) -> PluginInstallRecord {
        let mut guard = self.inner.write().await;
        let next_id = guard
            .plugin_installs
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            + 1;
        let created = PluginInstallRecord {
            id: next_id,
            name: input.file_name.trim_end_matches(".zip").to_string(),
            kind: input.kind,
            target: input.target,
            manifest: input.manifest,
            status: "已登记".to_string(),
            created_at: now_ts() * 1000,
        };
        guard.plugin_installs.insert(0, created.clone());
        guard.plugin_installs.truncate(20);
        created
    }

    pub async fn list_auto_code_registry(&self) -> PageResult<AutoCodeRegistryRecord> {
        let guard = self.inner.read().await;
        page_result(guard.auto_code_registry.clone(), Some(1), Some(20))
    }

    pub async fn append_auto_code_registry(
        &self,
        input: crate::models::AutoCodeRegistryUpsertRequest,
    ) -> AutoCodeRegistryRecord {
        let mut guard = self.inner.write().await;
        let next_id = guard
            .auto_code_registry
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            + 1;
        let created = AutoCodeRegistryRecord {
            id: next_id,
            created_at: now_ts() * 1000,
            payload: input.payload,
        };
        guard.auto_code_registry.insert(0, created.clone());
        guard.auto_code_registry.truncate(20);
        created
    }

    pub async fn clear_auto_code_registry(&self) {
        let mut guard = self.inner.write().await;
        guard.auto_code_registry.clear();
    }

    pub async fn list_releases(&self) -> PageResult<ReleaseRecord> {
        let guard = self.inner.read().await;
        page_result(guard.releases.clone(), Some(1), Some(20))
    }

    pub async fn append_release(
        &self,
        input: crate::models::ReleaseAppendRequest,
    ) -> ReleaseRecord {
        let mut guard = self.inner.write().await;
        let next_id = guard.releases.iter().map(|item| item.id).max().unwrap_or(0) + 1;
        let created = ReleaseRecord {
            id: next_id,
            created_at: now_ts() * 1000,
            note: input.note,
        };
        guard.releases.insert(0, created.clone());
        guard.releases.truncate(20);
        created
    }

    pub async fn list_email_records(&self) -> PageResult<EmailRecord> {
        let guard = self.inner.read().await;
        page_result(guard.email_records.clone(), Some(1), Some(20))
    }

    pub async fn send_email(
        &self,
        input: crate::models::EmailSendRequest,
        mode: &str,
    ) -> EmailRecord {
        let mut guard = self.inner.write().await;
        let next_id = guard
            .email_records
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            + 1;
        let record = EmailRecord {
            id: next_id,
            to: input
                .to
                .unwrap_or_else(|| "ops-team@gaa.local".to_string())
                .trim()
                .to_string(),
            subject: input
                .subject
                .unwrap_or_else(|| "GAA 邮件测试".to_string())
                .trim()
                .to_string(),
            body: input
                .body
                .unwrap_or_else(|| "当前邮件由 Rust 重构版邮件工作台登记。".to_string())
                .trim()
                .to_string(),
            mode: mode.to_string(),
            status: if mode == "test" {
                "测试已发送"
            } else {
                "已发送"
            }
            .to_string(),
            created_at: now_ts() * 1000,
        };
        guard.email_records.insert(0, record.clone());
        guard.email_records.truncate(20);
        record
    }

    pub async fn list_announcements(
        &self,
        input: crate::models::AnnouncementListRequest,
    ) -> PageResult<AnnouncementRecord> {
        let guard = self.inner.read().await;
        let keyword = input.title.unwrap_or_default().trim().to_lowercase();
        let rows = guard
            .announcements
            .iter()
            .filter(|item| {
                keyword.is_empty()
                    || item.title.to_lowercase().contains(&keyword)
                    || item.content.to_lowercase().contains(&keyword)
            })
            .cloned()
            .collect();
        page_result(rows, input.page, input.page_size)
    }

    pub async fn get_announcement(&self, id: u64) -> Option<AnnouncementRecord> {
        let guard = self.inner.read().await;
        guard
            .announcements
            .iter()
            .find(|item| item.id == id)
            .cloned()
    }

    pub async fn upsert_announcement(
        &self,
        input: crate::models::AnnouncementUpsertRequest,
    ) -> AnnouncementRecord {
        let mut guard = self.inner.write().await;
        let now = now_ts() * 1000;
        if let Some(existing) = guard
            .announcements
            .iter_mut()
            .find(|item| item.id == input.id.unwrap_or(0))
        {
            existing.title = input.title.trim().to_string();
            existing.content = input.content.trim().to_string();
            existing.user_id = input.user_id;
            existing.attachments = sanitize_announcement_attachments(input.attachments);
            existing.updated_at = now;
            return existing.clone();
        }

        let next_id = guard
            .announcements
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            + 1;
        let record = AnnouncementRecord {
            id: next_id,
            created_at: now,
            updated_at: now,
            title: input.title.trim().to_string(),
            content: input.content.trim().to_string(),
            user_id: input.user_id,
            attachments: sanitize_announcement_attachments(input.attachments),
        };
        guard.announcements.insert(0, record.clone());
        record
    }

    pub async fn delete_announcement(&self, id: u64) -> bool {
        let mut guard = self.inner.write().await;
        let before = guard.announcements.len();
        guard.announcements.retain(|item| item.id != id);
        before != guard.announcements.len()
    }

    pub async fn delete_announcements(&self, ids: Vec<u64>) -> usize {
        let mut guard = self.inner.write().await;
        let id_set: HashSet<u64> = ids.into_iter().collect();
        let before = guard.announcements.len();
        guard
            .announcements
            .retain(|item| !id_set.contains(&item.id));
        before.saturating_sub(guard.announcements.len())
    }

    pub async fn announcement_data_source(&self) -> AnnouncementDataSourcePayload {
        let guard = self.inner.read().await;
        let mut users = guard
            .users_by_id
            .values()
            .map(|item| SelectOption {
                label: item.info.nick_name.clone(),
                value: item.info.id,
            })
            .collect::<Vec<_>>();
        users.sort_by(|a, b| a.value.cmp(&b.value));
        AnnouncementDataSourcePayload { user_id: users }
    }
}
