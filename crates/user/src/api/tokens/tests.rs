use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use kernel::pagination::{CursorPage, CursorPageRequest};

use super::{TokenIssueInput, TokenService, TokenSettings, TokenSettingsReader, TokenTtlConfig};
use crate::{
    application::{AppError, AppResult, OnlineSession, OnlineSessionPageRequest, OnlineSessionSearch, OnlineSessionStore},
    domain::UserId,
    test_support::MemoryOnlineSessionStore,
};

const TEST_ACCESS_TTL_SECONDS: u64 = 900;
const TEST_REFRESH_TTL_SECONDS: u64 = 604_800;
const EXPECTED_COMPENSATION_DELETES: usize = 1;

#[tokio::test]
async fn refresh_rejects_access_token() {
    let service = token_service();
    let tokens = service.issue_pair(issue_input()).await.unwrap();

    let result = service.refresh(&tokens.access_token).await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[tokio::test]
async fn refresh_accepts_refresh_token_and_issues_access_token() {
    let service = token_service();
    let tokens = service.issue_pair(issue_input()).await.unwrap();

    let (user_id, refreshed) = service.refresh(&tokens.refresh_token).await.unwrap();

    assert_eq!(user_id, self::user_id());
    assert_eq!(service.validate_access(&refreshed.access_token).await.unwrap(), self::user_id());
}

#[tokio::test]
async fn validate_access_rejects_refresh_token() {
    let service = token_service();
    let tokens = service.issue_pair(issue_input()).await.unwrap();

    let result = service.validate_access(&tokens.refresh_token).await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[tokio::test]
async fn force_logout_invalidates_existing_tokens() {
    let service = token_service();
    let tokens = service.issue_pair(issue_input()).await.unwrap();
    let session = service.online_sessions(all_sessions_page()).await.unwrap().items.remove(0);

    service.force_logout(&session.token_id).await.unwrap();

    assert!(matches!(service.validate_access(&tokens.access_token).await, Err(AppError::Unauthorized)));
    assert!(matches!(service.refresh(&tokens.refresh_token).await, Err(AppError::Unauthorized)));
}

#[tokio::test]
async fn token_generation_failure_does_not_create_an_online_session() {
    let sessions = Arc::new(MemoryOnlineSessionStore::default());
    let service = token_service_with_ttl(
        sessions.clone(),
        TokenTtlConfig {
            access_token_ttl_seconds: u64::MAX,
            refresh_token_ttl_seconds: TEST_REFRESH_TTL_SECONDS,
        },
    );

    let result = service.issue_pair(issue_input()).await;

    assert!(matches!(result, Err(AppError::Infrastructure(_))));
    assert_eq!(sessions.sessions(), Vec::new());
}

#[tokio::test]
async fn ambiguous_session_save_is_compensated_before_returning_failure() {
    let sessions = Arc::new(AmbiguousSaveStore::default());
    let service = token_service_with_ttl(
        sessions.clone(),
        TokenTtlConfig {
            access_token_ttl_seconds: TEST_ACCESS_TTL_SECONDS,
            refresh_token_ttl_seconds: TEST_REFRESH_TTL_SECONDS,
        },
    );

    let result = service.issue_pair(issue_input()).await;

    assert!(matches!(result, Err(AppError::Infrastructure(message)) if message == "ambiguous session save"));
    assert_eq!(sessions.sessions(), Vec::new());
    assert_eq!(sessions.delete_count(), EXPECTED_COMPENSATION_DELETES);
}

fn token_service() -> TokenService {
    token_service_with_ttl(
        Arc::new(MemoryOnlineSessionStore::default()),
        TokenTtlConfig {
            access_token_ttl_seconds: TEST_ACCESS_TTL_SECONDS,
            refresh_token_ttl_seconds: TEST_REFRESH_TTL_SECONDS,
        },
    )
}

fn token_service_with_ttl(sessions: Arc<dyn OnlineSessionStore>, ttl: TokenTtlConfig) -> TokenService {
    TokenService::with_ttl_reader(
        TokenSettings {
            secret: "test-secret-with-enough-entropy".into(),
        },
        Arc::new(TestTokenSettingsReader(ttl)),
        sessions,
    )
}

#[derive(Default)]
struct AmbiguousSaveStore {
    sessions: Mutex<Vec<OnlineSession>>,
    delete_count: Mutex<usize>,
}

impl AmbiguousSaveStore {
    fn sessions(&self) -> Vec<OnlineSession> {
        self.sessions.lock().unwrap().clone()
    }

    fn delete_count(&self) -> usize {
        *self.delete_count.lock().unwrap()
    }
}

#[async_trait]
impl OnlineSessionStore for AmbiguousSaveStore {
    async fn create(&self, session: &OnlineSession) -> AppResult<()> {
        self.sessions.lock().unwrap().push(session.clone());
        Err(AppError::Infrastructure("ambiguous session save".into()))
    }

    async fn renew_active(&self, token_id: &str, user_id: &UserId, expires_at: i64) -> AppResult<Option<OnlineSession>> {
        let mut sessions = self.sessions.lock().unwrap();
        let Some(session) = sessions.iter_mut().find(|session| session.token_id == token_id && session.user_id == *user_id) else {
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
            .find(|session| session.token_id == token_id && session.user_id == *user_id)
            .cloned())
    }

    async fn find_active_by_token(&self, token_id: &str) -> AppResult<Option<OnlineSession>> {
        Ok(self.sessions().into_iter().find(|session| session.token_id == token_id))
    }

    async fn delete(&self, token_id: &str) -> AppResult<()> {
        self.sessions.lock().unwrap().retain(|session| session.token_id != token_id);
        *self.delete_count.lock().unwrap() += 1;
        Ok(())
    }

    async fn page_active(&self, _request: OnlineSessionPageRequest) -> AppResult<CursorPage<OnlineSession>> {
        let sessions = self.sessions();
        Ok(CursorPage::new(sessions, None, None))
    }
}

fn all_sessions_page() -> OnlineSessionPageRequest {
    OnlineSessionPageRequest {
        page: CursorPageRequest::default(),
        search: OnlineSessionSearch::default(),
        scope: None,
    }
}

struct TestTokenSettingsReader(TokenTtlConfig);

#[async_trait]
impl TokenSettingsReader for TestTokenSettingsReader {
    async fn token_ttl_config(&self) -> AppResult<TokenTtlConfig> {
        Ok(self.0.clone())
    }
}

fn issue_input() -> TokenIssueInput {
    TokenIssueInput {
        user_id: user_id(),
        dept_name: Some("研发部门".into()),
        user_name: "alice".into(),
        ipaddr: "127.0.0.1".into(),
        login_location: "127.0.0.1".into(),
        browser: "Chrome".into(),
        os: "macOS".into(),
    }
}

fn user_id() -> UserId {
    UserId("018f0000-0000-7000-8000-000000000001".into())
}
