use std::sync::Arc;

use async_trait::async_trait;

use super::{TokenIssueInput, TokenService, TokenSettings, TokenSettingsReader, TokenTtlConfig};
use crate::{
    application::{AppError, AppResult},
    domain::UserId,
    test_support::MemoryOnlineSessionStore,
};

const TEST_ACCESS_TTL_SECONDS: u64 = 900;
const TEST_REFRESH_TTL_SECONDS: u64 = 604_800;

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn refresh_rejects_access_token() {
    let service = token_service();
    let tokens = service.issue_pair(issue_input()).await.unwrap();

    let result = service.refresh(&tokens.access_token).await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn refresh_accepts_refresh_token_and_issues_access_token() {
    let service = token_service();
    let tokens = service.issue_pair(issue_input()).await.unwrap();

    let (user_id, refreshed) = service.refresh(&tokens.refresh_token).await.unwrap();

    assert_eq!(user_id, self::user_id());
    assert_eq!(service.validate_access(&refreshed.access_token).await.unwrap(), self::user_id());
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn validate_access_rejects_refresh_token() {
    let service = token_service();
    let tokens = service.issue_pair(issue_input()).await.unwrap();

    let result = service.validate_access(&tokens.refresh_token).await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn force_logout_invalidates_existing_tokens() {
    let service = token_service();
    let tokens = service.issue_pair(issue_input()).await.unwrap();
    let session = service.online_sessions(Default::default()).await.unwrap().remove(0);

    service.force_logout(&session.token_id).await.unwrap();

    assert!(matches!(service.validate_access(&tokens.access_token).await, Err(AppError::Unauthorized)));
    assert!(matches!(service.refresh(&tokens.refresh_token).await, Err(AppError::Unauthorized)));
}

fn token_service() -> TokenService {
    TokenService::with_ttl_reader(
        TokenSettings {
            secret: "test-secret-with-enough-entropy".into(),
        },
        Arc::new(TestTokenSettingsReader),
        Arc::new(MemoryOnlineSessionStore::default()),
    )
}

struct TestTokenSettingsReader;

#[async_trait]
impl TokenSettingsReader for TestTokenSettingsReader {
    async fn token_ttl_config(&self) -> AppResult<TokenTtlConfig> {
        Ok(TokenTtlConfig {
            access_token_ttl_seconds: TEST_ACCESS_TTL_SECONDS,
            refresh_token_ttl_seconds: TEST_REFRESH_TTL_SECONDS,
        })
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
