use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::keys;

macro_rules! uuid_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            pub const fn nil() -> Self {
                Self(Uuid::nil())
            }

            pub const fn as_uuid(self) -> Uuid {
                self.0
            }

            pub fn parse(value: &str) -> crate::FileResult<Self> {
                Uuid::parse_str(value)
                    .map(Self)
                    .map_err(|_| crate::FileError::InvalidInput(keys::IDENTIFIER_INVALID))
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

uuid_id!(FileId);
uuid_id!(DirectoryId);
uuid_id!(StoredObjectId);
uuid_id!(UploadId);

impl DirectoryId {
    pub const ROOT: Self = Self::nil();
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct SpaceId(String);

impl SpaceId {
    pub fn new(value: impl Into<String>) -> crate::FileResult<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(crate::FileError::InvalidInput(keys::SPACE_IDENTIFIER_REQUIRED));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SpaceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ProviderKey(String);

impl ProviderKey {
    pub fn new(value: impl Into<String>) -> crate::FileResult<Self> {
        let value = value.into();
        if value.trim().is_empty()
            || value
                .chars()
                .any(|character| !(character.is_ascii_lowercase() || character.is_ascii_digit() || ".-_".contains(character)))
        {
            return Err(crate::FileError::InvalidInput(keys::PROVIDER_KEY_INVALID));
        }
        Ok(Self(value))
    }

    pub fn local() -> Self {
        Self("local".into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_id_preserves_an_opaque_user_identifier() {
        let id = SpaceId::new("user-1").unwrap();

        assert_eq!(id.as_str(), "user-1");
        assert_eq!(id.to_string(), "user-1");
        assert_eq!(SpaceId::new("  ").unwrap_err(), crate::FileError::InvalidInput(keys::SPACE_IDENTIFIER_REQUIRED));
    }
}
