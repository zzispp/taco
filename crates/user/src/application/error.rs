use kernel::error::LocalizedError;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid input: {0}")]
    InvalidInput(LocalizedError),
    #[error("username or password is incorrect")]
    Unauthorized,
    #[error("forbidden: {0}")]
    Forbidden(LocalizedError),
    #[error("resource conflict: {0}")]
    Conflict(LocalizedError),
    #[error("user not found")]
    NotFound,
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}
