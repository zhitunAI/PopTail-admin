use super::seed::*;
use super::*;
use std::collections::HashSet;

impl AppState {
    fn apply_menu_upsert(
        flat: &mut Vec<MenuInfo>,
        input: crate::models::MenuUpsertRequest,
    ) -> MenuInfo {
        let node_id = input
            .id
            .unwrap_or_else(|| flat.iter().map(|item| item.id).max().unwrap_or(0) + 1);
        let menu_btn = normalize_menu_buttons(node_id, input.menu_btn);
        let parameters = normalize_menu_parameters(node_id, input.parameters);

        if let Some(existing) = flat.iter_mut().find(|item| item.id == node_id) {
            existing.parent_id = input.parent_id;
            existing.name = input.name;
            existing.path = input.path;
            existing.component = input.component;
            existing.sort = input.sort;
            existing.hidden = input.hidden;
            existing.meta = input.meta;
            existing.menu_btn = menu_btn;
            existing.parameters = parameters;
        } else {
            flat.push(MenuInfo {
                id: node_id,
                parent_id: input.parent_id,
                name: input.name,
                path: input.path,
                component: input.component,
                sort: input.sort,
                hidden: input.hidden,
                meta: input.meta,
                menu_btn,
                parameters,
                children: vec![],
            });
        }

        flat.iter().find(|item| item.id == node_id).cloned().unwrap_or(MenuInfo {
            id: node_id,
            parent_id: 0,
            name: String::new(),
            path: String::new(),
            component: String::new(),
            sort: 0,
            hidden: false,
            meta: MenuMeta {
                title: String::new(),
                icon: String::new(),
            },
            menu_btn: vec![],
            parameters: vec![],
            children: vec![],
        })
    }

    pub async fn list_users(&self, query: UserListRequest) -> PageResult<UserInfo> {
        let guard = self.inner.read().await;
        let mut items: Vec<UserInfo> = guard
            .users_by_id
            .values()
            .map(|item| item.info.clone())
            .collect();
        items.sort_by_key(|item| item.id);

        let username = query.username.unwrap_or_default().to_lowercase();
        let nick_name = query.nick_name.unwrap_or_default().to_lowercase();
        let phone = query.phone.unwrap_or_default().to_lowercase();
        let email = query.email.unwrap_or_default().to_lowercase();

        items.retain(|item| {
            (username.is_empty() || item.user_name.to_lowercase().contains(&username))
                && (nick_name.is_empty() || item.nick_name.to_lowercase().contains(&nick_name))
                && (phone.is_empty() || item.phone.to_lowercase().contains(&phone))
                && (email.is_empty() || item.email.to_lowercase().contains(&email))
        });

        page_result(items, query.page, query.page_size)
    }

    pub async fn list_authorities(&self) -> Vec<AuthorityInfo> {
        let guard = self.inner.read().await;
        guard.authorities.clone()
    }

    pub async fn list_menu_tree(&self) -> Vec<MenuInfo> {
        let guard = self.inner.read().await;
        guard.menus.clone()
    }

    pub async fn get_user_ids_by_authority(&self, authority_id: u32) -> Vec<u64> {
        let guard = self.inner.read().await;
        let mut user_ids = guard
            .users_by_id
            .values()
            .filter(|record| {
                record.info.authority_id == authority_id
                    || record
                        .info
                        .authorities
                        .iter()
                        .any(|item| item.authority_id == authority_id)
            })
            .map(|record| record.info.id)
            .collect::<Vec<_>>();
        user_ids.sort_unstable();
        user_ids
    }

    pub async fn set_role_users(
        &self,
        authority_id: u32,
        user_ids: Vec<u64>,
    ) -> Result<(), String> {
        let selected_authority = flatten_authorities(&self.list_authorities().await)
            .into_iter()
            .find(|item| item.authority_id == authority_id)
            .ok_or_else(|| "角色不存在".to_string())?;

        let mut guard = self.inner.write().await;
        let selected = user_ids.into_iter().collect::<HashSet<_>>();
        let all_authorities = flatten_authorities(&guard.authorities);

        for record in guard.users_by_id.values_mut() {
            let had_primary = record.info.authority_id == authority_id;
            let should_have = selected.contains(&record.info.id) || had_primary;
            if should_have {
                if !record
                    .info
                    .authorities
                    .iter()
                    .any(|item| item.authority_id == authority_id)
                {
                    record.info.authorities.push(selected_authority.clone());
                }
            } else {
                record
                    .info
                    .authorities
                    .retain(|item| item.authority_id != authority_id);
            }
            if record.info.authorities.is_empty() {
                record.info.authorities = all_authorities.clone();
            }
        }

        let refreshed = guard
            .users_by_id
            .values()
            .map(|record| (record.info.user_name.clone(), record.clone()))
            .collect::<HashMap<_, _>>();
        guard.users_by_name = refreshed;
        Ok(())
    }

    pub async fn get_menu_authority(&self, authority_id: u32) -> Vec<MenuInfo> {
        let guard = self.inner.read().await;
        let flat_menus = flatten_menus_for_state(&guard.menus);
        let selected = guard
            .authority_menu_ids
            .get(&authority_id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<HashSet<_>>();
        flat_menus
            .into_iter()
            .filter(|menu| selected.contains(&menu.id))
            .collect()
    }

    pub async fn set_menu_authority(
        &self,
        authority_id: u32,
        menu_ids: Vec<u64>,
    ) -> Result<(), String> {
        {
            let mut guard = self.inner.write().await;
            guard
                .authority_menu_ids
                .insert(authority_id, dedupe_u64(menu_ids));
        }
        self.persist_menu_state().await;
        Ok(())
    }

    pub async fn get_authority_buttons(&self, authority_id: u32, menu_id: u64) -> Vec<u64> {
        let guard = self.inner.read().await;
        guard
            .authority_button_ids
            .get(&(authority_id, menu_id))
            .cloned()
            .unwrap_or_default()
    }

    pub async fn get_authority_buttons_batch(
        &self,
        authority_id: u32,
        menu_ids: Vec<u64>,
    ) -> Vec<(u64, Vec<u64>)> {
        let guard = self.inner.read().await;
        menu_ids
            .into_iter()
            .map(|menu_id| {
                let selected = guard
                    .authority_button_ids
                    .get(&(authority_id, menu_id))
                    .cloned()
                    .unwrap_or_default();
                (menu_id, selected)
            })
            .collect()
    }

    pub async fn set_authority_buttons(
        &self,
        authority_id: u32,
        menu_id: u64,
        selected: Vec<u64>,
    ) -> Result<(), String> {
        {
            let mut guard = self.inner.write().await;
            guard
                .authority_button_ids
                .insert((authority_id, menu_id), dedupe_u64(selected));
        }
        self.persist_menu_state().await;
        Ok(())
    }

    pub async fn get_menu_roles(&self, menu_id: u64) -> (Vec<u32>, Vec<u32>) {
        let guard = self.inner.read().await;
        let authority_ids = guard
            .authority_menu_ids
            .iter()
            .filter_map(|(authority_id, menu_ids)| {
                menu_ids.contains(&menu_id).then_some(*authority_id)
            })
            .collect::<Vec<_>>();
        let menu_name = flatten_menus_for_state(&guard.menus)
            .into_iter()
            .find(|menu| menu.id == menu_id)
            .map(|menu| menu.name)
            .unwrap_or_default();
        let default_router_authority_ids = flatten_authorities(&guard.authorities)
            .into_iter()
            .filter(|authority| authority.default_router == menu_name)
            .map(|authority| authority.authority_id)
            .collect::<Vec<_>>();
        (authority_ids, default_router_authority_ids)
    }

    pub async fn set_menu_roles(
        &self,
        menu_id: u64,
        authority_ids: Vec<u32>,
    ) -> Result<(), String> {
        let selected = authority_ids.into_iter().collect::<HashSet<_>>();
        {
            let mut guard = self.inner.write().await;
            for (authority_id, menu_ids) in &mut guard.authority_menu_ids {
                if selected.contains(authority_id) {
                    if !menu_ids.contains(&menu_id) {
                        menu_ids.push(menu_id);
                    }
                } else {
                    menu_ids.retain(|id| *id != menu_id);
                }
                *menu_ids = dedupe_u64(menu_ids.clone());
            }
            for authority_id in selected {
                guard
                    .authority_menu_ids
                    .entry(authority_id)
                    .or_insert_with(|| vec![menu_id]);
            }
        }
        self.persist_menu_state().await;
        Ok(())
    }

    pub async fn list_api_entries(&self, query: ApiListRequest) -> PageResult<ApiInfo> {
        let guard = self.inner.read().await;
        let mut items = guard.apis.clone();

        let path = query.path.unwrap_or_default().to_lowercase();
        let description = query.description.unwrap_or_default().to_lowercase();
        let api_group = query.api_group.unwrap_or_default().to_lowercase();
        let method = query.method.unwrap_or_default().to_lowercase();

        items.retain(|item| {
            (path.is_empty() || item.path.to_lowercase().contains(&path))
                && (description.is_empty()
                    || item.description.to_lowercase().contains(&description))
                && (api_group.is_empty() || item.api_group.to_lowercase().contains(&api_group))
                && (method.is_empty() || item.method.to_lowercase() == method)
        });

        page_result(items, query.page, query.page_size)
    }

    pub async fn list_dictionaries(&self) -> PageResult<DictionaryInfo> {
        let guard = self.inner.read().await;
        page_result(guard.dictionaries.clone(), Some(1), Some(50))
    }

    pub async fn list_params(&self) -> PageResult<ParamInfo> {
        let guard = self.inner.read().await;
        page_result(guard.params.clone(), Some(1), Some(50))
    }

    pub async fn upsert_param(&self, input: crate::models::ParamUpsertRequest) -> ParamInfo {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard
            .params
            .iter_mut()
            .find(|item| input.id == Some(item.id) || item.key == input.key)
        {
            existing.key = input.key;
            existing.value = input.value;
            existing.desc = input.desc;
            return existing.clone();
        }

        let next_id = guard.params.iter().map(|item| item.id).max().unwrap_or(0) + 1;
        let created = ParamInfo {
            id: next_id,
            key: input.key,
            value: input.value,
            desc: input.desc,
        };
        guard.params.push(created.clone());
        created
    }

    pub async fn update_profile(
        &self,
        user_id: u64,
        input: crate::models::ProfileUpdateRequest,
    ) -> Result<UserInfo, String> {
        let mut guard = self.inner.write().await;
        let user_name = guard
            .users_by_id
            .get(&user_id)
            .map(|record| record.info.user_name.clone())
            .ok_or_else(|| "用户不存在".to_string())?;

        let updated = {
            let record = guard
                .users_by_id
                .get_mut(&user_id)
                .ok_or_else(|| "用户不存在".to_string())?;
            record.info.nick_name = input.nick_name;
            record.info.phone = input.phone;
            record.info.email = input.email;
            let mut password_changed = false;
            if let Some(password) = input
                .password
                .as_ref()
                .map(|item| item.trim())
                .filter(|item| !item.is_empty())
            {
                record.password =
                    hash_password(password).map_err(|err| format!("密码更新失败: {err}"))?;
                record.token_version = record.token_version.saturating_add(1);
                password_changed = true;
            }
            (
                record.info.clone(),
                record.password.clone(),
                record.token_version,
                password_changed,
            )
        };

        if let Some(record) = guard.users_by_name.get_mut(&user_name) {
            record.info = updated.0.clone();
            record.password = updated.1.clone();
            record.token_version = updated.2;
        }
        drop(guard);

        if updated.3 {
            self.persistence
                .upsert_user_auth_version(user_id, updated.2)
                .await;
        }

        Ok(updated.0)
    }

    pub async fn upsert_user(
        &self,
        input: crate::models::UserUpsertRequest,
    ) -> Result<UserInfo, String> {
        let authorities = flatten_authorities(&self.list_authorities().await);
        let selected_authority = authorities
            .iter()
            .find(|item| item.authority_id == input.authority_id)
            .cloned()
            .ok_or_else(|| "角色不存在".to_string())?;
        let mut selected_authority_ids = input.authority_ids.clone();
        if !selected_authority_ids.contains(&input.authority_id) {
            selected_authority_ids.insert(0, input.authority_id);
        }
        selected_authority_ids.sort_unstable();
        selected_authority_ids.dedup();
        let switchable_authorities = selected_authority_ids
            .iter()
            .filter_map(|authority_id| {
                authorities
                    .iter()
                    .find(|item| item.authority_id == *authority_id)
                    .cloned()
            })
            .collect::<Vec<_>>();
        let switchable_authorities = if switchable_authorities.is_empty() {
            vec![selected_authority.clone()]
        } else {
            switchable_authorities
        };

        let mut guard = self.inner.write().await;
        let target_id = input
            .id
            .unwrap_or_else(|| guard.users_by_id.keys().max().copied().unwrap_or(0) + 1);

        if guard
            .users_by_name
            .iter()
            .any(|(user_name, record)| *user_name == input.user_name && record.info.id != target_id)
        {
            return Err("用户名已存在".to_string());
        }

        let existing_user_name = guard
            .users_by_id
            .get(&target_id)
            .map(|record| record.info.user_name.clone());
        if let Some(old_user_name) = existing_user_name {
            guard.users_by_name.remove(&old_user_name);
        }

        let existing_uuid = guard
            .users_by_id
            .get(&target_id)
            .map(|record| record.info.uuid);
        let password = guard
            .users_by_id
            .get(&target_id)
            .map(|record| record.password.clone())
            .unwrap_or_else(|| match read_default_user_password() {
                Ok(value) => hash_password(value.as_str()).expect("hash default user password"),
                Err(err) => panic!("{err}"),
            });
        let token_version = guard
            .users_by_id
            .get(&target_id)
            .map(|record| record.token_version)
            .unwrap_or(1);

        let info = UserInfo {
            id: target_id,
            uuid: existing_uuid.unwrap_or_else(Uuid::new_v4),
            user_name: input.user_name,
            nick_name: input.nick_name,
            authority_id: input.authority_id,
            authority: selected_authority,
            authorities: switchable_authorities,
            header_img: format!("https://example.com/pop-tail-user-{}.png", target_id),
            phone: input.phone,
            email: input.email,
            enable: input.enable,
        };
        let record = UserRecord {
            info: info.clone(),
            password,
            token_version,
        };

        guard
            .users_by_name
            .insert(info.user_name.clone(), record.clone());
        guard.users_by_id.insert(target_id, record);
        Ok(info)
    }

    pub async fn upsert_authority(
        &self,
        input: crate::models::AuthorityUpsertRequest,
    ) -> Result<AuthorityInfo, String> {
        let mut guard = self.inner.write().await;
        let mut flat = flatten_authorities(&guard.authorities);
        let normalized_status = if input.enable == 2 || input.status == 2 {
            2
        } else {
            1
        };
        let node_id = input.id.unwrap_or_else(|| {
            flat.iter()
                .find(|item| item.authority_id == input.authority_id)
                .map(|item| item.id)
                .unwrap_or_else(|| flat.iter().map(|item| item.id).max().unwrap_or(0) + 1)
        });

        if flat
            .iter()
            .any(|item| item.authority_id == input.authority_id && item.id != node_id)
        {
            return Err("角色标识已存在".to_string());
        }

        if let Some(existing) = flat.iter_mut().find(|item| item.id == node_id) {
            existing.authority_id = input.authority_id;
            existing.authority_name = input.authority_name;
            existing.default_router = input.default_router;
            existing.parent_id = input.parent_id;
            existing.enable = normalized_status;
            existing.status = normalized_status;
        } else {
            flat.push(AuthorityInfo {
                id: node_id,
                authority_id: input.authority_id,
                authority_name: input.authority_name,
                default_router: input.default_router,
                parent_id: input.parent_id,
                enable: normalized_status,
                status: normalized_status,
                children: vec![],
            });
        }

        let saved = flat
            .iter()
            .find(|item| item.id == node_id)
            .cloned()
            .ok_or_else(|| "保存角色失败".to_string())?;
        guard.authorities = rebuild_authority_tree(flat);
        Ok(saved)
    }

    pub async fn upsert_menu(&self, input: crate::models::MenuUpsertRequest) -> MenuInfo {
        let saved = {
            let mut guard = self.inner.write().await;
            let mut flat = flatten_menus_for_state(&guard.menus);
            let saved = Self::apply_menu_upsert(&mut flat, input);
            guard.menus = rebuild_menu_tree(flat);
            saved
        };
        self.persist_menu_state().await;
        saved
    }

    pub async fn upsert_menus(
        &self,
        inputs: Vec<crate::models::MenuUpsertRequest>,
    ) -> Vec<MenuInfo> {
        let saved_items = {
            let mut guard = self.inner.write().await;
            let mut flat = flatten_menus_for_state(&guard.menus);
            let mut saved_items = Vec::with_capacity(inputs.len());

            for input in inputs {
                saved_items.push(Self::apply_menu_upsert(&mut flat, input));
            }

            guard.menus = rebuild_menu_tree(flat);
            saved_items
        };
        self.persist_menu_state().await;
        saved_items
    }

    pub async fn delete_menu(&self, menu_id: u64) -> Result<(), String> {
        let deleted_ids = {
            let mut guard = self.inner.write().await;
            let flat = flatten_menus_for_state(&guard.menus);
            let existing_ids = flat.iter().map(|item| item.id).collect::<HashSet<_>>();
            if !existing_ids.contains(&menu_id) {
                return Err("菜单不存在".to_string());
            }

            let mut delete_ids = HashSet::new();
            let mut stack = vec![menu_id];
            while let Some(current_id) = stack.pop() {
                if !delete_ids.insert(current_id) {
                    continue;
                }
                for child in flat.iter().filter(|item| item.parent_id == current_id) {
                    stack.push(child.id);
                }
            }

            let delete_ids_vec = delete_ids.iter().copied().collect::<Vec<_>>();
            let deleted_names = flat
                .iter()
                .filter(|item| delete_ids.contains(&item.id))
                .map(|item| item.name.clone())
                .collect::<HashSet<_>>();

            let next_flat = flat
                .into_iter()
                .filter(|item| !delete_ids.contains(&item.id))
                .collect::<Vec<_>>();
            guard.menus = rebuild_menu_tree(next_flat);

            for menu_ids in guard.authority_menu_ids.values_mut() {
                menu_ids.retain(|id| !delete_ids.contains(id));
                *menu_ids = dedupe_u64(menu_ids.clone());
            }

            guard.authority_button_ids.retain(|(_, id), _| !delete_ids.contains(id));

            let fallback_route = "dashboard".to_string();
            fn repair_default_router(
                items: &mut [AuthorityInfo],
                deleted_names: &HashSet<String>,
                fallback_route: &str,
            ) {
                for item in items {
                    if deleted_names.contains(&item.default_router) {
                        item.default_router = fallback_route.to_string();
                    }
                    repair_default_router(&mut item.children, deleted_names, fallback_route);
                }
            }
            repair_default_router(&mut guard.authorities, &deleted_names, &fallback_route);

            delete_ids_vec
        };

        if !deleted_ids.is_empty() {
            self.persist_menu_state().await;
        }
        Ok(())
    }

    pub async fn upsert_api(&self, input: crate::models::ApiUpsertRequest) -> ApiInfo {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard.apis.iter_mut().find(|item| {
            input.id == Some(item.id) || (item.path == input.path && item.method == input.method)
        }) {
            existing.path = input.path;
            existing.api_group = input.api_group;
            existing.description = input.description;
            existing.method = input.method;
            return existing.clone();
        }

        let next_id = guard.apis.iter().map(|item| item.id).max().unwrap_or(0) + 1;
        let created = ApiInfo {
            id: next_id,
            path: input.path,
            api_group: input.api_group,
            description: input.description,
            method: input.method,
        };
        guard.apis.push(created.clone());
        created
    }

    pub async fn upsert_dictionary(
        &self,
        input: crate::models::DictionaryUpsertRequest,
    ) -> DictionaryInfo {
        let mut guard = self.inner.write().await;
        if let Some(existing) = guard
            .dictionaries
            .iter_mut()
            .find(|item| input.id == Some(item.id) || item.type_name == input.type_name)
        {
            existing.name = input.name;
            existing.type_name = input.type_name;
            existing.status = input.status;
            existing.desc = input.desc;
            return existing.clone();
        }

        let next_id = guard
            .dictionaries
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            + 1;
        let created = DictionaryInfo {
            id: next_id,
            name: input.name,
            type_name: input.type_name,
            status: input.status,
            desc: input.desc,
        };
        guard.dictionaries.push(created.clone());
        created
    }

    pub async fn update_system_config(
        &self,
        input: crate::models::SystemConfigUpdateRequest,
    ) -> SystemConfigInfo {
        let mut guard = self.inner.write().await;
        guard.system_config = SystemConfigInfo {
            bind_address: input.bind_address,
            database_url: input.database_url,
            redis_url: input.redis_url,
            multipoint_enabled: input.multipoint_enabled,
            compatibility_refresh_headers: input.compatibility_refresh_headers,
        };
        redact_system_config(guard.system_config.clone())
    }
}
