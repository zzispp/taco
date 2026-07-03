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

    Some(fields.iter().map(|(key, value)| format!("{key}={value}")).collect::<Vec<_>>().join(" "))
}

pub fn info_with_fields_impl(message: &str, fields: &[(&str, &dyn Display)]) {
    match field_pairs(fields) {
        Some(field_pairs) => tracing::info!("{message} {field_pairs}"),
        None => tracing::info!("{message}"),
    }
}

pub fn warn_with_fields_impl(message: &str, fields: &[(&str, &dyn Display)]) {
    match field_pairs(fields) {
        Some(field_pairs) => tracing::warn!("{message} {field_pairs}"),
        None => tracing::warn!("{message}"),
    }
}

pub fn error_with_fields_impl<E: std::error::Error + ?Sized>(message: &str, error: &E, fields: &[(&str, &dyn Display)]) {
    match field_pairs(fields) {
        Some(field_pairs) => tracing::error!("{message}: {field_pairs} {error}"),
        None => tracing::error!("{message}: {error}"),
    }
}

pub fn error<E: std::error::Error + ?Sized>(message: &str, error: &E) {
    tracing::error!("{message}: {error}");
}
