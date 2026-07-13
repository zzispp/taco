use kernel::error::LocalizedError;
use serde::Deserialize;
use serde_json::Value;

use super::{AppError, AppResult};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct PasswordPolicy {
    pub min_length: usize,
    pub max_length: usize,
    #[serde(default)]
    pub require_letter: bool,
    #[serde(default)]
    pub require_number: bool,
    #[serde(default)]
    pub require_symbol: bool,
    #[serde(default)]
    pub forbid_username_contains: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct AvatarConfig {
    pub max_bytes: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct IpLocationConfig {
    pub enabled: bool,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: constants::auth::PASSWORD_MIN_LENGTH,
            max_length: constants::auth::PASSWORD_MAX_LENGTH,
            require_letter: false,
            require_number: false,
            require_symbol: false,
            forbid_username_contains: false,
        }
    }
}

impl PasswordPolicy {
    pub fn validate(&self) -> AppResult<()> {
        if self.min_length == 0 || self.max_length < self.min_length {
            return Err(invalid_config("sys.user.passwordPolicy"));
        }
        Ok(())
    }
}

impl AvatarConfig {
    pub fn validate(&self) -> AppResult<()> {
        if self.max_bytes == 0 {
            return Err(invalid_config("sys.upload.avatarConfig"));
        }
        Ok(())
    }
}

pub fn parse_password_policy(value: &str) -> AppResult<PasswordPolicy> {
    parse_config(value, "sys.user.passwordPolicy", PasswordPolicy::validate)
}

pub fn parse_avatar_config(value: &str) -> AppResult<AvatarConfig> {
    parse_config(value, "sys.upload.avatarConfig", AvatarConfig::validate)
}

pub fn parse_export_batch_config(value: &str) -> AppResult<kernel::runtime_config::ExportBatchConfig> {
    kernel::runtime_config::parse_export_batch_config(value).map_err(|_| invalid_config(constants::system_config::EXPORT_BATCH_CONFIG_KEY))
}

pub fn parse_ip_location_config(value: &str) -> AppResult<IpLocationConfig> {
    serde_json::from_str::<IpLocationConfig>(value).map_err(|_| invalid_config(constants::system_config::IP_LOCATION_CONFIG_KEY))
}

fn parse_config<T>(value: &str, key: &'static str, validate: fn(&T) -> AppResult<()>) -> AppResult<T>
where
    T: for<'de> Deserialize<'de>,
{
    let parsed = serde_json::from_str::<T>(value).map_err(|_| invalid_config(key))?;
    validate(&parsed)?;
    Ok(parsed)
}

fn invalid_config(key: &'static str) -> AppError {
    AppError::InvalidInput(LocalizedError::new("errors.user.invalid_system_config").with_param("key", key))
}

#[allow(dead_code)]
pub fn json_value(value: &str, key: &'static str) -> AppResult<Value> {
    serde_json::from_str(value).map_err(|_| invalid_config(key))
}

#[cfg(test)]
mod tests {
    use kernel::{error::LocalizedError, runtime_config::ExportBatchConfig};

    use super::{AppError, parse_export_batch_config};

    #[test]
    fn export_batch_config_reuses_shared_validation_and_preserves_user_error() {
        assert_eq!(parse_export_batch_config(r#"{"page_size":100}"#).unwrap(), ExportBatchConfig { page_size: 100 });

        for invalid in [r#"{"page_size":0}"#, r#"{"page_size":100,"unexpected":true}"#, "not-json"] {
            let AppError::InvalidInput(error) = parse_export_batch_config(invalid).unwrap_err() else {
                panic!("invalid export config must use the user invalid-input error");
            };
            assert_eq!(
                error,
                LocalizedError::new("errors.user.invalid_system_config").with_param("key", constants::system_config::EXPORT_BATCH_CONFIG_KEY)
            );
        }
    }
}
