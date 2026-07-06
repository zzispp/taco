use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalizedError {
    key: &'static str,
    params: Vec<ErrorParam>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ErrorParam {
    key: &'static str,
    value: String,
}

impl LocalizedError {
    pub fn new(key: &'static str) -> Self {
        Self { key, params: Vec::new() }
    }

    pub fn with_param(mut self, key: &'static str, value: impl Into<String>) -> Self {
        self.params.push(ErrorParam { key, value: value.into() });
        self
    }

    pub const fn key(&self) -> &'static str {
        self.key
    }

    pub fn params(&self) -> &[ErrorParam] {
        &self.params
    }
}

impl ErrorParam {
    pub const fn key(&self) -> &'static str {
        self.key
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl Display for LocalizedError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.key)?;
        for param in &self.params {
            write!(formatter, " {}={}", param.key, param.value)?;
        }
        Ok(())
    }
}
