use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("username or password is incorrect")]
    Unauthorized,
    #[error("resource conflict: {0}")]
    Conflict(String),
    #[error("user not found")]
    NotFound,
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}
