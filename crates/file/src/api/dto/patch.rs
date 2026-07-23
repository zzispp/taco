use serde::{Deserialize, Deserializer};

#[derive(Debug, Default)]
pub(in crate::api) enum NullablePatch<T> {
    #[default]
    Missing,
    Null,
    Value(T),
}

impl<'de, T> Deserialize<'de> for NullablePatch<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match Option::<T>::deserialize(deserializer)? {
            Some(value) => Self::Value(value),
            None => Self::Null,
        })
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::NullablePatch;

    #[derive(Deserialize)]
    struct Payload {
        #[serde(default)]
        parent_id: NullablePatch<String>,
    }

    #[test]
    fn distinguishes_missing_null_and_present_values() {
        let missing: Payload = serde_json::from_str("{}").unwrap();
        let null: Payload = serde_json::from_str(r#"{"parent_id":null}"#).unwrap();
        let value: Payload = serde_json::from_str(r#"{"parent_id":"folder-id"}"#).unwrap();

        assert!(matches!(missing.parent_id, NullablePatch::Missing));
        assert!(matches!(null.parent_id, NullablePatch::Null));
        assert!(matches!(value.parent_id, NullablePatch::Value(id) if id == "folder-id"));
    }
}
