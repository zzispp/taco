use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use kernel::pagination::{Page, PageRequest, PageSliceRequest};

use crate::{
    application::{AppError, AppResult, PasswordHasher, ReplaceUserRecord, SystemUserProvider, SystemUserRecord, UserAuthRecord, UserRepository},
    domain::{NewUser, ReplaceUser, User, UserId},
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
            password_hash: record.password_hash.clone(),
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

    async fn list(&self, page: PageRequest) -> AppResult<Page<User>> {
        let request = PageSliceRequest {
            offset: (page.page - 1) * page.page_size,
            limit: page.page_size,
            page: page.page,
            page_size: page.page_size,
        };
        self.list_slice(request).await
    }

    async fn list_slice(&self, request: PageSliceRequest) -> AppResult<Page<User>> {
        let state = self.state.lock().unwrap();
        let start = request.offset as usize;
        let end = start.saturating_add(request.limit as usize).min(state.users.len());
        let items = if start >= state.users.len() {
            vec![]
        } else {
            state.users[start..end].iter().map(|stored| stored.user.clone()).collect()
        };
        Ok(Page {
            items,
            total: state.users.len() as u64,
            page: request.page,
            page_size: request.page_size,
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
        email: format!("{}@example.com", username.trim()),
        role: "admin".into(),
        is_active: true,
    }
}

pub(crate) fn replace_user(username: &str, is_active: bool) -> ReplaceUser {
    ReplaceUser {
        username: username.into(),
        password: VALID_PASSWORD.into(),
        email: format!("{}@example.com", username.trim()),
        role: "admin".into(),
        is_active,
    }
}

pub(crate) fn stored_user(id: u64, username: &str, password_hash: &str) -> StoredUser {
    StoredUser {
        user: User {
            id: user_id(id),
            username: username.into(),
            email: format!("{username}@example.com"),
            role: "admin".into(),
            is_active: true,
            auth_source: constants::auth::DEFAULT_AUTH_SOURCE.into(),
            email_verified: false,
            system: false,
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
                email: "admin@example.com".into(),
                role: "admin".into(),
                is_active: true,
                auth_source: constants::auth::DEFAULT_AUTH_SOURCE.into(),
                email_verified: true,
                system: true,
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
    stored.password_hash = record.password_hash.clone();
    Ok(stored.user.clone())
}

fn user_from_record(id: UserId, record: &ReplaceUserRecord) -> User {
    User {
        id,
        username: record.username.clone(),
        email: record.email.clone(),
        role: record.role.clone(),
        is_active: record.is_active,
        auth_source: constants::auth::DEFAULT_AUTH_SOURCE.into(),
        email_verified: false,
        system: false,
    }
}

pub(crate) fn user_id(id: u64) -> UserId {
    UserId(format!("018f0000-0000-7000-8000-{id:012}"))
}
