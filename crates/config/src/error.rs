use thiserror::Error;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("configuration error: {0}")]
    Config(#[from] config_rs::ConfigError),
    #[error("database.password is required when database.url is not set")]
    MissingDatabasePassword,
    #[error("configuration file not found")]
    MissingConfigFile,
    #[error("--config requires a file path")]
    MissingConfigArgument,
    #[error("{0} cannot be blank")]
    BlankConfigValue(&'static str),
    #[error("{0} must not be empty")]
    EmptyList(&'static str),
    #[error("{0} must not contain blank items")]
    BlankListItem(&'static str),
    #[error("{0} cannot combine '*' with other values")]
    MixedWildcardList(&'static str),
    #[error("invalid HTTP method in {key}: {value}")]
    InvalidHttpMethod { key: &'static str, value: String },
    #[error("invalid HTTP header name in {key}: {value}")]
    InvalidHttpHeaderName { key: &'static str, value: String },
    #[error("cors.allow_credentials=true cannot be combined with wildcard {0}")]
    WildcardCorsWithCredentials(&'static str),
    #[error("{0} must be greater than 0")]
    NonPositiveNumber(&'static str),
    #[error("{0} is not a valid tracing level filter")]
    InvalidTracingFilter(&'static str),
}
