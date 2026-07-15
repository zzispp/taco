use axum::http::{HeaderMap, HeaderValue, header};
use kernel::error::LocalizedError;

use crate::application::{AppError, AppResult};

const REFRESH_COOKIE_NAME: &str = "refresh_token";
const COOKIE_SEPARATOR: char = ';';

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthHttpConfig {
    pub refresh_cookie: RefreshCookieConfig,
    pub trusted_origins: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefreshCookieConfig {
    pub secure: bool,
    pub domain: Option<String>,
    pub path: String,
}

impl AuthHttpConfig {
    pub(crate) fn require_trusted_origin(&self, headers: &HeaderMap) -> AppResult<()> {
        let origin = headers.get(header::ORIGIN).and_then(|value| value.to_str().ok()).ok_or_else(origin_error)?;
        if !self.trusted_origins.iter().any(|trusted| trusted == origin) {
            return Err(origin_error());
        }
        Ok(())
    }

    pub(crate) fn refresh_token<'a>(&self, headers: &'a HeaderMap) -> AppResult<&'a str> {
        headers
            .get(header::COOKIE)
            .and_then(|value| value.to_str().ok())
            .and_then(find_refresh_cookie)
            .filter(|value| !value.is_empty())
            .ok_or(AppError::Unauthorized)
    }

    pub(crate) fn issued_cookie(&self, token: &str, max_age_seconds: u64) -> AppResult<HeaderValue> {
        self.cookie_header(token, Some(max_age_seconds))
    }

    pub(crate) fn cleared_cookie(&self) -> AppResult<HeaderValue> {
        self.cookie_header("", Some(0))
    }

    fn cookie_header(&self, token: &str, max_age_seconds: Option<u64>) -> AppResult<HeaderValue> {
        let mut attributes = vec![
            format!("{REFRESH_COOKIE_NAME}={token}"),
            format!("Path={}", self.refresh_cookie.path),
            "HttpOnly".into(),
            "SameSite=Strict".into(),
        ];
        if self.refresh_cookie.secure {
            attributes.push("Secure".into());
        }
        if let Some(domain) = &self.refresh_cookie.domain {
            attributes.push(format!("Domain={domain}"));
        }
        if let Some(max_age) = max_age_seconds {
            attributes.push(format!("Max-Age={max_age}"));
        }
        HeaderValue::from_str(&attributes.join("; ")).map_err(|error| AppError::Infrastructure(format!("refresh cookie header error: {error}")))
    }
}

fn find_refresh_cookie(value: &str) -> Option<&str> {
    value.split(COOKIE_SEPARATOR).find_map(|cookie| {
        let (name, value) = cookie.trim().split_once('=')?;
        (name == REFRESH_COOKIE_NAME).then_some(value)
    })
}

fn origin_error() -> AppError {
    AppError::Forbidden(LocalizedError::new("errors.common.forbidden"))
}
