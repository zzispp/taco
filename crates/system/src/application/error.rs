use kernel::error::LocalizedError;
use thiserror::Error;

pub type SystemResult<T> = Result<T, SystemError>;

#[derive(Debug, Error)]
pub enum SystemError {
    #[error("resource not found")]
    NotFound,
    #[error("forbidden: {0}")]
    Forbidden(LocalizedError),
    #[error("resource conflict: {0}")]
    Conflict(LocalizedError),
    #[error("invalid input: {0}")]
    InvalidInput(LocalizedError),
    #[error("invalid cursor")]
    InvalidCursor,
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}
