use kernel::error::LocalizedError;
use serde::Deserialize;
use serde_json::Value;

use super::{AppError, AppResult};

const SECONDS_PER_MINUTE: u64 = 60;
const DEFAULT_PASSWORD_MIN_LENGTH: usize = 8;
const DEFAULT_PASSWORD_MAX_LENGTH: usize = 128;

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
#[serde(deny_unknown_fields)]
pub struct LoginLockConfig {
    pub max_retry_count: u32,
    pub lock_minutes: u64,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: DEFAULT_PASSWORD_MIN_LENGTH,
            max_length: DEFAULT_PASSWORD_MAX_LENGTH,
            require_letter: false,
            require_number: false,
            require_symbol: false,
            forbid_username_contains: true,
        }
    }
}

impl PasswordPolicy {
    pub fn validate(&self) -> AppResult<()> {
        if self.min_length == 0 || self.max_length < self.min_length {
            return Err(invalid_config(constants::system_config::PASSWORD_POLICY_KEY));
        }
        Ok(())
    }
}

impl AvatarConfig {
    pub fn validate(&self) -> AppResult<()> {
        if self.max_bytes == 0 {
            return Err(invalid_config(constants::system_config::AVATAR_CONFIG_KEY));
        }
        Ok(())
    }
}

pub fn parse_password_policy(value: &str) -> AppResult<PasswordPolicy> {
    parse_config(value, constants::system_config::PASSWORD_POLICY_KEY, PasswordPolicy::validate)
}

pub fn parse_avatar_config(value: &str) -> AppResult<AvatarConfig> {
    parse_config(value, constants::system_config::AVATAR_CONFIG_KEY, AvatarConfig::validate)
}

pub fn parse_export_batch_config(value: &str) -> AppResult<kernel::runtime_config::ExportBatchConfig> {
    kernel::runtime_config::parse_export_batch_config(value).map_err(|error| {
        taco_tracing::error_with_fields!(
            "invalid user export runtime config",
            &error,
            key = constants::system_config::EXPORT_BATCH_CONFIG_KEY
        );
        invalid_config(constants::system_config::EXPORT_BATCH_CONFIG_KEY)
    })
}

impl LoginLockConfig {
    pub fn validate(&self) -> AppResult<()> {
        if self.max_retry_count == 0 || self.lock_minutes == 0 {
            return Err(invalid_config(constants::system_config::LOGIN_LOCK_CONFIG_KEY));
        }
        self.lock_seconds()?;
        Ok(())
    }

    pub fn lock_seconds(&self) -> AppResult<u64> {
        self.lock_minutes
            .checked_mul(SECONDS_PER_MINUTE)
            .ok_or_else(|| invalid_config(constants::system_config::LOGIN_LOCK_CONFIG_KEY))
    }
}

pub fn parse_login_lock_config(value: &str) -> AppResult<LoginLockConfig> {
    parse_config(value, constants::system_config::LOGIN_LOCK_CONFIG_KEY, LoginLockConfig::validate)
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
    use kernel::{
        error::LocalizedError,
        runtime_config::{ExportBatchConfig, MAX_EXPORT_BATCH_PAGE_SIZE},
    };

    use super::{AppError, LoginLockConfig, PasswordPolicy, parse_export_batch_config, parse_login_lock_config};

    const EXPECTED_TEN_MINUTE_LOCK_SECONDS: u64 = 600;

    #[test]
    fn default_password_policy_matches_the_seed_security_baseline() {
        assert_eq!(
            PasswordPolicy::default(),
            PasswordPolicy {
                min_length: 8,
                max_length: 128,
                require_letter: false,
                require_number: false,
                require_symbol: false,
                forbid_username_contains: true,
            }
        );
    }

    #[test]
    fn export_batch_config_reuses_shared_validation_and_preserves_user_error() {
        assert_eq!(parse_export_batch_config(r#"{"page_size":100}"#).unwrap(), ExportBatchConfig { page_size: 100 });
        let above_maximum = format!(r#"{{"page_size":{}}}"#, MAX_EXPORT_BATCH_PAGE_SIZE + 1);

        for invalid in [r#"{"page_size":0}"#, &above_maximum, r#"{"page_size":100,"unexpected":true}"#, "not-json"] {
            let AppError::InvalidInput(error) = parse_export_batch_config(invalid).unwrap_err() else {
                panic!("invalid export config must use the user invalid-input error");
            };
            assert_eq!(
                error,
                LocalizedError::new("errors.user.invalid_system_config").with_param("key", constants::system_config::EXPORT_BATCH_CONFIG_KEY)
            );
        }
    }

    #[test]
    fn login_lock_config_requires_positive_values() {
        let config = LoginLockConfig {
            max_retry_count: 5,
            lock_minutes: 10,
        };
        assert_eq!(parse_login_lock_config(r#"{"max_retry_count":5,"lock_minutes":10}"#).unwrap(), config);
        assert_eq!(config.lock_seconds().unwrap(), EXPECTED_TEN_MINUTE_LOCK_SECONDS);

        for invalid in [
            r#"{"max_retry_count":0,"lock_minutes":10}"#,
            r#"{"max_retry_count":5,"lock_minutes":0}"#,
            r#"{"max_retry_count":5,"lock_minutes":10,"extra":true}"#,
            "not-json",
        ] {
            assert!(matches!(parse_login_lock_config(invalid), Err(AppError::InvalidInput(_))));
        }
        assert!(matches!(
            LoginLockConfig {
                max_retry_count: 5,
                lock_minutes: u64::MAX,
            }
            .validate(),
            Err(AppError::InvalidInput(_))
        ));
    }
}
