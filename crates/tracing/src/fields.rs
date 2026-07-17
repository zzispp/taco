use std::{fmt::Display, time::Duration};

/// Duration wrapper that prints milliseconds.
pub struct DurationMs(pub Duration);

impl Display for DurationMs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}ms", self.0.as_millis())
    }
}

fn field_pairs(fields: &[(&str, &dyn Display)]) -> Option<String> {
    if fields.is_empty() {
        return None;
    }

    Some(
        fields
            .iter()
            .map(|(key, value)| {
                let value = safe_field_value(key, *value);
                format!("{key}={value}")
            })
            .collect::<Vec<_>>()
            .join(" "),
    )
}

pub fn safe_field_value(key: &str, value: &dyn Display) -> String {
    if kernel::redaction::is_sensitive_key(key) {
        return kernel::redaction::REDACTED.into();
    }
    match kernel::redaction::normalize_key(key).as_str() {
        "body" => "[body omitted]".into(),
        "url" | "uri" | "endpoint" => kernel::redaction::sanitize_url(&value.to_string()).unwrap_or_else(|| "[URL omitted]".into()),
        _ => value.to_string(),
    }
}

pub fn safe_error_value<T: Display + ?Sized>(value: &T) -> String {
    kernel::redaction::redact_sensitive_text(&value.to_string())
}

pub fn info_with_fields_impl(message: &str, fields: &[(&str, &dyn Display)]) {
    match field_pairs(fields) {
        Some(field_pairs) => tracing::info!(target: module_path!(), __taco_system_log = true, message = %format!("{message} {field_pairs}")),
        None => tracing::info!(target: module_path!(), __taco_system_log = true, message = %message),
    }
}

pub fn warn_with_fields_impl(message: &str, fields: &[(&str, &dyn Display)]) {
    match field_pairs(fields) {
        Some(field_pairs) => tracing::warn!(target: module_path!(), __taco_system_log = true, message = %format!("{message} {field_pairs}")),
        None => tracing::warn!(target: module_path!(), __taco_system_log = true, message = %message),
    }
}

pub fn error_with_fields_impl<E: std::error::Error + ?Sized>(message: &str, error: &E, fields: &[(&str, &dyn Display)]) {
    let error = safe_error_value(error);
    match field_pairs(fields) {
        Some(field_pairs) => tracing::error!(target: module_path!(), __taco_system_log = true, message = %format!("{message}: {field_pairs} {error}")),
        None => tracing::error!(target: module_path!(), __taco_system_log = true, message = %format!("{message}: {error}")),
    }
}

pub fn error<E: std::error::Error + ?Sized>(message: &str, error: &E) {
    tracing::error!(target: module_path!(), __taco_system_log = true, message = %format!("{message}: {}", safe_error_value(error)));
}
