use kernel::error::LocalizedError;
use thiserror::Error;

pub type RbacResult<T> = Result<T, RbacError>;

#[derive(Debug, Error)]
pub enum RbacError {
    #[error("invalid cursor")]
    InvalidCursor,
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("resource not found")]
    NotFound,
    #[error("resource conflict: {0}")]
    Conflict(LocalizedError),
    #[error("invalid input: {0}")]
    InvalidInput(LocalizedError),
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}
