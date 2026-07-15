use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    api::{TokenService, TokenSettings, TokenSettingsReader, TokenTtlConfig},
    application::AppResult,
    test_support::MemoryOnlineSessionStore,
};

const TEST_SECRET: &str = "test-secret-with-enough-entropy";
const ACCESS_TTL_SECONDS: u64 = 900;
const REFRESH_TTL_SECONDS: u64 = 604800;

pub(super) fn token_service(sessions: Arc<MemoryOnlineSessionStore>) -> TokenService {
    TokenService::with_ttl_reader(TokenSettings { secret: TEST_SECRET.into() }, Arc::new(TestTokenSettingsReader), sessions)
}

struct TestTokenSettingsReader;

#[async_trait]
impl TokenSettingsReader for TestTokenSettingsReader {
    async fn token_ttl_config(&self) -> AppResult<TokenTtlConfig> {
        Ok(TokenTtlConfig {
            access_token_ttl_seconds: ACCESS_TTL_SECONDS,
            refresh_token_ttl_seconds: REFRESH_TTL_SECONDS,
        })
    }
}
