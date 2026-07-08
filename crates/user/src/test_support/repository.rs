use super::*;

#[async_trait]
impl UserRepository for MemoryUserRepository {
    async fn create(&self, record: ReplaceUserRecord) -> AppResult<User> {
        let mut state = self.state.lock().unwrap();
        let id = next_user_id(&mut state);
        let user = user_from_record(id, &record);
        state.users.push(StoredUser {
            user: user.clone(),
            password_hash: record.password_hash.clone().unwrap_or_default(),
        });
        state.created.push(record);
        Ok(user)
    }

    async fn replace(&self, id: UserId, record: ReplaceUserRecord) -> AppResult<User> {
        let mut state = self.state.lock().unwrap();
        let user = replace_stored_user(&mut state, &id, &record)?;
        state.replaced.push((id, record));
        Ok(user)
    }

    async fn delete(&self, id: UserId) -> AppResult<()> {
        self.state.lock().unwrap().deleted.push(id);
        Ok(())
    }

    async fn delete_many(&self, ids: Vec<UserId>) -> AppResult<()> {
        self.state.lock().unwrap().deleted.extend(ids);
        Ok(())
    }

    async fn find_by_id(&self, id: UserId) -> AppResult<Option<User>> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .users
            .iter()
            .find(|stored| stored.user.id == id)
            .map(|stored| stored.user.clone()))
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .users
            .iter()
            .find(|stored| stored.user.email == email)
            .map(|stored| stored.user.clone()))
    }

    async fn find_by_phone(&self, phone: &str) -> AppResult<Option<User>> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .users
            .iter()
            .find(|stored| stored.user.phonenumber.as_deref() == Some(phone))
            .map(|stored| stored.user.clone()))
    }

    async fn find_auth_by_username(&self, username: &str) -> AppResult<Option<UserAuthRecord>> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .users
            .iter()
            .find(|stored| stored.user.username == username)
            .map(StoredUser::auth_record))
    }

    async fn find_auth_by_email(&self, email: &str) -> AppResult<Option<UserAuthRecord>> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .users
            .iter()
            .find(|stored| stored.user.email == email)
            .map(StoredUser::auth_record))
    }

    async fn find_auth_by_id(&self, id: UserId) -> AppResult<Option<UserAuthRecord>> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .users
            .iter()
            .find(|stored| stored.user.id == id)
            .map(StoredUser::auth_record))
    }

    async fn record_login(&self, id: UserId) -> AppResult<()> {
        self.state.lock().unwrap().logins.push(id);
        Ok(())
    }

    async fn list(&self, filter: UserListFilter) -> AppResult<Page<User>> {
        let page = filter.page;
        let request = PageSliceRequest {
            offset: (page.page - 1) * page.page_size,
            limit: page.page_size,
            page: page.page,
            page_size: page.page_size,
        };
        self.list_slice(filter, request).await
    }

    async fn list_scoped(&self, filter: UserListFilter, scope: DataScopeFilter) -> AppResult<Page<User>> {
        let page = filter.page;
        let state = self.state.lock().unwrap();
        let filtered = state
            .users
            .iter()
            .filter(|stored| memory_scope_matches(&stored.user, &scope))
            .map(|stored| stored.user.clone())
            .collect::<Vec<_>>();
        let start = ((page.page - 1) * page.page_size) as usize;
        let end = start.saturating_add(page.page_size as usize).min(filtered.len());
        let items = if start >= filtered.len() { vec![] } else { filtered[start..end].to_vec() };
        Ok(Page {
            items,
            total: filtered.len() as u64,
            page: page.page,
            page_size: page.page_size,
        })
    }

    async fn list_scoped_ids(&self, ids: Vec<UserId>, scope: DataScopeFilter) -> AppResult<Vec<UserId>> {
        let state = self.state.lock().unwrap();
        Ok(state
            .users
            .iter()
            .filter(|stored| ids.contains(&stored.user.id) && memory_scope_matches(&stored.user, &scope))
            .map(|stored| stored.user.id.clone())
            .collect())
    }

    async fn list_slice(&self, filter: UserListFilter, request: PageSliceRequest) -> AppResult<Page<User>> {
        let state = self.state.lock().unwrap();
        let filtered = state
            .users
            .iter()
            .filter(|stored| memory_filter_matches(&stored.user, &filter))
            .map(|stored| stored.user.clone())
            .collect::<Vec<_>>();
        let start = request.offset as usize;
        let end = start.saturating_add(request.limit as usize).min(filtered.len());
        let items = if start >= filtered.len() { vec![] } else { filtered[start..end].to_vec() };
        Ok(Page {
            items,
            total: filtered.len() as u64,
            page: request.page,
            page_size: request.page_size,
        })
    }

    async fn update_password(&self, id: UserId, password_hash: String) -> AppResult<()> {
        let mut state = self.state.lock().unwrap();
        let stored = find_stored_user_mut(&mut state, &id)?;
        stored.password_hash = password_hash;
        Ok(())
    }

    async fn update_profile(&self, id: UserId, profile: ProfileUpdate) -> AppResult<User> {
        let mut state = self.state.lock().unwrap();
        let stored = find_stored_user_mut(&mut state, &id)?;
        stored.user.nick_name = profile.nick_name;
        stored.user.phonenumber = profile.phonenumber;
        stored.user.email = profile.email;
        stored.user.sex = profile.sex;
        Ok(stored.user.clone())
    }

    async fn update_avatar(&self, id: UserId, avatar: String) -> AppResult<User> {
        let mut state = self.state.lock().unwrap();
        let stored = find_stored_user_mut(&mut state, &id)?;
        stored.user.avatar = Some(avatar);
        Ok(stored.user.clone())
    }

    async fn update_status(&self, id: UserId, status: String) -> AppResult<User> {
        let mut state = self.state.lock().unwrap();
        let stored = find_stored_user_mut(&mut state, &id)?;
        stored.user.status = status;
        Ok(stored.user.clone())
    }

    async fn replace_roles(&self, id: UserId, role_ids: Vec<String>) -> AppResult<User> {
        let mut state = self.state.lock().unwrap();
        let stored = find_stored_user_mut(&mut state, &id)?;
        stored.user.roles = role_ids.iter().map(|id| role_summary(id)).collect();
        stored.user.role_ids = role_ids;
        Ok(stored.user.clone())
    }

    async fn profile_groups(&self, id: UserId) -> AppResult<UserProfileGroups> {
        let state = self.state.lock().unwrap();
        let user = state
            .users
            .iter()
            .find(|stored| stored.user.id == id)
            .map(|stored| stored.user.clone())
            .ok_or(AppError::NotFound)?;
        Ok(UserProfileGroups {
            role_group: user.roles.iter().map(|role| role.role_name.clone()).collect::<Vec<_>>().join(","),
            post_group: user.post_ids.join(","),
            dept_name: user.dept_id.map(|id| format!("部门{id}")),
        })
    }

    async fn form_options(&self) -> AppResult<UserFormOptions> {
        Ok(UserFormOptions {
            roles: vec![types::rbac::RoleOption {
                role_id: "1".into(),
                role_name: "超级管理员".into(),
                role_key: "admin".into(),
                status: "0".into(),
            }],
            posts: vec![Post {
                post_id: "1".into(),
                post_code: "ceo".into(),
                post_name: "董事长".into(),
                post_sort: 1,
                status: "0".into(),
                remark: None,
                create_time: "2026-01-01 00:00:00".into(),
            }],
            depts: vec![TreeSelectNode {
                id: "103".into(),
                label: "研发部门".into(),
                parent_id: "100".into(),
                disabled: false,
                children: vec![],
            }],
        })
    }
}
