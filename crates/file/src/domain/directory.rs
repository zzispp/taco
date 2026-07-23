use serde::{Deserialize, Serialize};

use crate::error::keys;
use crate::{FileError, FileResult};

const ENTRY_NAME_MAX_BYTES: usize = 255;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct EntryName {
    display: String,
    normalized: String,
}

impl EntryName {
    pub fn new(value: impl Into<String>) -> FileResult<Self> {
        let display = value.into();
        if display.trim().is_empty() || display.len() > ENTRY_NAME_MAX_BYTES {
            return Err(FileError::InvalidInput(keys::ENTRY_NAME_INVALID));
        }
        if display == "." || display == ".." || display.contains(['/', '\\']) || display.chars().any(char::is_control) {
            return Err(FileError::InvalidInput(keys::ENTRY_NAME_FORBIDDEN_PATH));
        }
        let normalized = display.to_lowercase();
        Ok(Self { display, normalized })
    }

    pub fn as_str(&self) -> &str {
        &self.display
    }

    pub fn normalized(&self) -> &str {
        &self.normalized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_names_normalize_case_and_reject_path_components() {
        let name = EntryName::new("Photo.PNG").unwrap();

        assert_eq!(name.as_str(), "Photo.PNG");
        assert_eq!(name.normalized(), "photo.png");
        for value in ["", "..", "a/b", "a\\b"] {
            assert!(EntryName::new(value).is_err(), "{value:?} should be rejected");
        }
    }
}
