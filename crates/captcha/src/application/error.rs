use kernel::error::LocalizedError;

pub type CaptchaResult<T> = Result<T, CaptchaError>;

#[derive(Debug, thiserror::Error)]
pub enum CaptchaError {
    #[error("{0}")]
    InvalidInput(LocalizedError),
    #[error("{0}")]
    Infrastructure(String),
}
