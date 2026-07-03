use crate::{CorsSettings, HttpSettings, Settings, SettingsError, TracingFileSettings};
use std::str::FromStr;
use tracing::level_filters::LevelFilter;

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

    pub fn validated_cors(&self) -> Result<ValidatedCorsSettings, SettingsError> {
        self.cors.validate()
    }

    pub(crate) fn validate(&self) -> Result<(), SettingsError> {
        self.http_config()?;
        self.validated_cors()?;
        self.tracing_config()?;
        Ok(())
    }
}

impl CorsSettings {
    fn validate(&self) -> Result<ValidatedCorsSettings, SettingsError> {
        let allowed_origins = validate_string_list("cors.allowed_origins", &self.allowed_origins)?;
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

fn validate_string_list(key: &'static str, values: &[String]) -> Result<ValidatedCorsList, SettingsError> {
    validate_list(key, values, false)
}

fn validate_method_list(key: &'static str, values: &[String]) -> Result<ValidatedCorsList, SettingsError> {
    let list = validate_list(key, values, true)?;
    if let ValidatedCorsList::Values(values) = &list {
        for value in values {
            http::Method::from_bytes(value.as_bytes()).map_err(|_| SettingsError::InvalidHttpMethod {
                key,
                value: value.clone(),
            })?;
        }
    }
    Ok(list)
}

fn validate_header_list(key: &'static str, values: &[String]) -> Result<ValidatedCorsList, SettingsError> {
    let list = validate_list(key, values, false)?;
    if let ValidatedCorsList::Values(values) = &list {
        for value in values {
            http::header::HeaderName::from_bytes(value.as_bytes()).map_err(|_| SettingsError::InvalidHttpHeaderName {
                key,
                value: value.clone(),
            })?;
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
        let normalized_value = if uppercase {
            trimmed.to_ascii_uppercase()
        } else {
            trimmed.to_owned()
        };
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
