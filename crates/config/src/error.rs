use thiserror::Error;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("configuration error: {0}")]
    Config(#[from] config_rs::ConfigError),
    #[error("database.password is required when database.url is not set")]
    MissingDatabasePassword,
    #[error("--config <path> is required")]
    MissingConfigArgument,
    #[error("{0} cannot be blank")]
    BlankConfigValue(&'static str),
    #[error("jwt.secret must not use the known insecure development value")]
    InsecureJwtSecret,
    #[error("jwt.secret must be at least {minimum_bytes} UTF-8 bytes; got {actual_bytes}")]
    JwtSecretTooShort { minimum_bytes: usize, actual_bytes: usize },
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
    #[error("{0} requires concrete HTTP(S) origins and cannot use '*'")]
    WildcardCorsOrigin(&'static str),
    #[error("invalid HTTP(S) origin in {key}: {value}")]
    InvalidHttpOrigin { key: &'static str, value: String },
    #[error("{0} must be an absolute cookie path")]
    InvalidCookiePath(&'static str),
    #[error("auth.refresh_cookie.secure must be true")]
    InsecureRefreshCookie,
    #[error("{0} must be greater than 0")]
    NonPositiveNumber(&'static str),
    #[error("{0} is not a valid tracing level filter")]
    InvalidTracingFilter(&'static str),
}
