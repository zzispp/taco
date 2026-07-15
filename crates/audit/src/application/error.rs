use kernel::error::LocalizedError;

pub type AuditResult<T> = Result<T, AuditError>;

#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("audit log not found")]
    NotFound,
    #[error("invalid audit input: {0}")]
    InvalidInput(LocalizedError),
    #[error("invalid audit cursor")]
    InvalidCursor,
    #[error("audit infrastructure failure: {0}")]
    Infrastructure(String),
}

pub fn localized(key: &'static str) -> LocalizedError {
    LocalizedError::new(key)
}

pub fn localized_param(key: &'static str, name: &'static str, value: impl Into<String>) -> LocalizedError {
    LocalizedError::new(key).with_param(name, value)
}
