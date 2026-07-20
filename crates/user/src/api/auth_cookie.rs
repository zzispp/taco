use axum::http::{HeaderMap, HeaderName, HeaderValue, header};
use kernel::error::LocalizedError;

use crate::application::{AppError, AppResult};

const REFRESH_COOKIE_NAME: &str = "refresh_token";
const REFRESH_COOKIE_PATH: &str = "/api/auth";
const COOKIE_SEPARATOR: char = ';';
const X_FORWARDED_HOST: &str = "x-forwarded-host";
const X_FORWARDED_PROTO: &str = "x-forwarded-proto";
const HTTPS_SCHEME: &str = "https";
const HTTP_SCHEME: &str = "http";

pub(crate) fn require_same_origin(headers: &HeaderMap) -> AppResult<()> {
    let origin = header_text(headers, &header::ORIGIN).ok_or_else(origin_error)?;
    let host = forwarded_host(headers)
        .or_else(|| header_text(headers, &header::HOST))
        .ok_or_else(origin_error)?;
    let expected_origin = format!("{}://{host}", external_scheme(headers));
    (origin == expected_origin).then_some(()).ok_or_else(origin_error)
}

pub(crate) fn refresh_token_from_headers(headers: &HeaderMap) -> AppResult<&str> {
    headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(find_refresh_cookie)
        .filter(|value| !value.is_empty())
        .ok_or(AppError::Unauthorized)
}

pub(crate) fn issued_refresh_cookie(headers: &HeaderMap, token: &str, max_age_seconds: u64) -> AppResult<HeaderValue> {
    refresh_cookie_header(headers, token, max_age_seconds)
}

pub(crate) fn cleared_refresh_cookie(headers: &HeaderMap) -> AppResult<HeaderValue> {
    refresh_cookie_header(headers, "", 0)
}

fn refresh_cookie_header(headers: &HeaderMap, token: &str, max_age_seconds: u64) -> AppResult<HeaderValue> {
    let mut attributes = vec![
        format!("{REFRESH_COOKIE_NAME}={token}"),
        format!("Path={REFRESH_COOKIE_PATH}"),
        "HttpOnly".into(),
        "SameSite=Strict".into(),
    ];
    if forwarded_https(headers) {
        attributes.push("Secure".into());
    }
    attributes.push(format!("Max-Age={max_age_seconds}"));
    HeaderValue::from_str(&attributes.join("; ")).map_err(|error| AppError::Infrastructure(format!("refresh cookie header error: {error}")))
}

fn forwarded_host(headers: &HeaderMap) -> Option<&str> {
    first_forwarded_value(headers, X_FORWARDED_HOST)
}

fn external_scheme(headers: &HeaderMap) -> &'static str {
    if forwarded_https(headers) { HTTPS_SCHEME } else { HTTP_SCHEME }
}

fn forwarded_https(headers: &HeaderMap) -> bool {
    first_forwarded_value(headers, X_FORWARDED_PROTO).is_some_and(|value| value.eq_ignore_ascii_case(HTTPS_SCHEME))
}

fn first_forwarded_value<'a>(headers: &'a HeaderMap, name: &'static str) -> Option<&'a str> {
    let name = HeaderName::from_static(name);
    headers
        .get_all(name)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .map(str::trim)
        .find(|value| !value.is_empty())
}

fn header_text<'a>(headers: &'a HeaderMap, name: &HeaderName) -> Option<&'a str> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
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

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderName, HeaderValue, header};

    use super::{cleared_refresh_cookie, issued_refresh_cookie, require_same_origin};

    #[test]
    fn same_origin_prefers_forwarded_host_and_https_scheme() {
        let headers = headers(&[
            (header::ORIGIN.as_str(), "https://console.example.test"),
            (header::HOST.as_str(), "127.0.0.1:3000"),
            ("x-forwarded-host", "console.example.test"),
            ("x-forwarded-proto", "https"),
        ]);

        assert!(require_same_origin(&headers).is_ok());
    }

    #[test]
    fn same_origin_falls_back_to_direct_http_host() {
        let headers = headers(&[(header::ORIGIN.as_str(), "http://localhost:8082"), (header::HOST.as_str(), "localhost:8082")]);

        assert!(require_same_origin(&headers).is_ok());
    }

    #[test]
    fn same_origin_rejects_mismatched_forwarded_scheme_or_host() {
        let scheme_mismatch = headers(&[
            (header::ORIGIN.as_str(), "https://console.example.test"),
            ("x-forwarded-host", "console.example.test"),
        ]);
        let host_mismatch = headers(&[
            (header::ORIGIN.as_str(), "https://console.example.test"),
            ("x-forwarded-host", "other.example.test"),
            ("x-forwarded-proto", "https"),
        ]);

        assert!(require_same_origin(&scheme_mismatch).is_err());
        assert!(require_same_origin(&host_mismatch).is_err());
    }

    #[test]
    fn refresh_cookie_is_secure_only_for_forwarded_https() {
        let https_headers = headers(&[("x-forwarded-proto", "HTTPS, http")]);
        let forwarded_http_headers = headers(&[("x-forwarded-proto", "http")]);
        let http_headers = HeaderMap::new();

        let secure_cookie = issued_refresh_cookie(&https_headers, "token", 60).unwrap();
        let forwarded_http_cookie = issued_refresh_cookie(&forwarded_http_headers, "token", 60).unwrap();
        let direct_cookie = issued_refresh_cookie(&http_headers, "token", 60).unwrap();
        let cleared_cookie = cleared_refresh_cookie(&https_headers).unwrap();

        assert_eq!(
            secure_cookie,
            HeaderValue::from_static("refresh_token=token; Path=/api/auth; HttpOnly; SameSite=Strict; Secure; Max-Age=60")
        );
        assert_eq!(
            forwarded_http_cookie,
            HeaderValue::from_static("refresh_token=token; Path=/api/auth; HttpOnly; SameSite=Strict; Max-Age=60")
        );
        assert_eq!(
            direct_cookie,
            HeaderValue::from_static("refresh_token=token; Path=/api/auth; HttpOnly; SameSite=Strict; Max-Age=60")
        );
        assert_eq!(
            cleared_cookie,
            HeaderValue::from_static("refresh_token=; Path=/api/auth; HttpOnly; SameSite=Strict; Secure; Max-Age=0")
        );
    }

    fn headers(values: &[(&str, &str)]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (name, value) in values {
            headers.insert(HeaderName::from_bytes(name.as_bytes()).unwrap(), HeaderValue::from_str(value).unwrap());
        }
        headers
    }
}
