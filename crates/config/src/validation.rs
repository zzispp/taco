use crate::{CorsSettings, HttpSettings, RefreshCookieSettings, Settings, SettingsError, TracingFileSettings};
use std::str::FromStr;
use tracing::level_filters::LevelFilter;
use url::{Host, Url};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedTracingSettings {
    pub log_level: String,
    pub file: TracingFileSettings,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedCorsSettings {
    pub allowed_origins: ValidatedCorsList,
    pub allowed_methods: ValidatedCorsList,
    pub allowed_headers: ValidatedCorsList,
    pub exposed_headers: ValidatedCorsList,
    pub allow_credentials: bool,
    pub max_age_seconds: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidatedCorsList {
    Any,
    Values(Vec<String>),
}

impl Settings {
    pub fn tracing_config(&self) -> Result<ValidatedTracingSettings, SettingsError> {
        let log_level = self.tracing_log_level()?;
        LevelFilter::from_str(&log_level).map_err(|_| SettingsError::InvalidTracingFilter("tracing.log_level"))?;

        if self.tracing.file.enabled {
            crate::loader::required_config_value("tracing.file.directory", &self.tracing.file.directory)?;
            crate::loader::required_config_value("tracing.file.prefix", &self.tracing.file.prefix)?;
        }

        Ok(ValidatedTracingSettings {
            log_level,
            file: self.tracing.file.clone(),
        })
    }

    pub fn http_config(&self) -> Result<HttpSettings, SettingsError> {
        if self.http.request_timeout_ms == 0 {
            return Err(SettingsError::NonPositiveNumber("http.request_timeout_ms"));
        }
        Ok(self.http.clone())
    }

    pub fn scheduler_config(&self) -> Result<crate::SchedulerSettings, SettingsError> {
        if self.scheduler.http_client.request_timeout_ms == 0 {
            return Err(SettingsError::NonPositiveNumber("scheduler.http_client.request_timeout_ms"));
        }
        if self.scheduler.runtime.reconcile_interval_ms == 0 {
            return Err(SettingsError::NonPositiveNumber("scheduler.runtime.reconcile_interval_ms"));
        }
        Ok(self.scheduler.clone())
    }

    pub fn audit_config(&self) -> Result<crate::AuditSettings, SettingsError> {
        positive("audit.outbox.worker_count", self.audit.outbox.worker_count)?;
        positive("audit.outbox.claim_batch_size", self.audit.outbox.claim_batch_size)?;
        positive("audit.outbox.poll_interval_ms", self.audit.outbox.poll_interval_ms)?;
        positive("audit.outbox.lease_duration_ms", self.audit.outbox.lease_duration_ms)?;
        positive("audit.outbox.retry_delay_ms", self.audit.outbox.retry_delay_ms)?;
        positive("audit.outbox.cleanup_interval_ms", self.audit.outbox.cleanup_interval_ms)?;
        positive("audit.outbox.cleanup_batch_size", self.audit.outbox.cleanup_batch_size)?;
        positive("audit.outbox.processed_retention_days", self.audit.outbox.processed_retention_days)?;
        Ok(self.audit.clone())
    }

    pub fn online_session_config(&self) -> Result<crate::OnlineSessionSettings, SettingsError> {
        positive("user.online_sessions.cleanup_interval_ms", self.user.online_sessions.cleanup_interval_ms)?;
        positive("user.online_sessions.cleanup_batch_size", self.user.online_sessions.cleanup_batch_size)?;
        Ok(self.user.online_sessions.clone())
    }

    pub fn client_info_config(&self) -> Result<crate::ClientInfoSettings, SettingsError> {
        positive("client_info.ip_location.request_timeout_ms", self.client_info.ip_location.request_timeout_ms)?;
        Ok(self.client_info.clone())
    }

    pub fn validated_cors(&self) -> Result<ValidatedCorsSettings, SettingsError> {
        self.cors.validate()
    }

    pub fn refresh_cookie_config(&self) -> Result<RefreshCookieSettings, SettingsError> {
        if !self.auth.refresh_cookie.secure {
            return Err(SettingsError::InsecureRefreshCookie);
        }
        let path = crate::loader::required_config_value("auth.refresh_cookie.path", &self.auth.refresh_cookie.path)?;
        if !path.starts_with('/') {
            return Err(SettingsError::InvalidCookiePath("auth.refresh_cookie.path"));
        }
        Ok(RefreshCookieSettings {
            secure: self.auth.refresh_cookie.secure,
            path,
        })
    }

    pub(crate) fn validate(&self) -> Result<(), SettingsError> {
        crate::loader::required_config_value("server.host", &self.server.host)?;
        positive("server.port", self.server.port)?;
        self.database_url()?;
        self.redis_url()?;
        crate::loader::required_config_value("uploads.avatar_directory", &self.uploads.avatar_directory)?;
        self.jwt_secret()?;
        self.http_config()?;
        self.scheduler_config()?;
        self.audit_config()?;
        self.online_session_config()?;
        self.client_info_config()?;
        self.refresh_cookie_config()?;
        self.validated_cors()?;
        self.tracing_config()?;
        Ok(())
    }
}

fn positive(key: &'static str, value: impl PartialEq + From<u8>) -> Result<(), SettingsError> {
    if value == 0_u8.into() {
        return Err(SettingsError::NonPositiveNumber(key));
    }
    Ok(())
}

impl CorsSettings {
    fn validate(&self) -> Result<ValidatedCorsSettings, SettingsError> {
        let allowed_origins = validate_origin_list("cors.allowed_origins", &self.allowed_origins)?;
        let allowed_methods = validate_method_list("cors.allowed_methods", &self.allowed_methods)?;
        let allowed_headers = validate_header_list("cors.allowed_headers", &self.allowed_headers)?;
        let exposed_headers = validate_header_list("cors.exposed_headers", &self.exposed_headers)?;

        if self.allow_credentials {
            reject_wildcard_with_credentials("cors.allowed_origins", &allowed_origins)?;
            reject_wildcard_with_credentials("cors.allowed_methods", &allowed_methods)?;
            reject_wildcard_with_credentials("cors.allowed_headers", &allowed_headers)?;
            reject_wildcard_with_credentials("cors.exposed_headers", &exposed_headers)?;
        }

        Ok(ValidatedCorsSettings {
            allowed_origins,
            allowed_methods,
            allowed_headers,
            exposed_headers,
            allow_credentials: self.allow_credentials,
            max_age_seconds: self.max_age_seconds,
        })
    }
}

fn validate_origin_list(key: &'static str, values: &[String]) -> Result<ValidatedCorsList, SettingsError> {
    let ValidatedCorsList::Values(values) = validate_list(key, values, false)? else {
        return Err(SettingsError::WildcardCorsOrigin(key));
    };
    if values.len() != 1 {
        return Err(SettingsError::ExpectedSingleCorsOrigin(key));
    }
    let origins = values
        .into_iter()
        .map(|value| normalized_http_origin(key, value))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ValidatedCorsList::Values(origins))
}

fn normalized_http_origin(key: &'static str, value: String) -> Result<String, SettingsError> {
    let parsed = Url::parse(&value).map_err(|_| invalid_http_origin(key, &value))?;
    let valid_scheme = matches!(parsed.scheme(), "http" | "https");
    let has_credentials = !parsed.username().is_empty() || parsed.password().is_some();
    let has_resource = parsed.path() != "/" || parsed.query().is_some() || parsed.fragment().is_some();
    if !valid_scheme || parsed.host().is_none() || has_credentials || has_resource {
        return Err(invalid_http_origin(key, &value));
    }
    if parsed.scheme() == "http" && !is_loopback_origin(&parsed) {
        return Err(SettingsError::InsecureHttpOrigin(key));
    }
    Ok(parsed.origin().ascii_serialization())
}

fn is_loopback_origin(origin: &Url) -> bool {
    match origin.host() {
        Some(Host::Domain(host)) => host.eq_ignore_ascii_case("localhost"),
        Some(Host::Ipv4(address)) => address == std::net::Ipv4Addr::LOCALHOST,
        Some(Host::Ipv6(address)) => address == std::net::Ipv6Addr::LOCALHOST,
        None => false,
    }
}

fn invalid_http_origin(key: &'static str, value: &str) -> SettingsError {
    SettingsError::InvalidHttpOrigin { key, value: value.into() }
}

fn validate_method_list(key: &'static str, values: &[String]) -> Result<ValidatedCorsList, SettingsError> {
    let list = validate_list(key, values, true)?;
    if let ValidatedCorsList::Values(values) = &list {
        for value in values {
            http::Method::from_bytes(value.as_bytes()).map_err(|_| SettingsError::InvalidHttpMethod { key, value: value.clone() })?;
        }
    }
    Ok(list)
}

fn validate_header_list(key: &'static str, values: &[String]) -> Result<ValidatedCorsList, SettingsError> {
    let list = validate_list(key, values, false)?;
    if let ValidatedCorsList::Values(values) = &list {
        for value in values {
            http::header::HeaderName::from_bytes(value.as_bytes()).map_err(|_| SettingsError::InvalidHttpHeaderName { key, value: value.clone() })?;
        }
    }
    Ok(list)
}

fn validate_list(key: &'static str, values: &[String], uppercase: bool) -> Result<ValidatedCorsList, SettingsError> {
    if values.is_empty() {
        return Err(SettingsError::EmptyList(key));
    }

    let mut normalized = Vec::with_capacity(values.len());
    let mut has_wildcard = false;
    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(SettingsError::BlankListItem(key));
        }
        if trimmed == "*" {
            has_wildcard = true;
        }
        let normalized_value = if uppercase { trimmed.to_ascii_uppercase() } else { trimmed.to_owned() };
        normalized.push(normalized_value);
    }

    if has_wildcard {
        if normalized.len() > 1 {
            return Err(SettingsError::MixedWildcardList(key));
        }
        return Ok(ValidatedCorsList::Any);
    }

    Ok(ValidatedCorsList::Values(normalized))
}

fn reject_wildcard_with_credentials(key: &'static str, value: &ValidatedCorsList) -> Result<(), SettingsError> {
    if matches!(value, ValidatedCorsList::Any) {
        return Err(SettingsError::WildcardCorsWithCredentials(key));
    }
    Ok(())
}
