use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use kernel::pagination::{Page, PageSliceRequest};

use crate::{
    application::{
        AppError, AppResult, PasswordHasher, ReplaceUserRecord, SystemUserProvider, SystemUserRecord, UserAuthRecord, UserListFilter, UserRepository,
    },
    domain::{NewUser, ProfileUpdate, ReplaceUser, User, UserFormOptions, UserId, UserProfileGroups},
};
use types::{
    rbac::{DataScopeFilter, RoleSummary},
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

mod filters;
mod online_session_store;
mod repository;

pub(crate) use online_session_store::MemoryOnlineSessionStore;

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

    pub(crate) fn with_nick_name(mut self, nick_name: &str) -> Self {
        self.user.nick_name = nick_name.into();
        self
    }

    pub(crate) fn with_email(mut self, email: &str) -> Self {
        self.user.email = email.into();
        self
    }

    pub(crate) fn with_sex(mut self, sex: &str) -> Self {
        self.user.sex = sex.into();
        self
    }

    pub(crate) fn with_role_ids(mut self, ids: Vec<&str>) -> Self {
        self.user.role_ids = ids.iter().map(|id| (*id).into()).collect();
        self.user.roles = self.user.role_ids.iter().map(|id| role_summary(id)).collect();
        self
    }

    pub(crate) fn with_post_ids(mut self, ids: Vec<&str>) -> Self {
        self.user.post_ids = ids.into_iter().map(str::to_owned).collect();
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
