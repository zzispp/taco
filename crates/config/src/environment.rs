use std::env;

use thiserror::Error;

pub trait EnvironmentReader {
    fn read(&self, variable: &str) -> Result<Option<String>, EnvironmentReadError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum EnvironmentReadError {
    #[error("environment value is not valid UTF-8")]
    NotUnicode,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ProcessEnvironment;

impl EnvironmentReader for ProcessEnvironment {
    fn read(&self, variable: &str) -> Result<Option<String>, EnvironmentReadError> {
        match env::var(variable) {
            Ok(value) => Ok(Some(value)),
            Err(env::VarError::NotPresent) => Ok(None),
            Err(env::VarError::NotUnicode(_)) => Err(EnvironmentReadError::NotUnicode),
        }
    }
}
