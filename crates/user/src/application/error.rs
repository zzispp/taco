use kernel::error::LocalizedError;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid cursor")]
    InvalidCursor,
    #[error("invalid input: {0}")]
    InvalidInput(LocalizedError),
    #[error("user import validation failed for {} rows", .0.len())]
    ImportValidation(Vec<LocalizedError>),
    #[error("username or password is incorrect")]
    Unauthorized,
    #[error("account is disabled")]
    AccountDisabled,
    #[error("account is locked for {lock_minutes} minutes")]
    AccountLocked { lock_minutes: u64 },
    #[error("forbidden: {0}")]
    Forbidden(LocalizedError),
    #[error("resource conflict: {0}")]
    Conflict(LocalizedError),
    #[error("user not found")]
    NotFound,
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}
