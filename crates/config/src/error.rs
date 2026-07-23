use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("--config <path> is required")]
    MissingConfigArgument,
    #[error("--config can only be supplied once")]
    RepeatedConfigArgument,
    #[error("failed to read configuration file {path}: {source}")]
    ReadConfiguration {
        path: String,
        #[source]
        source: io::Error,
    },
    #[error("failed to resolve configuration path {path}: {source}")]
    ResolveConfigurationPath {
        path: String,
        #[source]
        source: io::Error,
    },
    #[error("configuration path has no parent directory: {0}")]
    ConfigurationPathWithoutParent(String),
    #[error("invalid YAML configuration: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("data_directory must be an absolute path")]
    RelativeDataDirectory,
    #[error("{0} must be explicitly present in configuration")]
    MissingConfigField(&'static str),
    #[error("{0} cannot be blank")]
    BlankConfigValue(&'static str),
    #[error("{0} contains an example placeholder")]
    PlaceholderConfigValue(&'static str),
    #[error("{0} contains unsupported environment interpolation")]
    EnvironmentInterpolation(&'static str),
    #[error("jwt.secret must not use the known insecure development value")]
    InsecureJwtSecret,
    #[error("jwt.secret must be at least {minimum_bytes} UTF-8 bytes; got {actual_bytes}")]
    JwtSecretTooShort { minimum_bytes: usize, actual_bytes: usize },
    #[error("{0} must be greater than 0")]
    NonPositiveNumber(&'static str),
    #[error("{0} is not a valid URL component")]
    InvalidUrlComponent(&'static str),
}
