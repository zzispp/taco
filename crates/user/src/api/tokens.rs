use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use constants::system_config::TOKEN_CONFIG_KEY;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use kernel::error::LocalizedError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    application::{AppError, AppResult, OnlineSession, OnlineSessionPageRequest, OnlineSessionStore},
    domain::UserId,
};

const JWT_EXPIRATION_OVERFLOW_ERROR: &str = "infra.jwt.expiration_overflow";
const MILLIS_PER_SECOND: u64 = 1_000;

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
    sessions: Arc<dyn OnlineSessionStore>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub refresh_token_ttl_seconds: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenIssueInput {
    pub user_id: UserId,
    pub dept_name: Option<String>,
    pub user_name: String,
    pub ipaddr: String,
    pub login_location: String,
    pub browser: String,
    pub os: String,
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

struct ValidatedToken {
    user_id: UserId,
    token_id: String,
}

struct TokenPayload<'a> {
    user_id: &'a UserId,
    token_id: &'a str,
    token_type: TokenKind,
    ttl_seconds: u64,
}

#[async_trait]
pub trait TokenSettingsReader: Send + Sync + 'static {
    async fn token_ttl_config(&self) -> AppResult<TokenTtlConfig>;
}

impl TokenService {
    pub fn with_ttl_reader(settings: TokenSettings, ttl_reader: Arc<dyn TokenSettingsReader>, sessions: Arc<dyn OnlineSessionStore>) -> Self {
        Self {
            settings,
            ttl_reader,
            sessions,
        }
    }

    pub async fn issue_pair(&self, input: TokenIssueInput) -> AppResult<TokenPair> {
        let ttl = self.ttl_reader.token_ttl_config().await?;
        let login_time = system_time_millis()?;
        let expires_at = expiration_millis(login_time, ttl.refresh_token_ttl_seconds)?;
        let session = input.into_session(Uuid::now_v7().to_string(), login_time, expires_at);
        let tokens = self.issue_pair_for_session(&session.user_id, &session.token_id, &ttl)?;
        if let Err(error) = self.sessions.create(&session).await {
            self.compensate_failed_session_save(&session.token_id).await;
            return Err(error);
        }
        Ok(tokens)
    }

    async fn compensate_failed_session_save(&self, token_id: &str) {
        if let Err(error) = self.sessions.delete(token_id).await {
            hook_tracing::error_with_fields!(
                "failed to compensate ambiguous online session save",
                &error,
                token_id = token_id,
                operation = "issue_pair"
            );
        }
    }

    pub async fn refresh(&self, refresh_token: &str) -> AppResult<(UserId, TokenPair)> {
        let token = self.validate_token(refresh_token, TokenKind::Refresh)?;
        let ttl = self.ttl_reader.token_ttl_config().await?;
        let expires_at = expiration_millis(system_time_millis()?, ttl.refresh_token_ttl_seconds)?;
        let session = self
            .sessions
            .renew_active(&token.token_id, &token.user_id, expires_at)
            .await?
            .ok_or(AppError::Unauthorized)?;
        let tokens = self.issue_pair_for_session(&session.user_id, &session.token_id, &ttl)?;
        Ok((session.user_id.clone(), tokens))
    }

    pub async fn validate_access(&self, access_token: &str) -> AppResult<UserId> {
        let token = self.validate_token(access_token, TokenKind::Access)?;
        Ok(self.require_session(&token).await?.user_id)
    }

    pub async fn logout_access(&self, access_token: &str) -> AppResult<OnlineSession> {
        let token = self.validate_token(access_token, TokenKind::Access)?;
        let session = self.require_session(&token).await?;
        self.sessions.delete(&token.token_id).await?;
        Ok(session)
    }

    pub async fn logout_refresh(&self, refresh_token: &str) -> AppResult<OnlineSession> {
        let token = self.validate_token(refresh_token, TokenKind::Refresh)?;
        let session = self.require_session(&token).await?;
        self.sessions.delete(&token.token_id).await?;
        Ok(session)
    }

    pub async fn force_logout(&self, token_id: &str) -> AppResult<()> {
        self.sessions.delete(token_id).await
    }

    pub async fn online_session(&self, token_id: &str) -> AppResult<Option<OnlineSession>> {
        self.sessions.find_active_by_token(token_id).await
    }

    pub async fn online_sessions(&self, request: OnlineSessionPageRequest) -> AppResult<kernel::pagination::CursorPage<OnlineSession>> {
        self.sessions.page_active(request).await
    }

    async fn require_session(&self, token: &ValidatedToken) -> AppResult<OnlineSession> {
        self.sessions.find_active(&token.token_id, &token.user_id).await?.ok_or(AppError::Unauthorized)
    }

    fn issue_pair_for_session(&self, user_id: &UserId, token_id: &str, ttl: &TokenTtlConfig) -> AppResult<TokenPair> {
        Ok(TokenPair {
            access_token: self.issue_token(TokenPayload {
                user_id,
                token_id,
                token_type: TokenKind::Access,
                ttl_seconds: ttl.access_token_ttl_seconds,
            })?,
            refresh_token: self.issue_token(TokenPayload {
                user_id,
                token_id,
                token_type: TokenKind::Refresh,
                ttl_seconds: ttl.refresh_token_ttl_seconds,
            })?,
            refresh_token_ttl_seconds: ttl.refresh_token_ttl_seconds,
        })
    }

    fn issue_token(&self, payload: TokenPayload<'_>) -> AppResult<String> {
        let iat = system_time()?.as_secs();
        let exp = iat
            .checked_add(payload.ttl_seconds)
            .ok_or_else(|| AppError::Infrastructure(JWT_EXPIRATION_OVERFLOW_ERROR.into()))?;
        let claims = Claims {
            sub: payload.user_id.0.clone(),
            exp,
            iat,
            jti: payload.token_id.into(),
            token_type: payload.token_type,
        };
        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.settings.secret.as_bytes()),
        )
        .map_err(jwt_encode_error)
    }

    fn validate_token(&self, token: &str, expected_type: TokenKind) -> AppResult<ValidatedToken> {
        let claims = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.settings.secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(jwt_decode_error)?
        .claims;
        if claims.token_type != expected_type || claims.jti.trim().is_empty() {
            return Err(AppError::Unauthorized);
        }
        Ok(ValidatedToken {
            user_id: parse_user_id(&claims.sub)?,
            token_id: claims.jti,
        })
    }
}

impl TokenIssueInput {
    fn into_session(self, token_id: String, login_time: i64, expires_at: i64) -> OnlineSession {
        OnlineSession {
            token_id,
            user_id: self.user_id,
            dept_id: None,
            dept_name: self.dept_name,
            user_name: self.user_name,
            ipaddr: self.ipaddr,
            login_location: self.login_location,
            browser: self.browser,
            os: self.os,
            login_time,
            expires_at,
        }
    }
}

impl TokenTtlConfig {
    pub fn validate(&self) -> AppResult<()> {
        if self.access_token_ttl_seconds == 0 || self.refresh_token_ttl_seconds == 0 {
            return Err(AppError::InvalidInput(
                LocalizedError::new("errors.user.invalid_system_config").with_param("key", TOKEN_CONFIG_KEY),
            ));
        }
        Ok(())
    }
}

pub fn parse_token_ttl_config(value: &str) -> AppResult<TokenTtlConfig> {
    let parsed = serde_json::from_str::<TokenTtlConfig>(value)
        .map_err(|_| AppError::InvalidInput(LocalizedError::new("errors.user.invalid_system_config").with_param("key", TOKEN_CONFIG_KEY)))?;
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

fn system_time_millis() -> AppResult<i64> {
    i64::try_from(system_time()?.as_millis()).map_err(|error| AppError::Infrastructure(format!("system time overflow: {error}")))
}

fn expiration_millis(start: i64, ttl_seconds: u64) -> AppResult<i64> {
    let ttl_millis = ttl_seconds
        .checked_mul(MILLIS_PER_SECOND)
        .and_then(|value| i64::try_from(value).ok())
        .ok_or_else(|| AppError::Infrastructure(JWT_EXPIRATION_OVERFLOW_ERROR.into()))?;
    start
        .checked_add(ttl_millis)
        .ok_or_else(|| AppError::Infrastructure(JWT_EXPIRATION_OVERFLOW_ERROR.into()))
}

fn jwt_encode_error(error: jsonwebtoken::errors::Error) -> AppError {
    AppError::Infrastructure(format!("jwt error: {error}"))
}

fn jwt_decode_error(_: jsonwebtoken::errors::Error) -> AppError {
    AppError::Unauthorized
}

#[cfg(test)]
mod tests;
