mod access;
mod deserializer;

use std::{error::Error, fmt};

use config_rs::{Value, ValueKind};
use serde::de;

use crate::{EnvironmentReadError, EnvironmentReader};

pub(crate) enum InterpolationErrorKind {
    Message(String),
    MissingEnvironmentVariable(String),
    InvalidEnvironmentEncoding(String),
    InvalidEnvironmentPlaceholder,
    InvalidEnvironmentValue { variable: String, expected: &'static str },
}

#[derive(Debug)]
pub(crate) struct InterpolationError(InterpolationErrorKind);

impl InterpolationError {
    pub(crate) fn into_kind(self) -> InterpolationErrorKind {
        self.0
    }

    pub(super) fn invalid_environment_value(variable: String, expected: &'static str) -> Self {
        Self(InterpolationErrorKind::InvalidEnvironmentValue { variable, expected })
    }

    pub(super) fn environment_type(variable: String, expected: &'static str) -> Self {
        Self::invalid_environment_value(variable, expected)
    }
}

impl fmt::Debug for InterpolationErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for InterpolationErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(message) => formatter.write_str(message),
            Self::MissingEnvironmentVariable(variable) => write!(formatter, "environment variable {variable} is not set"),
            Self::InvalidEnvironmentEncoding(variable) => write!(formatter, "environment variable {variable} is not valid UTF-8"),
            Self::InvalidEnvironmentPlaceholder => formatter.write_str("environment placeholder must be a complete ${VAR} scalar"),
            Self::InvalidEnvironmentValue { variable, expected } => {
                write!(formatter, "environment variable {variable} is not a valid {expected}")
            }
        }
    }
}

impl fmt::Display for InterpolationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

impl Error for InterpolationError {}

impl de::Error for InterpolationError {
    fn custom<T: fmt::Display>(message: T) -> Self {
        Self(InterpolationErrorKind::Message(message.to_string()))
    }
}

impl From<config_rs::ConfigError> for InterpolationError {
    fn from(error: config_rs::ConfigError) -> Self {
        Self(InterpolationErrorKind::Message(error.to_string()))
    }
}

pub(super) enum Input {
    Config(Value),
    Environment { variable: String, value: String },
}

pub(crate) struct InterpolatingDeserializer<'a> {
    pub(super) input: Input,
    pub(super) environment: &'a dyn EnvironmentReader,
}

impl<'a> InterpolatingDeserializer<'a> {
    pub(crate) fn new(value: Value, environment: &'a dyn EnvironmentReader) -> Self {
        Self {
            input: Input::Config(value),
            environment,
        }
    }

    pub(super) fn resolve(self) -> Result<Self, InterpolationError> {
        let Input::Config(config_value) = self.input else {
            return Ok(self);
        };
        let ValueKind::String(value) = &config_value.kind else {
            return Ok(Self {
                input: Input::Config(config_value),
                environment: self.environment,
            });
        };
        let Some(variable) = environment_variable(value)? else {
            return Ok(Self {
                input: Input::Config(config_value),
                environment: self.environment,
            });
        };
        let variable = variable.to_owned();
        let value = self.environment.read(&variable).map_err(|error| environment_read_error(&variable, error))?;
        let value = value.ok_or_else(|| InterpolationError(InterpolationErrorKind::MissingEnvironmentVariable(variable.clone())))?;
        Ok(Self {
            input: Input::Environment { variable, value },
            environment: self.environment,
        })
    }
}

fn environment_variable(value: &str) -> Result<Option<&str>, InterpolationError> {
    if !value.contains("${") {
        return Ok(None);
    }
    let Some(variable) = value.strip_prefix("${").and_then(|value| value.strip_suffix('}')) else {
        return Err(InterpolationError(InterpolationErrorKind::InvalidEnvironmentPlaceholder));
    };
    if !valid_environment_variable(variable) {
        return Err(InterpolationError(InterpolationErrorKind::InvalidEnvironmentPlaceholder));
    }
    Ok(Some(variable))
}

fn valid_environment_variable(variable: &str) -> bool {
    let mut characters = variable.chars();
    matches!(characters.next(), Some('_' | 'A'..='Z' | 'a'..='z')) && characters.all(|character| matches!(character, '_' | 'A'..='Z' | 'a'..='z' | '0'..='9'))
}

fn environment_read_error(variable: &str, error: EnvironmentReadError) -> InterpolationError {
    match error {
        EnvironmentReadError::NotUnicode => InterpolationError(InterpolationErrorKind::InvalidEnvironmentEncoding(variable.to_owned())),
    }
}
