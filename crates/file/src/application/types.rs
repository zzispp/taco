use std::pin::Pin;

use bytes::Bytes;
use futures_util::Stream;
use serde::{Deserialize, Serialize};

use crate::domain::{ByteSize, ContentDigest, PartNumber, ProviderKey, StoredObjectId, UploadId};
use crate::error::keys;
use crate::{FileError, FileResult};

const OBJECT_KEY_MAX_BYTES: usize = 1_024;

pub type ByteStream = Pin<Box<dyn Stream<Item = FileResult<Bytes>> + Send + 'static>>;
pub type ObjectStream = ByteStream;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ObjectKey(String);

impl ObjectKey {
    pub fn new(value: impl Into<String>) -> FileResult<Self> {
        let value = value.into();
        if value.trim().is_empty() || value.len() > OBJECT_KEY_MAX_BYTES || value.starts_with('/') || value.contains('\\') {
            return Err(FileError::InvalidInput(keys::OBJECT_KEY_INVALID));
        }
        if value
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == ".." || part.chars().any(char::is_control))
        {
            return Err(FileError::InvalidInput(keys::OBJECT_KEY_FORBIDDEN_PATH));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ProviderUploadRef(String);

impl ProviderUploadRef {
    pub fn new(value: impl Into<String>) -> FileResult<Self> {
        let value = value.into();
        if value.trim().is_empty() || value.chars().any(char::is_control) {
            return Err(FileError::InvalidInput(keys::PROVIDER_UPLOAD_REF_INVALID));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ProviderPartRef(String);

impl ProviderPartRef {
    pub fn new(value: impl Into<String>) -> FileResult<Self> {
        let value = value.into();
        if value.trim().is_empty() || value.chars().any(char::is_control) {
            return Err(FileError::InvalidInput(keys::PROVIDER_PART_REF_INVALID));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ByteRange {
    start: u64,
    end_exclusive: u64,
}

impl ByteRange {
    pub fn new(start: u64, end_exclusive: u64) -> FileResult<Self> {
        (start < end_exclusive)
            .then_some(Self { start, end_exclusive })
            .ok_or(FileError::RangeNotSatisfiable)
    }

    pub const fn start(self) -> u64 {
        self.start
    }

    pub const fn end_exclusive(self) -> u64 {
        self.end_exclusive
    }

    pub const fn byte_len(self) -> u64 {
        self.end_exclusive - self.start
    }

    pub fn within(self, size: ByteSize) -> FileResult<Self> {
        (self.end_exclusive <= size.bytes()).then_some(self).ok_or(FileError::RangeNotSatisfiable)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BeginUpload {
    pub stored_object_id: StoredObjectId,
    pub expected_size: ByteSize,
    pub expected_digest: ContentDigest,
    pub part_size: ByteSize,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct UploadSession {
    pub stored_object_id: StoredObjectId,
    pub provider_key: ProviderKey,
    pub provider_upload_ref: ProviderUploadRef,
    pub key: ObjectKey,
    pub expected_size: ByteSize,
    pub expected_digest: ContentDigest,
    pub part_size: ByteSize,
}

pub struct UploadPart {
    pub provider_upload_ref: ProviderUploadRef,
    pub part_number: PartNumber,
    pub expected_digest: ContentDigest,
    pub body: ByteStream,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PartReceipt {
    pub session_id: UploadId,
    pub part_number: PartNumber,
    pub provider_part_ref: ProviderPartRef,
    pub size: ByteSize,
    pub digest: ContentDigest,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProviderPartReceipt {
    pub part_number: PartNumber,
    pub provider_part_ref: ProviderPartRef,
    pub size: ByteSize,
    pub digest: ContentDigest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteUpload {
    pub provider_upload_ref: ProviderUploadRef,
    pub parts: Vec<ProviderPartReceipt>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StoredObject {
    pub id: StoredObjectId,
    pub provider_key: ProviderKey,
    pub key: ObjectKey,
    pub size: ByteSize,
    pub digest: Option<ContentDigest>,
}

pub struct ObjectRead {
    pub object: StoredObject,
    pub range: Option<ByteRange>,
    pub body: ObjectStream,
}
