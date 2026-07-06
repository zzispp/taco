use serde_json::Value;

pub(crate) fn provider_config<'a>(value: &'a Value, provider: &str) -> &'a Value {
    value.get(provider).filter(|item| item.is_object()).unwrap_or(value)
}
