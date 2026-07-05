use crate::{DatabaseSettings, RedisSettings, Settings, SettingsError};
use config_rs::{Config, File};
use std::{
    env,
    path::{Path, PathBuf},
};

const CONFIG_ARG: &str = "--config";
pub(crate) const MODULE_CONFIG_PATH: &str = "config/config.yaml";
pub(crate) const ROOT_CONFIG_PATH: &str = "config.yaml";

impl Settings {
    pub fn load() -> Result<Self, SettingsError> {
        Self::load_from_args(env::args_os())
    }

    pub fn load_from_args<I, S>(args: I) -> Result<Self, SettingsError>
    where
        I: IntoIterator<Item = S>,
        S: Into<std::ffi::OsString>,
    {
        let path = resolve_config_path(args)?;
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
        required_config_value("jwt.secret", &self.jwt.secret)
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

pub(crate) fn explicit_config_path(args: &[std::ffi::OsString]) -> Result<Option<PathBuf>, SettingsError> {
    let Some(index) = args.iter().position(|arg| arg == CONFIG_ARG) else {
        return Ok(None);
    };

    args.get(index + 1).map(PathBuf::from).map(Some).ok_or(SettingsError::MissingConfigArgument)
}

fn resolve_config_path<I, S>(args: I) -> Result<PathBuf, SettingsError>
where
    I: IntoIterator<Item = S>,
    S: Into<std::ffi::OsString>,
{
    let args = args.into_iter().map(Into::into).collect::<Vec<std::ffi::OsString>>();
    if let Some(path) = explicit_config_path(&args)? {
        return Ok(path);
    }

    [MODULE_CONFIG_PATH, ROOT_CONFIG_PATH]
        .into_iter()
        .map(PathBuf::from)
        .find(|path| Path::new(path).is_file())
        .ok_or(SettingsError::MissingConfigFile)
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
