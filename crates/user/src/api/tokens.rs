use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use kernel::error::LocalizedError;
use serde::{Deserialize, Serialize};

use crate::{
    application::{AppError, AppResult},
    domain::UserId,
};

const JWT_EXPIRATION_OVERFLOW_ERROR: &str = "infra.jwt.expiration_overflow";

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct TokenTtlConfig {
    pub access_token_ttl_seconds: u64,
    pub refresh_token_ttl_seconds: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenSettings {
    pub secret: String,
}

#[derive(Clone)]
pub struct TokenService {
    settings: TokenSettings,
    ttl_reader: Arc<dyn TokenSettingsReader>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum TokenKind {
    Access,
    Refresh,
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: String,
    exp: u64,
    iat: u64,
    jti: String,
    token_type: TokenKind,
}

#[async_trait]
pub trait TokenSettingsReader: Send + Sync + 'static {
    async fn token_ttl_config(&self) -> AppResult<TokenTtlConfig>;
}

#[derive(Clone, Copy)]
pub struct StaticTokenSettingsReader;

#[async_trait]
impl TokenSettingsReader for StaticTokenSettingsReader {
    async fn token_ttl_config(&self) -> AppResult<TokenTtlConfig> {
        Ok(TokenTtlConfig {
            access_token_ttl_seconds: 900,
            refresh_token_ttl_seconds: 604800,
        })
    }
}

impl TokenService {
    pub fn new(settings: TokenSettings) -> Self {
        Self::with_ttl_reader(settings, Arc::new(StaticTokenSettingsReader))
    }

    pub fn with_ttl_reader(settings: TokenSettings, ttl_reader: Arc<dyn TokenSettingsReader>) -> Self {
        Self { settings, ttl_reader }
    }

    pub async fn issue_pair(&self, user_id: UserId) -> AppResult<TokenPair> {
        let ttl = self.ttl_reader.token_ttl_config().await?;
        Ok(TokenPair {
            access_token: self.issue_token(user_id.clone(), TokenKind::Access, ttl.access_token_ttl_seconds)?,
            refresh_token: self.issue_token(user_id, TokenKind::Refresh, ttl.refresh_token_ttl_seconds)?,
        })
    }

    pub async fn refresh(&self, refresh_token: &str) -> AppResult<(UserId, TokenPair)> {
        let user_id = self.validate_token(refresh_token, TokenKind::Refresh)?;
        Ok((user_id.clone(), self.issue_pair(user_id).await?))
    }

    pub fn validate_access(&self, access_token: &str) -> AppResult<UserId> {
        self.validate_token(access_token, TokenKind::Access)
    }

    fn issue_token(&self, user_id: UserId, token_type: TokenKind, ttl_seconds: u64) -> AppResult<String> {
        let now = system_time()?;
        let iat = now.as_secs();
        let exp = iat
            .checked_add(ttl_seconds)
            .ok_or_else(|| AppError::Infrastructure(JWT_EXPIRATION_OVERFLOW_ERROR.into()))?;
        let claims = Claims {
            sub: user_id.0,
            exp,
            iat,
            jti: now.as_nanos().to_string(),
            token_type,
        };

        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.settings.secret.as_bytes()),
        )
        .map_err(jwt_encode_error)
    }

    fn validate_token(&self, token: &str, expected_type: TokenKind) -> AppResult<UserId> {
        let claims = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.settings.secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(jwt_decode_error)?
        .claims;

        if claims.token_type != expected_type {
            return Err(AppError::Unauthorized);
        }

        parse_user_id(&claims.sub)
    }
}

impl TokenTtlConfig {
    pub fn validate(&self) -> AppResult<()> {
        if self.access_token_ttl_seconds == 0 || self.refresh_token_ttl_seconds == 0 {
            return Err(AppError::InvalidInput(
                LocalizedError::new("errors.user.invalid_system_config").with_param("key", "sys.auth.tokenConfig"),
            ));
        }
        Ok(())
    }
}

pub fn parse_token_ttl_config(value: &str) -> AppResult<TokenTtlConfig> {
    let parsed = serde_json::from_str::<TokenTtlConfig>(value)
        .map_err(|_| AppError::InvalidInput(LocalizedError::new("errors.user.invalid_system_config").with_param("key", "sys.auth.tokenConfig")))?;
    parsed.validate()?;
    Ok(parsed)
}

fn parse_user_id(subject: &str) -> AppResult<UserId> {
    if subject.trim().is_empty() {
        return Err(AppError::Unauthorized);
    }
    Ok(UserId(subject.into()))
}

fn system_time() -> AppResult<Duration> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| AppError::Infrastructure(format!("system time error: {error}")))
}

fn jwt_encode_error(error: jsonwebtoken::errors::Error) -> AppError {
    AppError::Infrastructure(format!("jwt error: {error}"))
}

fn jwt_decode_error(_: jsonwebtoken::errors::Error) -> AppError {
    AppError::Unauthorized
}

#[cfg(test)]
mod tests {
    use super::{TokenService, TokenSettings};
    use crate::{application::AppError, domain::UserId};

    #[tokio::test]
    async fn refresh_rejects_access_token() {
        let service = token_service();
        let tokens = service.issue_pair(user_id()).await.unwrap();

        let result = service.refresh(&tokens.access_token).await;

        assert!(matches!(result, Err(AppError::Unauthorized)));
    }

    #[tokio::test]
    async fn refresh_accepts_refresh_token_and_issues_access_token() {
        let service = token_service();
        let tokens = service.issue_pair(user_id()).await.unwrap();

        let (user_id, refreshed) = service.refresh(&tokens.refresh_token).await.unwrap();

        assert_eq!(user_id, self::user_id());
        assert_eq!(service.validate_access(&refreshed.access_token).unwrap(), self::user_id());
    }

    #[tokio::test]
    async fn validate_access_rejects_refresh_token() {
        let service = token_service();
        let tokens = service.issue_pair(user_id()).await.unwrap();

        let result = service.validate_access(&tokens.refresh_token);

        assert!(matches!(result, Err(AppError::Unauthorized)));
    }

    fn token_service() -> TokenService {
        TokenService::new(TokenSettings {
            secret: "test-secret-with-enough-entropy".into(),
        })
    }

    fn user_id() -> UserId {
        UserId("018f0000-0000-7000-8000-000000000001".into())
    }
}
