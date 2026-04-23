use super::seed::*;
use super::*;

impl AppState {
    pub async fn list_llm_configs(&self) -> PageResult<LlmConfigRecord> {
        let guard = self.inner.read().await;
        page_result(guard.llm_configs.clone(), Some(1), Some(50))
    }

    pub async fn upsert_llm_config(
        &self,
        input: crate::models::LlmConfigUpsertRequest,
    ) -> LlmConfigRecord {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard
            .llm_configs
            .iter_mut()
            .find(|item| input.id == Some(item.id))
        {
            existing.provider = input.provider;
            existing.model = input.model;
            existing.base_url = input.base_url;
            existing.api_key = input.api_key;
            existing.status = input.status;
            return existing.clone();
        }

        let next_id = next_id(guard.llm_configs.iter().map(|item| item.id));
        let created = LlmConfigRecord {
            id: next_id,
            provider: input.provider,
            model: input.model,
            base_url: input.base_url,
            api_key: input.api_key,
            status: input.status,
        };
        guard.llm_configs.insert(0, created.clone());
        created
    }

    pub async fn delete_llm_config(&self, id: u64) -> bool {
        let mut guard = self.inner.write().await;
        let before = guard.llm_configs.len();
        guard.llm_configs.retain(|item| item.id != id);
        before != guard.llm_configs.len()
    }

    pub async fn list_frontend_nav(&self) -> PageResult<FrontendNavRecord> {
        let guard = self.inner.read().await;
        let mut rows = guard.frontend_nav.clone();
        rows.sort_by_key(|item| (item.order, item.id));
        page_result(rows, Some(1), Some(100))
    }

    pub async fn upsert_frontend_nav(
        &self,
        input: crate::models::FrontendNavUpsertRequest,
    ) -> FrontendNavRecord {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard
            .frontend_nav
            .iter_mut()
            .find(|item| input.id == Some(item.id))
        {
            existing.title = input.title;
            existing.path = input.path;
            existing.icon = input.icon;
            existing.order = input.order;
            existing.visible = input.visible;
            return existing.clone();
        }

        let created = FrontendNavRecord {
            id: next_id(guard.frontend_nav.iter().map(|item| item.id)),
            title: input.title,
            path: input.path,
            icon: input.icon,
            order: input.order,
            visible: input.visible,
        };
        guard.frontend_nav.insert(0, created.clone());
        created
    }

    pub async fn delete_frontend_nav(&self, id: u64) -> bool {
        let mut guard = self.inner.write().await;
        let before = guard.frontend_nav.len();
        guard.frontend_nav.retain(|item| item.id != id);
        before != guard.frontend_nav.len()
    }

    pub async fn frontend_settings(&self) -> FrontendSettingsRecord {
        let guard = self.inner.read().await;
        guard.frontend_settings.clone()
    }

    pub async fn update_frontend_settings(
        &self,
        input: FrontendSettingsUpdateRequest,
    ) -> FrontendSettingsRecord {
        let mut guard = self.inner.write().await;
        guard.frontend_settings = FrontendSettingsRecord {
            logo_url: input.logo_url,
            site_name: input.site_name,
            site_slogan: input.site_slogan,
            site_description: input.site_description,
            record_number: input.record_number,
            smtp_host: input.smtp_host,
            smtp_port: input.smtp_port,
            smtp_user: input.smtp_user,
            smtp_password: input.smtp_password,
            mail_from: input.mail_from,
        };
        guard.frontend_settings.clone()
    }

    pub async fn list_article_categories(&self) -> PageResult<ArticleCategoryRecord> {
        let guard = self.inner.read().await;
        let mut rows = guard.article_categories.clone();
        rows.sort_by_key(|item| (item.sort, item.id));
        page_result(rows, Some(1), Some(100))
    }

    pub async fn upsert_article_category(
        &self,
        input: crate::models::ArticleCategoryUpsertRequest,
    ) -> ArticleCategoryRecord {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard
            .article_categories
            .iter_mut()
            .find(|item| input.id == Some(item.id))
        {
            existing.name = input.name;
            existing.slug = input.slug;
            existing.sort = input.sort;
            existing.status = input.status;
            return existing.clone();
        }

        let created = ArticleCategoryRecord {
            id: next_id(guard.article_categories.iter().map(|item| item.id)),
            name: input.name,
            slug: input.slug,
            sort: input.sort,
            status: input.status,
        };
        guard.article_categories.insert(0, created.clone());
        created
    }

    pub async fn delete_article_category(&self, id: u64) -> bool {
        let mut guard = self.inner.write().await;
        let before = guard.article_categories.len();
        guard.article_categories.retain(|item| item.id != id);
        before != guard.article_categories.len()
    }

    pub async fn list_articles(&self) -> PageResult<ArticleRecord> {
        let guard = self.inner.read().await;
        page_result(guard.articles.clone(), Some(1), Some(100))
    }

    pub async fn upsert_article(
        &self,
        input: crate::models::ArticleUpsertRequest,
    ) -> ArticleRecord {
        let mut guard = self.inner.write().await;
        let updated_at = timestamp_ms_string();
        if let Some(existing) = guard
            .articles
            .iter_mut()
            .find(|item| input.id == Some(item.id))
        {
            existing.category_id = input.category_id;
            existing.title = input.title;
            existing.slug = input.slug;
            existing.content = input.content;
            existing.status = input.status;
            existing.updated_at = updated_at;
            return existing.clone();
        }

        let created = ArticleRecord {
            id: next_id(guard.articles.iter().map(|item| item.id)),
            category_id: input.category_id,
            title: input.title,
            slug: input.slug,
            content: input.content,
            status: input.status,
            updated_at,
        };
        guard.articles.insert(0, created.clone());
        created
    }

    pub async fn delete_article(&self, id: u64) -> bool {
        let mut guard = self.inner.write().await;
        let before = guard.articles.len();
        guard.articles.retain(|item| item.id != id);
        before != guard.articles.len()
    }

    pub async fn list_members(&self) -> PageResult<MemberRecord> {
        let guard = self.inner.read().await;
        page_result(guard.members.clone(), Some(1), Some(100))
    }

    pub async fn upsert_member(&self, input: crate::models::MemberUpsertRequest) -> MemberRecord {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard
            .members
            .iter_mut()
            .find(|item| input.id == Some(item.id))
        {
            existing.email = input.email;
            existing.nickname = input.nickname;
            existing.provider = input.provider;
            existing.status = input.status;
            existing.last_login = input.last_login.unwrap_or_else(timestamp_ms_string);
            existing.console_path = input.console_path;
            return existing.clone();
        }

        let created = MemberRecord {
            id: next_id(guard.members.iter().map(|item| item.id)),
            email: input.email,
            nickname: input.nickname,
            provider: input.provider,
            status: input.status,
            last_login: input.last_login.unwrap_or_else(timestamp_ms_string),
            console_path: input.console_path,
        };
        guard.members.insert(0, created.clone());
        created
    }

    pub async fn delete_member(&self, id: u64) -> bool {
        let mut guard = self.inner.write().await;
        let before = guard.members.len();
        guard.members.retain(|item| item.id != id);
        before != guard.members.len()
    }

    pub async fn list_console_menus(&self) -> PageResult<ConsoleMenuRecord> {
        let guard = self.inner.read().await;
        let mut rows = guard.console_menus.clone();
        rows.sort_by_key(|item| (item.order, item.id));
        page_result(rows, Some(1), Some(100))
    }

    pub async fn upsert_console_menu(
        &self,
        input: crate::models::ConsoleMenuUpsertRequest,
    ) -> ConsoleMenuRecord {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard
            .console_menus
            .iter_mut()
            .find(|item| input.id == Some(item.id))
        {
            existing.title = input.title;
            existing.group = input.group;
            existing.path = input.path;
            existing.icon = input.icon;
            existing.order = input.order;
            existing.visible = input.visible;
            return existing.clone();
        }

        let created = ConsoleMenuRecord {
            id: next_id(guard.console_menus.iter().map(|item| item.id)),
            title: input.title,
            group: input.group,
            path: input.path,
            icon: input.icon,
            order: input.order,
            visible: input.visible,
        };
        guard.console_menus.insert(0, created.clone());
        created
    }

    pub async fn delete_console_menu(&self, id: u64) -> bool {
        let mut guard = self.inner.write().await;
        let before = guard.console_menus.len();
        guard.console_menus.retain(|item| item.id != id);
        before != guard.console_menus.len()
    }
}
