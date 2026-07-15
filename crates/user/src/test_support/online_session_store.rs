use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use kernel::pagination::CursorPage;
use rbac::domain::DataScope;

use crate::application::{AppResult, OnlineSession, OnlineSessionPageRequest, OnlineSessionSearch, OnlineSessionStore};
use crate::domain::UserId;

const NANOS_PER_MILLISECOND: i128 = 1_000_000;

#[derive(Clone, Default)]
pub(crate) struct MemoryOnlineSessionStore {
    sessions: Arc<Mutex<Vec<OnlineSession>>>,
}

impl MemoryOnlineSessionStore {
    pub(crate) async fn save_session(&self, session: OnlineSession) {
        self.create(&session).await.unwrap();
    }

    pub(crate) fn sessions(&self) -> Vec<OnlineSession> {
        self.sessions.lock().unwrap().clone()
    }
}

#[async_trait]
impl OnlineSessionStore for MemoryOnlineSessionStore {
    async fn create(&self, session: &OnlineSession) -> AppResult<()> {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.retain(|item| item.token_id != session.token_id);
        sessions.push(session.clone());
        Ok(())
    }

    async fn renew_active(&self, token_id: &str, user_id: &UserId, expires_at: i64) -> AppResult<Option<OnlineSession>> {
        let mut sessions = self.sessions.lock().unwrap();
        let Some(session) = sessions
            .iter_mut()
            .find(|item| item.token_id == token_id && item.user_id == *user_id && item.expires_at > now_millis())
        else {
            return Ok(None);
        };
        session.expires_at = expires_at;
        Ok(Some(session.clone()))
    }

    async fn find_active(&self, token_id: &str, user_id: &UserId) -> AppResult<Option<OnlineSession>> {
        Ok(self
            .sessions
            .lock()
            .unwrap()
            .iter()
            .find(|item| item.token_id == token_id && item.user_id == *user_id && item.expires_at > now_millis())
            .cloned())
    }

    async fn find_active_by_token(&self, token_id: &str) -> AppResult<Option<OnlineSession>> {
        Ok(self
            .sessions()
            .into_iter()
            .find(|session| session.token_id == token_id && session.expires_at > now_millis()))
    }

    async fn delete(&self, token_id: &str) -> AppResult<()> {
        self.sessions.lock().unwrap().retain(|item| item.token_id != token_id);
        Ok(())
    }

    async fn page_active(&self, request: OnlineSessionPageRequest) -> AppResult<CursorPage<OnlineSession>> {
        let now = now_millis();
        let mut sessions = self
            .sessions()
            .into_iter()
            .filter(|session| session.expires_at > now && matches_search(session, &request.search) && matches_scope(session, &request))
            .collect::<Vec<_>>();
        sessions.sort_by(|left, right| right.login_time.cmp(&left.login_time).then_with(|| left.token_id.cmp(&right.token_id)));
        let items = sessions.into_iter().take(request.page.limit as usize).collect();
        Ok(CursorPage::new(items, None, None))
    }
}

fn matches_search(session: &OnlineSession, search: &OnlineSessionSearch) -> bool {
    contains(&session.ipaddr, &search.ipaddr)
        && contains(&session.user_name, &search.user_name)
        && contains(&session.login_location, &search.login_location)
        && contains(&session.browser, &search.browser)
        && contains(&session.os, &search.os)
        && search.begin_time.is_none_or(|begin| session.login_time >= begin)
        && search.end_time.is_none_or(|end| session.login_time <= end)
}

fn contains(value: &str, filter: &Option<String>) -> bool {
    filter.as_ref().is_none_or(|needle| value.to_lowercase().contains(&needle.to_lowercase()))
}

fn matches_scope(session: &OnlineSession, request: &OnlineSessionPageRequest) -> bool {
    let Some(scope) = &request.scope else { return true };
    match scope.data_scope {
        DataScope::All => true,
        DataScope::Custom => session.dept_id.as_ref().is_some_and(|id| scope.dept_ids.contains(id)),
        DataScope::Department | DataScope::DepartmentAndChildren => session.dept_id == scope.dept_id,
        DataScope::SelfOnly => session.user_id.0 == scope.user_id,
    }
}

fn now_millis() -> i64 {
    i64::try_from(time::OffsetDateTime::now_utc().unix_timestamp_nanos() / NANOS_PER_MILLISECOND).unwrap()
}
