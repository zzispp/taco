use std::{
    env,
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
};

use crate::{Settings, SettingsError};

const CONFIG_ARGUMENT: &str = "--config";
const REQUIRED_REDIS_OPTION_FIELDS: [(&str, &str); 4] = [
    ("username", "redis.username"),
    ("password", "redis.password"),
    ("database", "redis.database"),
    ("protocol", "redis.protocol"),
];

impl Settings {
    pub fn load() -> Result<Self, SettingsError> {
        Self::load_from_args(env::args_os())
    }

    pub fn load_from_args<I, S>(args: I) -> Result<Self, SettingsError>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let path = explicit_config_path(args)?;
        let contents = fs::read_to_string(&path).map_err(|source| SettingsError::ReadConfiguration {
            path: path.display().to_string(),
            source,
        })?;
        let document = serde_yaml::from_str(&contents)?;
        require_explicit_redis_option_fields(&document)?;
        let settings: Self = serde_yaml::from_value(document)?;
        let settings = resolve_data_directory(settings, &path)?;
        settings.validate()?;
        Ok(settings)
    }
}

fn resolve_data_directory(settings: Settings, configuration_path: &Path) -> Result<Settings, SettingsError> {
    crate::validation::validate_data_directory_value(&settings.data_directory)?;
    if settings.data_directory.is_absolute() {
        return Ok(settings);
    }

    let configuration_directory = configuration_path
        .parent()
        .ok_or_else(|| SettingsError::ConfigurationPathWithoutParent(configuration_path.display().to_string()))?;
    let data_directory = configuration_directory.join(&settings.data_directory);
    Ok(Settings { data_directory, ..settings })
}

fn require_explicit_redis_option_fields(document: &serde_yaml::Value) -> Result<(), SettingsError> {
    let Some(root) = document.as_mapping() else {
        return Ok(());
    };
    let Some(redis) = root.get(serde_yaml::Value::String("redis".into())) else {
        return Ok(());
    };
    let Some(redis) = redis.as_mapping() else {
        return Ok(());
    };

    for (field, path) in REQUIRED_REDIS_OPTION_FIELDS {
        if !redis.contains_key(serde_yaml::Value::String(field.into())) {
            return Err(SettingsError::MissingConfigField(path));
        }
    }
    Ok(())
}

fn explicit_config_path<I, S>(args: I) -> Result<PathBuf, SettingsError>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let mut args = args.into_iter().map(Into::into);
    let mut path = None;

    while let Some(argument) = args.next() {
        if argument != OsStr::new(CONFIG_ARGUMENT) {
            continue;
        }
        if path.is_some() {
            return Err(SettingsError::RepeatedConfigArgument);
        }
        let value = args.next().ok_or(SettingsError::MissingConfigArgument)?;
        if value.is_empty() {
            return Err(SettingsError::MissingConfigArgument);
        }
        path = Some(PathBuf::from(value));
    }

    let path = path.ok_or(SettingsError::MissingConfigArgument)?;
    absolute_config_path(path)
}

fn absolute_config_path(path: PathBuf) -> Result<PathBuf, SettingsError> {
    if path.is_absolute() {
        return Ok(path);
    }

    let display = path.display().to_string();
    std::path::absolute(path).map_err(|source| SettingsError::ResolveConfigurationPath { path: display, source })
}
