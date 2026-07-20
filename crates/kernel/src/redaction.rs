use std::sync::LazyLock;

use regex::Regex;
use serde_json::{Map, Value};
use url::{
    Url,
    form_urlencoded::{Serializer, parse},
};

pub const REDACTED: &str = "***";

const SENSITIVE_KEYWORDS: [&str; 10] = [
    "password",
    "passwd",
    "pwd",
    "token",
    "secret",
    "captcha",
    "verificationcode",
    "verifycode",
    "authcode",
    "authorization",
];
const SENSITIVE_SUFFIXES: [&str; 3] = ["cookie", "apikey", "credential"];

static URL_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?:https?|postgres(?:ql)?|redis|rediss)://[^\s\"'<>]+"#).expect("URL redaction pattern must compile"));
static SENSITIVE_VALUE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?ix)
        (?P<label>[\"']?[a-z0-9_-]*(?:password|passwd|pwd|token|secret|captcha|verificationcode|verifycode|authcode|authorization|cookie|apikey|credential)[a-z0-9_-]*[\"']?)
        (?P<separator>\s*(?:=|:)\s*)
        (?P<value>\"(?:[^\"\\]|\\.)*\"|'(?:[^'\\]|\\.)*'|[^,\s;\}\]&]+)
        "#,
    )
    .expect("sensitive text redaction pattern must compile")
});

pub fn redact_json(value: &mut Value) {
    match value {
        Value::Object(object) => redact_object(object),
        Value::Array(values) => values.iter_mut().for_each(redact_json),
        _ => {}
    }
}

/// Redacts credentials embedded in unstructured diagnostics before they reach a sink.
pub fn redact_sensitive_text(value: &str) -> String {
    let urls_redacted = URL_PATTERN.replace_all(value, |captures: &regex::Captures<'_>| {
        sanitize_url(&captures[0]).unwrap_or_else(|| "[URL omitted]".into())
    });
    SENSITIVE_VALUE_PATTERN
        .replace_all(&urls_redacted, |captures: &regex::Captures<'_>| {
            format!("{}{}{}", &captures["label"], &captures["separator"], REDACTED)
        })
        .into_owned()
}

pub fn sanitize_url(value: &str) -> Option<String> {
    let mut url = Url::parse(value).ok()?;
    url.set_username("").ok()?;
    url.set_password(None).ok()?;
    url.set_fragment(None);
    let query = url.query().map(redact_query);
    url.set_query(query.as_deref());
    Some(url.into())
}

pub fn redact_sensitive_field(key: &str, value: &mut Value) -> bool {
    if !is_sensitive_key(key) {
        return false;
    }
    *value = Value::String(REDACTED.into());
    true
}

pub fn is_sensitive_key(key: &str) -> bool {
    let key = normalize_key(key);
    SENSITIVE_KEYWORDS.iter().any(|keyword| key.contains(keyword)) || SENSITIVE_SUFFIXES.iter().any(|suffix| key.contains(suffix))
}

fn redact_object(object: &mut Map<String, Value>) {
    for (key, value) in object {
        if !redact_sensitive_field(key, value) {
            redact_json(value);
        }
    }
}

pub fn normalize_key(key: &str) -> String {
    key.chars().filter(|value| value.is_ascii_alphanumeric()).flat_map(char::to_lowercase).collect()
}

fn redact_query(value: &str) -> String {
    let mut serializer = Serializer::new(String::new());
    for (key, value) in parse(value.as_bytes()) {
        let value = if is_sensitive_key(&key) { REDACTED.into() } else { value.into_owned() };
        serializer.append_pair(&key, &value);
    }
    serializer.finish()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{REDACTED, is_sensitive_key, redact_json, redact_sensitive_text};

    #[test]
    fn redacts_nested_sensitive_fields() {
        let mut value = json!({
            "password": "one",
            "nested": [{"access_token": "two"}]
        });

        redact_json(&mut value);

        assert_eq!(
            value,
            json!({
                "password": REDACTED,
                "nested": [{"access_token": REDACTED}]
            })
        );
    }

    #[test]
    fn detects_normalized_sensitive_names() {
        for key in ["Authorization", "X-API-Key", "refresh_token", "verify-code", "session_cookie"] {
            assert!(is_sensitive_key(key), "{key} must be sensitive");
        }
        assert!(!is_sensitive_key("request_id"));
    }

    #[test]
    fn redacts_sensitive_values_from_unstructured_text() {
        let redacted =
            redact_sensitive_text("failed https://login-user:credential-pass@example.com/run?token=query-token-value#url-fragment password=top-secret-value");

        assert_eq!(redacted, "failed https://example.com/run?token=*** password=***");
    }

    #[test]
    fn redacts_database_and_redis_connection_urls() {
        let redacted = redact_sensitive_text(
            "postgres://db-user:db-password@postgres.internal/taco?token=db-token redis://cache-user:cache-password@redis.internal/0?password=query-password",
        );

        for secret in ["db-user", "db-password", "db-token", "cache-user", "cache-password", "query-password"] {
            assert!(!redacted.contains(secret), "redaction leaked {secret}");
        }
        assert!(redacted.contains("postgres://postgres.internal/taco?token=***"));
        assert!(redacted.contains("redis://redis.internal/0?password=***"));
    }
}
