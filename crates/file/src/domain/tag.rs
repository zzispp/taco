use crate::error::keys;
use crate::{FileError, FileResult};

const TAG_NAME_MAX_CHARS: usize = 100;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TagName(String);

impl TagName {
    pub fn new(value: impl Into<String>) -> FileResult<Self> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(FileError::InvalidInput(keys::TAG_REQUIRED));
        }
        if trimmed.chars().any(|character| character == '\0') {
            return Err(FileError::InvalidInput(keys::TAG_INVALID));
        }
        if trimmed.chars().count() > TAG_NAME_MAX_CHARS {
            return Err(FileError::InvalidInput(keys::TAG_TOO_LONG));
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn normalized(&self) -> String {
        self.0.to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::TagName;
    use crate::FileError;
    use crate::error::keys;

    #[test]
    fn tag_names_trim_and_normalize_without_accepting_invalid_storage_values() {
        let tag = TagName::new("  Work  ").unwrap();
        assert_eq!(tag.as_str(), "Work");
        assert_eq!(tag.normalized(), "work");
        assert_eq!(TagName::new("\0").unwrap_err(), FileError::InvalidInput(keys::TAG_INVALID));
        assert_eq!(TagName::new("x".repeat(101)).unwrap_err(), FileError::InvalidInput(keys::TAG_TOO_LONG));
    }
}
