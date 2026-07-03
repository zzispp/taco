use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::{
    application::{AppError, AppResult},
    domain::UserId,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenSettings {
    pub secret: String,
    pub access_token_ttl_seconds: u64,
    pub refresh_token_ttl_seconds: u64,
}

#[derive(Clone)]
pub struct TokenService {
    settings: TokenSettings,
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

impl TokenService {
    pub fn new(settings: TokenSettings) -> Self {
        Self { settings }
    }

    pub fn issue_pair(&self, user_id: UserId) -> AppResult<TokenPair> {
        Ok(TokenPair {
            access_token: self.issue_token(user_id.clone(), TokenKind::Access, self.settings.access_token_ttl_seconds)?,
            refresh_token: self.issue_token(user_id, TokenKind::Refresh, self.settings.refresh_token_ttl_seconds)?,
        })
    }

    pub fn refresh(&self, refresh_token: &str) -> AppResult<(UserId, TokenPair)> {
        let user_id = self.validate_token(refresh_token, TokenKind::Refresh)?;
        Ok((user_id.clone(), self.issue_pair(user_id)?))
    }

    pub fn validate_access(&self, access_token: &str) -> AppResult<UserId> {
        self.validate_token(access_token, TokenKind::Access)
    }

    fn issue_token(&self, user_id: UserId, token_type: TokenKind, ttl_seconds: u64) -> AppResult<String> {
        let now = system_time()?;
        let iat = now.as_secs();
        let exp = iat
            .checked_add(ttl_seconds)
            .ok_or_else(|| AppError::Infrastructure("jwt expiration overflow".into()))?;
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

    #[test]
    fn refresh_rejects_access_token() {
        let service = token_service();
        let tokens = service.issue_pair(user_id()).unwrap();

        let result = service.refresh(&tokens.access_token);

        assert!(matches!(result, Err(AppError::Unauthorized)));
    }

    #[test]
    fn refresh_accepts_refresh_token_and_issues_access_token() {
        let service = token_service();
        let tokens = service.issue_pair(user_id()).unwrap();

        let (user_id, refreshed) = service.refresh(&tokens.refresh_token).unwrap();

        assert_eq!(user_id, self::user_id());
        assert_eq!(service.validate_access(&refreshed.access_token).unwrap(), self::user_id());
    }

    #[test]
    fn validate_access_rejects_refresh_token() {
        let service = token_service();
        let tokens = service.issue_pair(user_id()).unwrap();

        let result = service.validate_access(&tokens.refresh_token);

        assert!(matches!(result, Err(AppError::Unauthorized)));
    }

    fn token_service() -> TokenService {
        TokenService::new(TokenSettings {
            secret: "test-secret-with-enough-entropy".into(),
            access_token_ttl_seconds: 900,
            refresh_token_ttl_seconds: 604800,
        })
    }

    fn user_id() -> UserId {
        UserId("018f0000-0000-7000-8000-000000000007".into())
    }
}
