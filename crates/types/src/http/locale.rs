use axum::{
    extract::Request,
    http::{HeaderValue, header::ACCEPT_LANGUAGE},
    middleware::Next,
    response::Response,
};
use kernel::error::LocalizedError;
use rust_i18n::t;

use crate::http::ApiErrorResponse;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Locale {
    #[default]
    ZhCn,
    En,
    ZhTw,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApiErrorKind {
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    InvalidInput,
    Infrastructure,
    UnsupportedMediaType,
    InvalidJson,
    InvalidBody,
}

impl Locale {
    pub fn from_header(value: &str) -> Self {
        value.split(',').filter_map(header_language_range).find_map(Self::from_tag).unwrap_or_default()
    }

    pub fn from_header_value(value: Option<&HeaderValue>) -> Self {
        value.and_then(|value| value.to_str().ok()).map(Self::from_header).unwrap_or_default()
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ZhCn => "zh-CN",
            Self::En => "en",
            Self::ZhTw => "zh-TW",
        }
    }

    fn from_tag(value: &str) -> Option<Self> {
        let normalized = value.trim().to_ascii_lowercase().replace('_', "-");
        if normalized.is_empty() || normalized == "*" {
            return None;
        }
        if normalized == "cn" || normalized == "zh" || normalized.starts_with("zh-cn") || normalized.starts_with("zh-hans") {
            return Some(Self::ZhCn);
        }
        if normalized == "tw"
            || normalized.starts_with("zh-tw")
            || normalized.starts_with("zh-hk")
            || normalized.starts_with("zh-mo")
            || normalized.starts_with("zh-hant")
        {
            return Some(Self::ZhTw);
        }
        if normalized == "en" || normalized.starts_with("en-") {
            return Some(Self::En);
        }
        None
    }
}

impl ApiErrorKind {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::NotFound => "not_found",
            Self::Conflict => "conflict",
            Self::InvalidInput => "invalid_input",
            Self::Infrastructure => "infrastructure_error",
            Self::UnsupportedMediaType => "unsupported_media_type",
            Self::InvalidJson => "invalid_json",
            Self::InvalidBody => "invalid_body",
        }
    }

    const fn message_key(self) -> &'static str {
        match self {
            Self::Unauthorized => "errors.common.unauthorized",
            Self::Forbidden => "errors.common.forbidden",
            Self::NotFound => "errors.common.not_found",
            Self::Conflict => "errors.common.conflict",
            Self::InvalidInput => "errors.common.invalid_input",
            Self::Infrastructure => "errors.common.infrastructure_error",
            Self::UnsupportedMediaType => "errors.common.unsupported_media_type",
            Self::InvalidJson => "errors.common.invalid_json",
            Self::InvalidBody => "errors.common.invalid_body",
        }
    }
}

tokio::task_local! {
    static REQUEST_LOCALE: Locale;
}

pub async fn locale_middleware(mut request: Request, next: Next) -> Response {
    let locale = Locale::from_header_value(request.headers().get(ACCEPT_LANGUAGE));
    request.extensions_mut().insert(locale);
    REQUEST_LOCALE.scope(locale, next.run(request)).await
}

pub fn current_locale() -> Locale {
    REQUEST_LOCALE.try_with(|locale| *locale).unwrap_or_default()
}

pub fn localized_error_response(locale: Locale, kind: ApiErrorKind, details: Option<&LocalizedError>) -> ApiErrorResponse {
    let message = translate_error(locale, kind.message_key());
    match details {
        Some(details) => ApiErrorResponse::with_details(kind.code(), message, translate_localized_error(locale, details)),
        None => ApiErrorResponse::new(kind.code(), message),
    }
}

pub fn translate_error(locale: Locale, key: &str) -> String {
    translate_message(locale, key)
}

pub fn translate_message(locale: Locale, key: &str) -> String {
    t!(key, locale = locale.as_str()).into_owned()
}

pub fn translate_message_with_params(locale: Locale, key: &str, params: &[(&str, String)]) -> String {
    let template = translate_message(locale, key);
    let patterns = params.iter().map(|param| param.0).collect::<Vec<_>>();
    let values = params.iter().map(|param| param.1.clone()).collect::<Vec<_>>();
    rust_i18n::replace_patterns(&template, &patterns, &values)
}

fn translate_localized_error(locale: Locale, error: &LocalizedError) -> String {
    let params = error.params().iter().map(|param| (param.key(), param.value().to_owned())).collect::<Vec<_>>();
    translate_message_with_params(locale, error.key(), &params)
}

fn header_language_range(part: &str) -> Option<&str> {
    let range = part.split(';').next()?.trim();
    if range.is_empty() { None } else { Some(range) }
}
