use async_trait::async_trait;
use serde::Deserialize;

use crate::domain::ByteSize;
use crate::error::keys;
use crate::{FileError, FileResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileManagementConfig {
    pub max_file_bytes: ByteSize,
    pub default_space_quota_bytes: ByteSize,
    pub upload_part_bytes: ByteSize,
    pub upload_session_inactivity_days: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFileManagementConfig {
    max_file_bytes: u64,
    default_space_quota_bytes: u64,
    upload_part_bytes: u64,
    upload_session_inactivity_days: u64,
}

pub fn parse_file_management_config(value: &str) -> FileResult<FileManagementConfig> {
    let raw: RawFileManagementConfig = serde_json::from_str(value).map_err(|_| FileError::InvalidInput(keys::CONFIG_INVALID_JSON))?;
    validate(raw)
}

fn validate(raw: RawFileManagementConfig) -> FileResult<FileManagementConfig> {
    if raw.max_file_bytes == 0 {
        return Err(FileError::InvalidInput(keys::CONFIG_MAX_FILE_BYTES_INVALID));
    }
    if raw.default_space_quota_bytes == 0 {
        return Err(FileError::InvalidInput(keys::CONFIG_DEFAULT_QUOTA_INVALID));
    }
    if raw.upload_part_bytes == 0 || raw.upload_part_bytes > raw.max_file_bytes {
        return Err(FileError::InvalidInput(keys::CONFIG_UPLOAD_PART_BYTES_INVALID));
    }
    if raw.upload_session_inactivity_days == 0 {
        return Err(FileError::InvalidInput(keys::CONFIG_UPLOAD_SESSION_INACTIVITY_INVALID));
    }
    Ok(FileManagementConfig {
        max_file_bytes: ByteSize::from_bytes(raw.max_file_bytes),
        default_space_quota_bytes: ByteSize::from_bytes(raw.default_space_quota_bytes),
        upload_part_bytes: ByteSize::from_bytes(raw.upload_part_bytes),
        upload_session_inactivity_days: raw.upload_session_inactivity_days,
    })
}

#[async_trait]
pub trait FileManagementConfigProvider: Send + Sync + 'static {
    async fn file_management_config(&self) -> FileResult<FileManagementConfig>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_required_file_management_runtime_parameters() {
        let config =
            parse_file_management_config(r#"{"max_file_bytes":100,"default_space_quota_bytes":200,"upload_part_bytes":20,"upload_session_inactivity_days":7}"#)
                .unwrap();

        assert_eq!(config.max_file_bytes, ByteSize::from_bytes(100));
        assert_eq!(config.default_space_quota_bytes, ByteSize::from_bytes(200));
        assert_eq!(config.upload_part_bytes, ByteSize::from_bytes(20));
        assert_eq!(config.upload_session_inactivity_days, 7);
    }

    #[test]
    fn rejects_missing_or_zero_runtime_parameters() {
        assert!(parse_file_management_config("{}").is_err());
        assert!(
            parse_file_management_config(r#"{"max_file_bytes":0,"default_space_quota_bytes":200,"upload_part_bytes":20,"upload_session_inactivity_days":7}"#,)
                .is_err()
        );
    }
}
