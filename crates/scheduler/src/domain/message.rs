use std::collections::BTreeMap;

use kernel::error::LocalizedError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalizedMessage {
    pub key: String,
    pub params: BTreeMap<String, String>,
}

impl LocalizedMessage {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            params: BTreeMap::new(),
        }
    }

    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }
}

impl From<&LocalizedError> for LocalizedMessage {
    fn from(error: &LocalizedError) -> Self {
        let params = error.params().iter().map(|param| (param.key().to_owned(), param.value().to_owned())).collect();
        Self {
            key: error.key().to_owned(),
            params,
        }
    }
}
