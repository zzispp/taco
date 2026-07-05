use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use kernel::pagination::{Page, PageSliceRequest};

use crate::{
    application::{
        AppError, AppResult, PasswordHasher, ReplaceUserRecord, SystemUserProvider, SystemUserRecord, UserAuthRecord, UserListFilter, UserRepository,
    },
    domain::{NewUser, ReplaceUser, User, UserFormOptions, UserId},
};
use types::{
    rbac::{DATA_SCOPE_ALL, DATA_SCOPE_CUSTOM, DATA_SCOPE_DEPT, DATA_SCOPE_SELF, DataScopeFilter, RoleSummary},
    system::{Post, TreeSelectNode},
};

pub(crate) const VALID_PASSWORD: &str = "secret123";

#[derive(Clone, Default)]
pub(crate) struct MemoryUserRepository {
    state: Arc<Mutex<RepositoryState>>,
}

#[derive(Default)]
struct RepositoryState {
    next_id: u64,
    users: Vec<StoredUser>,
    created: Vec<ReplaceUserRecord>,
    replaced: Vec<(UserId, ReplaceUserRecord)>,
    deleted: Vec<UserId>,
    logins: Vec<UserId>,
}

#[derive(Clone)]
pub(crate) struct StoredUser {
    user: User,
    password_hash: String,
}

#[derive(Clone)]
pub(crate) struct TestPasswordHasher;

#[derive(Clone)]
pub(crate) struct TestSystemUserProvider {
    record: SystemUserRecord,
}

impl MemoryUserRepository {
    pub(crate) fn with_user(user: StoredUser) -> Self {
        let repository = Self::default();
        repository.state.lock().unwrap().users.push(user);
        repository
    }

    pub(crate) fn with_users(users: Vec<StoredUser>) -> Self {
        let repository = Self::default();
        repository.state.lock().unwrap().users = users;
        repository
    }

    pub(crate) fn created_records(&self) -> Vec<ReplaceUserRecord> {
        self.state.lock().unwrap().created.clone()
    }

    pub(crate) fn replaced_records(&self) -> Vec<(UserId, ReplaceUserRecord)> {
        self.state.lock().unwrap().replaced.clone()
    }

    pub(crate) fn deleted_records(&self) -> Vec<UserId> {
        self.state.lock().unwrap().deleted.clone()
    }

    pub(crate) fn login_records(&self) -> Vec<UserId> {
        self.state.lock().unwrap().logins.clone()
    }
}

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

impl PasswordHasher for TestPasswordHasher {
    fn hash(&self, password: &str) -> AppResult<String> {
        Ok(format!("hashed:{password}"))
    }

    fn verify(&self, password: &str, password_hash: &str) -> AppResult<bool> {
        Ok(password_hash == format!("hashed:{password}"))
    }
}

impl SystemUserProvider for TestSystemUserProvider {
    fn system_user(&self) -> Option<SystemUserRecord> {
        Some(self.record.clone())
    }
}

impl StoredUser {
    pub(crate) fn with_id(mut self, id: UserId) -> Self {
        self.user.id = id;
        self
    }

    pub(crate) fn with_dept_id(mut self, dept_id: &str) -> Self {
        self.user.dept_id = Some(dept_id.into());
        self
    }

    fn auth_record(&self) -> UserAuthRecord {
        UserAuthRecord {
            user: self.user.clone(),
            password_hash: self.password_hash.clone(),
        }
    }
}

pub(crate) fn new_user(username: &str) -> NewUser {
    NewUser {
        username: username.into(),
        password: VALID_PASSWORD.into(),
        nick_name: username.trim().into(),
        dept_id: Some("103".into()),
        email: format!("{}@example.com", username.trim()),
        phonenumber: Some("15888888888".into()),
        sex: "2".into(),
        status: "0".into(),
        remark: None,
        role_ids: vec!["1".into()],
        post_ids: vec!["1".into()],
    }
}

pub(crate) fn replace_user(username: &str, is_active: bool) -> ReplaceUser {
    ReplaceUser {
        username: username.into(),
        password: Some(VALID_PASSWORD.into()),
        nick_name: username.trim().into(),
        dept_id: Some("103".into()),
        email: format!("{}@example.com", username.trim()),
        phonenumber: Some("15888888888".into()),
        sex: "2".into(),
        status: if is_active { "0".into() } else { "1".into() },
        remark: None,
        role_ids: vec!["1".into()],
        post_ids: vec!["1".into()],
    }
}

pub(crate) fn stored_user(id: u64, username: &str, password_hash: &str) -> StoredUser {
    StoredUser {
        user: User {
            id: user_id(id),
            username: username.into(),
            nick_name: username.into(),
            dept_id: Some("103".into()),
            email: format!("{username}@example.com"),
            phonenumber: Some("15888888888".into()),
            sex: "2".into(),
            avatar: None,
            status: "0".into(),
            auth_source: constants::auth::DEFAULT_AUTH_SOURCE.into(),
            email_verified: false,
            system: false,
            remark: None,
            roles: vec![admin_role()],
            role_ids: vec!["1".into()],
            post_ids: vec!["1".into()],
            permissions: vec!["system:user:list".into()],
            create_time: String::new(),
        },
        password_hash: password_hash.into(),
    }
}

pub(crate) fn system_user() -> TestSystemUserProvider {
    TestSystemUserProvider {
        record: SystemUserRecord {
            user: User {
                id: user_id(0),
                username: "admin".into(),
                nick_name: "admin".into(),
                dept_id: None,
                email: "admin@example.com".into(),
                phonenumber: None,
                sex: "2".into(),
                avatar: None,
                status: "0".into(),
                auth_source: constants::auth::DEFAULT_AUTH_SOURCE.into(),
                email_verified: true,
                system: true,
                remark: None,
                roles: vec![admin_role()],
                role_ids: vec!["1".into()],
                post_ids: vec![],
                permissions: vec!["system:user:list".into()],
                create_time: String::new(),
            },
            password_hash: format!("hashed:{VALID_PASSWORD}"),
        },
    }
}

fn next_user_id(state: &mut RepositoryState) -> UserId {
    state.next_id += 1;
    user_id(state.next_id)
}

fn find_stored_user_mut<'a>(state: &'a mut RepositoryState, id: &UserId) -> AppResult<&'a mut StoredUser> {
    state.users.iter_mut().find(|stored| stored.user.id == *id).ok_or(AppError::NotFound)
}

fn replace_stored_user(state: &mut RepositoryState, id: &UserId, record: &ReplaceUserRecord) -> AppResult<User> {
    let stored = find_stored_user_mut(state, id)?;
    stored.user = user_from_record(id.clone(), record);
    if let Some(password_hash) = &record.password_hash {
        stored.password_hash = password_hash.clone();
    }
    Ok(stored.user.clone())
}

fn user_from_record(id: UserId, record: &ReplaceUserRecord) -> User {
    User {
        id,
        username: record.username.clone(),
        nick_name: record.nick_name.clone(),
        dept_id: record.dept_id.clone(),
        email: record.email.clone(),
        phonenumber: record.phonenumber.clone(),
        sex: record.sex.clone(),
        avatar: None,
        status: record.status.clone(),
        auth_source: constants::auth::DEFAULT_AUTH_SOURCE.into(),
        email_verified: false,
        system: false,
        remark: record.remark.clone(),
        roles: record.role_ids.iter().map(|id| role_summary(id)).collect(),
        role_ids: record.role_ids.clone(),
        post_ids: record.post_ids.clone(),
        permissions: vec!["system:user:list".into()],
        create_time: String::new(),
    }
}

pub(crate) fn user_id(id: u64) -> UserId {
    UserId(format!("018f0000-0000-7000-8000-{id:012}"))
}

fn admin_role() -> RoleSummary {
    role_summary("1")
}

fn role_summary(id: &str) -> RoleSummary {
    RoleSummary {
        role_id: id.into(),
        role_name: "超级管理员".into(),
        role_key: "admin".into(),
    }
}

fn memory_scope_matches(user: &User, scope: &DataScopeFilter) -> bool {
    match scope.data_scope.as_str() {
        DATA_SCOPE_ALL => true,
        DATA_SCOPE_CUSTOM => user.dept_id.as_ref().is_some_and(|id| scope.dept_ids.contains(id)),
        DATA_SCOPE_DEPT => user.dept_id == scope.dept_id,
        DATA_SCOPE_SELF => user.id.0 == scope.user_id,
        _ => user.dept_id == scope.dept_id || user.dept_id.as_ref().is_some_and(|id| scope.dept_ids.contains(id)),
    }
}

fn memory_filter_matches(user: &User, filter: &UserListFilter) -> bool {
    contains_filter(&user.username, &filter.username)
        && contains_optional_filter(&user.phonenumber, &filter.phonenumber)
        && exact_filter(&user.status, &filter.status)
        && exact_optional_filter(&user.dept_id, &filter.dept_id)
}

fn contains_filter(value: &str, filter: &Option<String>) -> bool {
    filter.as_ref().is_none_or(|needle| value.contains(needle))
}

fn contains_optional_filter(value: &Option<String>, filter: &Option<String>) -> bool {
    filter.as_ref().is_none_or(|needle| value.as_ref().is_some_and(|item| item.contains(needle)))
}

fn exact_filter(value: &str, filter: &Option<String>) -> bool {
    filter.as_ref().is_none_or(|expected| value == expected)
}

fn exact_optional_filter(value: &Option<String>, filter: &Option<String>) -> bool {
    filter.as_ref().is_none_or(|expected| value.as_deref() == Some(expected.as_str()))
}
