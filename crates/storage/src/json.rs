use serde::{Serialize, de::DeserializeOwned};

use crate::{StorageError, StorageResult};

// ----------------------------------------------------------------------

pub fn encode_required<T: Serialize>(value: &T) -> StorageResult<String> {
    serde_json::to_string(value).map_err(StorageError::from)
}

pub fn encode_optional<T: Serialize>(value: &Option<T>) -> StorageResult<Option<String>> {
    value.as_ref().map(serde_json::to_string).transpose().map_err(StorageError::from)
}

pub fn decode_required<T: DeserializeOwned>(value: String) -> StorageResult<T> {
    serde_json::from_str(&value).map_err(StorageError::from)
}

pub fn decode_optional<T: DeserializeOwned>(value: Option<String>) -> StorageResult<Option<T>> {
    value.map(|text| serde_json::from_str(&text)).transpose().map_err(StorageError::from)
}
