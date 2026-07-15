use crate::{DatabaseSettings, RedisSettings, Settings, SettingsError};
use config_rs::{Config, File};
use sha2::{Digest, Sha256};
use std::{env, path::PathBuf};

const CONFIG_ARG: &str = "--config";
const KNOWN_INSECURE_JWT_SECRET_SHA256: [u8; 32] = [
    0xb8, 0x9f, 0x85, 0xb2, 0x25, 0x06, 0xeb, 0x72, 0xf6, 0x0b, 0x3a, 0x7b, 0x3c, 0xa1, 0xd0, 0x9b, 0x74, 0x90, 0xd0, 0xe0, 0x52, 0xe2, 0x08, 0xec, 0xfd, 0xa4,
    0x88, 0xe0, 0x7a, 0x09, 0x66, 0x8f,
];
const MIN_JWT_SECRET_BYTES: usize = 32;

impl Settings {
    pub fn load() -> Result<Self, SettingsError> {
        Self::load_from_args(env::args_os())
    }

    pub fn load_from_args<I, S>(args: I) -> Result<Self, SettingsError>
    where
        I: IntoIterator<Item = S>,
        S: Into<std::ffi::OsString>,
    {
        let args = args.into_iter().map(Into::into).collect::<Vec<std::ffi::OsString>>();
        let path = explicit_config_path(&args)?;
        let settings: Settings = Config::builder()
            .add_source(File::from(path))
            .build()?
            .try_deserialize()
            .map_err(SettingsError::from)?;
        settings.validate()?;
        Ok(settings)
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    pub fn database_url(&self) -> Result<String, SettingsError> {
        if let Some(url) = non_empty_url(self.database.url.as_deref()) {
            return Ok(url.to_owned());
        }

        let password = self.database.password.as_ref().ok_or(SettingsError::MissingDatabasePassword)?;
        Ok(format!(
            "{}://{}:{}@{}:{}/{}",
            self.database.scheme, self.database.username, password, self.database.host, self.database.port, self.database.name
        ))
    }

    pub fn jwt_secret(&self) -> Result<String, SettingsError> {
        let secret = required_config_value("jwt.secret", &self.jwt.secret)?;
        if is_known_insecure_jwt_secret(&secret) {
            return Err(SettingsError::InsecureJwtSecret);
        }
        let actual_bytes = secret.len();
        if actual_bytes < MIN_JWT_SECRET_BYTES {
            return Err(SettingsError::JwtSecretTooShort {
                minimum_bytes: MIN_JWT_SECRET_BYTES,
                actual_bytes,
            });
        }
        Ok(secret)
    }

    pub fn cloudflare_turnstile_secret_key(&self) -> String {
        self.captcha.cloudflare_turnstile.secret_key.trim().to_owned()
    }

    pub fn redis_url(&self) -> Result<String, SettingsError> {
        if let Some(url) = non_empty_url(self.redis.url.as_deref()) {
            return Ok(url.to_owned());
        }

        Ok(format!(
            "{}://{}{}{}{}",
            self.redis.scheme,
            redis_auth(&self.redis),
            self.redis.host,
            redis_port(self.redis.port),
            redis_query(&self.redis)
        ))
    }

    pub fn tracing_log_level(&self) -> Result<String, SettingsError> {
        required_config_value("tracing.log_level", &self.tracing.log_level)
    }
}

fn is_known_insecure_jwt_secret(secret: &str) -> bool {
    let digest: [u8; 32] = Sha256::digest(secret.as_bytes()).into();
    digest == KNOWN_INSECURE_JWT_SECRET_SHA256
}

pub(crate) fn explicit_config_path(args: &[std::ffi::OsString]) -> Result<PathBuf, SettingsError> {
    let index = args.iter().position(|arg| arg == CONFIG_ARG).ok_or(SettingsError::MissingConfigArgument)?;
    args.get(index + 1).map(PathBuf::from).ok_or(SettingsError::MissingConfigArgument)
}

fn non_empty_url(url: Option<&str>) -> Option<&str> {
    match url {
        Some(value) if !value.trim().is_empty() => Some(value.trim()),
        _ => None,
    }
}

fn redis_auth(settings: &RedisSettings) -> String {
    let Some(username) = settings.username.as_deref().map(str::trim).filter(|value| !value.is_empty()) else {
        return String::new();
    };

    match settings.password.as_deref() {
        Some(password) => format!("{username}:{password}@"),
        None => format!("{username}@"),
    }
}

fn redis_port(port: u16) -> String {
    format!(":{port}")
}

fn redis_query(settings: &RedisSettings) -> String {
    let path = settings.database.map(|database| format!("/{database}")).unwrap_or_default();
    let query = settings
        .protocol
        .as_deref()
        .map(str::trim)
        .filter(|protocol| !protocol.is_empty())
        .map(|protocol| format!("?protocol={protocol}"))
        .unwrap_or_default();
    format!("{path}{query}")
}

pub(crate) fn required_config_value(key: &'static str, value: &str) -> Result<String, SettingsError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(SettingsError::BlankConfigValue(key));
    }

    Ok(trimmed.to_owned())
}

#[allow(dead_code)]
fn _database_settings(_: &DatabaseSettings) {}
