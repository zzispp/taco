use serde::Serialize;
use serde_json::{Map, Value};

const MAX_KIND_CHARACTERS: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ExecutionDetail {
    kind: String,
    schema_version: i16,
    payload: Value,
}

impl ExecutionDetail {
    pub fn new(kind: impl Into<String>, schema_version: i16, payload: Map<String, Value>) -> Self {
        let kind = kind.into();
        assert!(Self::kind_is_valid(&kind), "execution detail kind must be non-blank and at most 64 characters");
        assert!(schema_version > 0, "execution detail schema version must be positive");
        assert!(payload_has_no_nul(&payload), "execution detail payload keys and strings must not contain NUL");
        Self {
            kind,
            schema_version,
            payload: Value::Object(payload),
        }
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub const fn schema_version(&self) -> i16 {
        self.schema_version
    }

    pub fn payload(&self) -> &Value {
        &self.payload
    }

    pub fn into_parts(self) -> (String, i16, Value) {
        (self.kind, self.schema_version, self.payload)
    }

    pub(crate) fn kind_is_valid(kind: &str) -> bool {
        !kind.contains('\0') && !kind.trim_matches(|value: char| value.is_ascii_whitespace()).is_empty() && kind.chars().count() <= MAX_KIND_CHARACTERS
    }
}

fn payload_has_no_nul(payload: &Map<String, Value>) -> bool {
    payload.iter().all(|(key, value)| !key.contains('\0') && value_has_no_nul(value))
}

fn value_has_no_nul(value: &Value) -> bool {
    match value {
        Value::String(value) => !value.contains('\0'),
        Value::Array(values) => values.iter().all(value_has_no_nul),
        Value::Object(values) => payload_has_no_nul(values),
        Value::Null | Value::Bool(_) | Value::Number(_) => true,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, json};

    use super::{ExecutionDetail, MAX_KIND_CHARACTERS};

    #[test]
    fn serializes_as_kind_version_and_object_payload() {
        let payload = Map::from_iter([("result".into(), json!("complete"))]);
        let detail = ExecutionDetail::new("test", 2, payload);

        assert_eq!(
            serde_json::to_value(detail).unwrap(),
            json!({"kind": "test", "schema_version": 2, "payload": {"result": "complete"}})
        );
    }

    #[test]
    #[should_panic(expected = "execution detail schema version must be positive")]
    fn rejects_non_positive_schema_version() {
        ExecutionDetail::new("test", 0, Map::new());
    }

    #[test]
    fn rejects_blank_or_oversized_kind() {
        let blank = std::panic::catch_unwind(|| ExecutionDetail::new("\t", 1, Map::new()));
        let oversized = std::panic::catch_unwind(|| ExecutionDetail::new("x".repeat(MAX_KIND_CHARACTERS + 1), 1, Map::new()));
        let nul = std::panic::catch_unwind(|| ExecutionDetail::new("http\0exchange", 1, Map::new()));

        assert!(blank.is_err());
        assert!(oversized.is_err());
        assert!(nul.is_err());
    }

    #[test]
    fn rejects_nul_in_payload_keys_or_values() {
        let nul_value = Map::from_iter([("body".into(), json!("a\0b"))]);
        let nul_key = Map::from_iter([("body\0value".into(), json!("content"))]);

        assert!(std::panic::catch_unwind(|| ExecutionDetail::new("test", 1, nul_value)).is_err());
        assert!(std::panic::catch_unwind(|| ExecutionDetail::new("test", 1, nul_key)).is_err());
    }
}
