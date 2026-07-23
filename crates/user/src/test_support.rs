use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use constants::system::{ADMIN_ROLE_KEY, STATUS_NORMAL};
use kernel::error::LocalizedError;
use kernel::pagination::CursorPage;
use rbac::domain::DataScopeFilter;

use crate::{
    application::{
        AppError, AppResult, AuthorizationUser, BootstrapAdministratorOutcome, BootstrapAdministratorRecord, BootstrapAdministratorRepository, PasswordHasher,
        ReplaceUserRecord, UserAuthRecord, UserExportRequest, UserExportSink, UserListFilter, UserRepository,
    },
    domain::{AvatarFileId, ProfileUpdate, User, UserFormOptions, UserId, UserProfileGroups},
};
use types::{
    rbac::RoleSummary,
    system::{Post, TreeSelectNode},
};

pub(crate) const VALID_PASSWORD: &str = "secret123";
pub(crate) const SYSTEM_ADMIN_ROLE_ID: &str = "admin-role";
pub(crate) const NON_SYSTEM_ADMIN_ROLE_ID: &str = "non-system-admin-role";

#[derive(Clone, Default)]
pub(crate) struct MemoryUserRepository {
    state: Arc<Mutex<RepositoryState>>,
}

#[derive(Clone, Default)]
struct RepositoryState {
    next_id: u64,
    users: Vec<StoredUser>,
    created: Vec<ReplaceUserRecord>,
    replaced: Vec<(UserId, ReplaceUserRecord)>,
    deleted: Vec<UserId>,
    logins: Vec<UserId>,
    login_ips: Vec<(UserId, String)>,
    audits: Vec<audit_contract::AuditOutboxRecord>,
    audit_failure: Option<String>,
    auth_lookup_failure: Option<String>,
}

#[derive(Clone)]
pub(crate) struct StoredUser {
    user: User,
    password_hash: String,
}

#[derive(Clone)]
pub(crate) struct TestPasswordHasher;

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

    pub(crate) fn login_ip_records(&self) -> Vec<(UserId, String)> {
        self.state.lock().unwrap().login_ips.clone()
    }

    pub(crate) fn audit_records(&self) -> Vec<audit_contract::AuditOutboxRecord> {
        self.state.lock().unwrap().audits.clone()
    }

    pub(crate) fn fail_audit_with(&self, message: &str) {
        self.state.lock().unwrap().audit_failure = Some(message.into());
    }

    pub(crate) fn fail_auth_lookup_with(&self, message: &str) {
        self.state.lock().unwrap().auth_lookup_failure = Some(message.into());
    }

    pub(crate) fn ensure_audit_available(&self) -> AppResult<()> {
        self.state
            .lock()
            .unwrap()
            .audit_failure
            .clone()
            .map_or(Ok(()), |message| Err(AppError::Infrastructure(message)))
    }
}

mod audited_repository;
mod filters;
mod fixtures;
mod login_security;
mod online_session_store;
mod repository;

pub(crate) use fixtures::{new_user, replace_user, stored_user};
pub(crate) use login_security::{MemoryLoginFailureStore, TestLoginLockConfigProvider, user_service_with_login_security};
pub(crate) use online_session_store::MemoryOnlineSessionStore;

impl PasswordHasher for TestPasswordHasher {
    fn hash(&self, password: &str) -> AppResult<String> {
        Ok(format!("hashed:{password}"))
    }

    fn verify(&self, password: &str, password_hash: &str) -> AppResult<bool> {
        Ok(password_hash == format!("hashed:{password}"))
    }
}

impl StoredUser {
    fn auth_record(&self) -> UserAuthRecord {
        UserAuthRecord {
            user: self.user.clone(),
            password_hash: self.password_hash.clone(),
        }
    }
}

fn next_user_id(state: &mut RepositoryState) -> UserId {
    state.next_id += 1;
    user_id(state.next_id)
}

fn store_created_user(state: &mut RepositoryState, record: ReplaceUserRecord) -> User {
    let id = next_user_id(state);
    let user = user_from_record(id, &record);
    state.users.push(StoredUser {
        user: user.clone(),
        password_hash: record.password_hash.clone().unwrap_or_default(),
    });
    state.created.push(record);
    user
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
        avatar_file_id: None,
        avatar_version: 0,
        status: record.status.clone(),
        auth_source: "local".into(),
        email_verified: false,
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

fn business_role() -> RoleSummary {
    role_summary("1")
}

fn role_summary(id: &str) -> RoleSummary {
    let (role_name, role_key) = match id {
        "1" => ("业务管理员", "business-admin"),
        SYSTEM_ADMIN_ROLE_ID => ("系统管理员", ADMIN_ROLE_KEY),
        NON_SYSTEM_ADMIN_ROLE_ID => ("非系统管理员", ADMIN_ROLE_KEY),
        _ => ("业务角色", "business-role"),
    };
    RoleSummary {
        role_id: id.into(),
        role_name: role_name.into(),
        role_key: role_key.into(),
    }
}

fn commit_continuous_mutation<T>(state: &mut RepositoryState, mutation: impl FnOnce(&mut RepositoryState) -> AppResult<T>) -> AppResult<T> {
    let mut next = state.clone();
    let result = mutation(&mut next)?;
    ensure_admin_continuity(state, &next)?;
    *state = next;
    Ok(result)
}

fn ensure_admin_continuity(before: &RepositoryState, after: &RepositoryState) -> AppResult<()> {
    if has_enabled_admin(before) && !has_enabled_admin(after) {
        return Err(AppError::Conflict(LocalizedError::new("errors.user.last_enabled_admin_required")));
    }
    Ok(())
}

fn has_enabled_admin(state: &RepositoryState) -> bool {
    state.users.iter().any(|stored| {
        !state.deleted.contains(&stored.user.id)
            && stored.user.status == STATUS_NORMAL
            && stored
                .user
                .roles
                .iter()
                .any(|role| role.role_id == SYSTEM_ADMIN_ROLE_ID && role.role_key == ADMIN_ROLE_KEY)
    })
}
