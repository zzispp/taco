use kernel::error::LocalizedError;

pub type SchedulerResult<T> = Result<T, SchedulerError>;

#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    #[error("scheduler resource not found")]
    NotFound,
    #[error("forbidden: {0}")]
    Forbidden(LocalizedError),
    #[error("{code}: {details}")]
    Conflict { code: &'static str, details: LocalizedError },
    #[error("invalid scheduler input: {0}")]
    InvalidInput(LocalizedError),
    #[error("invalid scheduler cursor")]
    InvalidCursor,
    #[error("scheduler infrastructure failure: {0}")]
    Infrastructure(String),
}

impl SchedulerError {
    pub fn conflict(code: &'static str, details_key: &'static str) -> Self {
        Self::Conflict {
            code,
            details: LocalizedError::new(details_key),
        }
    }
}

pub fn localized(key: &'static str) -> LocalizedError {
    LocalizedError::new(key)
}

pub fn localized_param(key: &'static str, name: &'static str, value: impl Into<String>) -> LocalizedError {
    LocalizedError::new(key).with_param(name, value)
}
