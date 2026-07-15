use thiserror::Error;

pub type ClientInfoResult<T> = Result<T, ClientInfoError>;

#[derive(Debug, Error)]
pub enum ClientInfoError {
    #[error("invalid client information config: {0}")]
    InvalidConfig(&'static str),
    #[error("invalid client information setting: {0}")]
    InvalidSetting(&'static str),
    #[error("client information provider error: {0}")]
    Provider(String),
}
