use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::application::{AppResult, OnlineSession, OnlineSessionStore};

#[derive(Clone, Default)]
pub(crate) struct MemoryOnlineSessionStore {
    sessions: Arc<Mutex<Vec<OnlineSession>>>,
}

impl MemoryOnlineSessionStore {
    pub(crate) async fn save_session(&self, session: OnlineSession) {
        self.save(&session, 900).await.unwrap();
    }

    pub(crate) fn sessions(&self) -> Vec<OnlineSession> {
        self.sessions.lock().unwrap().clone()
    }
}

#[async_trait]
impl OnlineSessionStore for MemoryOnlineSessionStore {
    async fn save(&self, session: &OnlineSession, _ttl_seconds: u64) -> AppResult<()> {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.retain(|item| item.token_id != session.token_id);
        sessions.push(session.clone());
        Ok(())
    }

    async fn find(&self, token_id: &str) -> AppResult<Option<OnlineSession>> {
        Ok(self.sessions.lock().unwrap().iter().find(|item| item.token_id == token_id).cloned())
    }

    async fn delete(&self, token_id: &str) -> AppResult<()> {
        self.sessions.lock().unwrap().retain(|item| item.token_id != token_id);
        Ok(())
    }

    async fn list(&self) -> AppResult<Vec<OnlineSession>> {
        Ok(self.sessions())
    }
}
