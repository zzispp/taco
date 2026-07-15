use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;

use crate::{
    application::{AppError, AppResult, LoginFailureStore, LoginLockConfig, LoginLockConfigProvider, UserService, UserUseCase},
    domain::UserId,
};

use super::{MemoryUserRepository, TestPasswordHasher};

const TEST_MAX_RETRY_COUNT: u32 = 5;
const TEST_LOCK_MINUTES: u64 = 10;

#[derive(Clone, Default)]
pub(crate) struct MemoryLoginFailureStore {
    state: Arc<Mutex<HashMap<String, FailureState>>>,
    clear_error: Arc<Mutex<Option<String>>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FailureState {
    count: u32,
    ttl_seconds: u64,
}

#[derive(Clone)]
pub(crate) struct TestLoginLockConfigProvider {
    config: LoginLockConfig,
}

impl MemoryLoginFailureStore {
    pub(crate) fn count(&self, user_id: &UserId) -> u32 {
        self.state.lock().unwrap().get(&user_id.0).map_or(0, |state| state.count)
    }

    pub(crate) fn ttl_seconds(&self, user_id: &UserId) -> Option<u64> {
        self.state.lock().unwrap().get(&user_id.0).map(|state| state.ttl_seconds)
    }

    pub(crate) fn fail_clear_with(&self, message: &str) {
        *self.clear_error.lock().unwrap() = Some(message.into());
    }
}

impl Default for TestLoginLockConfigProvider {
    fn default() -> Self {
        Self {
            config: LoginLockConfig {
                max_retry_count: TEST_MAX_RETRY_COUNT,
                lock_minutes: TEST_LOCK_MINUTES,
            },
        }
    }
}

#[async_trait]
impl LoginLockConfigProvider for TestLoginLockConfigProvider {
    async fn login_lock_config(&self) -> AppResult<LoginLockConfig> {
        Ok(self.config.clone())
    }
}

#[async_trait]
impl LoginFailureStore for MemoryLoginFailureStore {
    async fn failure_count(&self, user_id: &UserId) -> AppResult<u32> {
        Ok(self.count(user_id))
    }

    async fn record_failure(&self, user_id: &UserId, ttl_seconds: u64) -> AppResult<u32> {
        let mut states = self.state.lock().unwrap();
        let state = states.entry(user_id.0.clone()).or_insert(FailureState { count: 0, ttl_seconds });
        state.count += 1;
        state.ttl_seconds = ttl_seconds;
        Ok(state.count)
    }

    async fn clear_failures(&self, user_id: &UserId) -> AppResult<()> {
        if let Some(message) = self.clear_error.lock().unwrap().clone() {
            return Err(AppError::Infrastructure(message));
        }
        self.state.lock().unwrap().remove(&user_id.0);
        Ok(())
    }
}

pub(crate) fn user_service_with_login_security(repository: MemoryUserRepository) -> impl UserUseCase {
    UserService::new(repository, TestPasswordHasher).with_login_security(MemoryLoginFailureStore::default(), TestLoginLockConfigProvider::default())
}
