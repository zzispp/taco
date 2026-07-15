use async_trait::async_trait;
use kernel::pagination::{CursorPage, CursorPageRequest};
use rbac::domain::DataScopeFilter;

use crate::{application::AppResult, domain::UserId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OnlineSession {
    pub token_id: String,
    pub user_id: UserId,
    pub dept_id: Option<String>,
    pub dept_name: Option<String>,
    pub user_name: String,
    pub ipaddr: String,
    pub login_location: String,
    pub browser: String,
    pub os: String,
    pub login_time: i64,
    pub expires_at: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OnlineSessionFilter {
    pub ipaddr: Option<String>,
    pub user_name: Option<String>,
    pub login_location: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OnlineSessionSearch {
    pub ipaddr: Option<String>,
    pub user_name: Option<String>,
    pub login_location: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub begin_time: Option<i64>,
    pub end_time: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OnlineSessionPageRequest {
    pub page: CursorPageRequest,
    pub search: OnlineSessionSearch,
    pub scope: Option<DataScopeFilter>,
}

#[async_trait]
pub trait OnlineSessionStore: Send + Sync + 'static {
    async fn create(&self, session: &OnlineSession) -> AppResult<()>;
    async fn renew_active(&self, token_id: &str, user_id: &UserId, expires_at: i64) -> AppResult<Option<OnlineSession>>;
    async fn find_active(&self, token_id: &str, user_id: &UserId) -> AppResult<Option<OnlineSession>>;
    async fn find_active_by_token(&self, token_id: &str) -> AppResult<Option<OnlineSession>>;
    async fn delete(&self, token_id: &str) -> AppResult<()>;
    async fn page_active(&self, request: OnlineSessionPageRequest) -> AppResult<CursorPage<OnlineSession>>;
}

#[async_trait]
pub trait OnlineSessionCleanup: Send + Sync + 'static {
    async fn delete_expired(&self, batch_size: usize) -> AppResult<u64>;
}
