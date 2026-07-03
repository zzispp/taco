use thiserror::Error;

pub type RbacResult<T> = Result<T, RbacError>;

#[derive(Debug, Error)]
pub enum RbacError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("resource not found")]
    NotFound,
    #[error("resource conflict: {0}")]
    Conflict(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}
