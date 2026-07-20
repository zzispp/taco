use thiserror::Error;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("installation state is not complete")]
    IncompleteInstallation,
    #[error("data directory cannot be represented as a UTF-8 avatar path")]
    NonUnicodeDataDirectory,
    #[error("{0} cannot be blank")]
    BlankConfigValue(&'static str),
    #[error("jwt.secret must not use the known insecure development value")]
    InsecureJwtSecret,
    #[error("jwt.secret must be at least {minimum_bytes} UTF-8 bytes; got {actual_bytes}")]
    JwtSecretTooShort { minimum_bytes: usize, actual_bytes: usize },
    #[error("{0} must be greater than 0")]
    NonPositiveNumber(&'static str),
    #[error("{0} is not a valid URL component")]
    InvalidUrlComponent(&'static str),
}
