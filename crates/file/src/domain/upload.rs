use serde::{Deserialize, Serialize};

use crate::{FileError, FileResult};

const FIRST_PART_NUMBER: u32 = 1;
pub const DEFAULT_UPLOAD_PART_SIZE: crate::domain::ByteSize = crate::domain::ByteSize::from_bytes(16 * 1_024 * 1_024);
pub const DEFAULT_MANAGED_FILE_SIZE_LIMIT: crate::domain::ByteSize = crate::domain::ByteSize::from_bytes(10 * 1_024 * 1_024 * 1_024);
pub const DEFAULT_SPACE_QUOTA: crate::domain::ByteSize = crate::domain::ByteSize::from_bytes(20 * 1_024 * 1_024 * 1_024);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum UploadState {
    Open,
    Completing,
    Completed,
    Aborted,
    Expired,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PartNumber(u32);

impl PartNumber {
    pub fn new(value: u32) -> FileResult<Self> {
        (value >= FIRST_PART_NUMBER).then_some(Self(value)).ok_or(FileError::InvalidPart)
    }

    pub const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UploadLifecycle {
    state: UploadState,
    expected_size: crate::domain::ByteSize,
    expected_digest: crate::domain::ContentDigest,
    received: crate::domain::ByteSize,
}

impl UploadLifecycle {
    pub const fn new(expected_size: crate::domain::ByteSize, expected_digest: crate::domain::ContentDigest) -> Self {
        Self {
            state: UploadState::Open,
            expected_size,
            expected_digest,
            received: crate::domain::ByteSize::ZERO,
        }
    }

    pub const fn state(&self) -> UploadState {
        self.state
    }

    pub const fn received(&self) -> crate::domain::ByteSize {
        self.received
    }

    pub fn record_part(&mut self, size: crate::domain::ByteSize) -> FileResult<()> {
        if self.state != UploadState::Open {
            return Err(FileError::InvalidUploadTransition {
                from: self.state,
                to: UploadState::Open,
            });
        }
        let received = self.received.checked_add(size)?;
        if received > self.expected_size {
            return Err(FileError::SizeMismatch);
        }
        self.received = received;
        Ok(())
    }

    pub fn begin_completion(&mut self, digest: crate::domain::ContentDigest) -> FileResult<()> {
        if self.state != UploadState::Open {
            return Err(FileError::InvalidUploadTransition {
                from: self.state,
                to: UploadState::Completing,
            });
        }
        if self.expected_size != self.received {
            return Err(FileError::SizeMismatch);
        }
        if self.expected_digest != digest {
            return Err(FileError::DigestMismatch);
        }
        self.transition_to(UploadState::Completing)
    }

    pub fn complete(&mut self) -> FileResult<()> {
        self.transition_to(UploadState::Completed)
    }

    pub fn abort(&mut self) -> FileResult<()> {
        self.transition_to(UploadState::Aborted)
    }

    pub fn expire(&mut self) -> FileResult<()> {
        self.transition_to(UploadState::Expired)
    }

    pub fn reconcile_abort(&mut self) -> FileResult<()> {
        if self.state != UploadState::Completing {
            return Err(FileError::InvalidUploadTransition {
                from: self.state,
                to: UploadState::Aborted,
            });
        }
        self.transition_to(UploadState::Aborted)
    }

    fn transition_to(&mut self, next: UploadState) -> FileResult<()> {
        if can_transition(self.state, next) {
            self.state = next;
            Ok(())
        } else {
            Err(FileError::InvalidUploadTransition { from: self.state, to: next })
        }
    }
}

fn can_transition(from: UploadState, to: UploadState) -> bool {
    matches!(
        (from, to),
        (UploadState::Open, UploadState::Completing | UploadState::Aborted | UploadState::Expired)
            | (UploadState::Completing, UploadState::Completed | UploadState::Aborted)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_requires_exact_size_before_completion() {
        let mut lifecycle = UploadLifecycle::new(crate::domain::ByteSize::from_bytes(3), crate::domain::ContentDigest::from_bytes(b"abc"));
        lifecycle.record_part(crate::domain::ByteSize::from_bytes(2)).unwrap();

        assert_eq!(
            lifecycle.begin_completion(crate::domain::ContentDigest::from_bytes(b"abc")).unwrap_err(),
            FileError::SizeMismatch
        );
        assert_eq!(lifecycle.state(), UploadState::Open);
    }

    #[test]
    fn lifecycle_rejects_writes_after_abort() {
        let mut lifecycle = UploadLifecycle::new(crate::domain::ByteSize::from_bytes(1), crate::domain::ContentDigest::from_bytes(b"a"));
        lifecycle.abort().unwrap();

        assert_eq!(
            lifecycle.record_part(crate::domain::ByteSize::from_bytes(1)).unwrap_err(),
            FileError::InvalidUploadTransition {
                from: UploadState::Aborted,
                to: UploadState::Open,
            }
        );
    }

    #[test]
    fn completion_has_an_explicit_intermediate_state() {
        let mut lifecycle = UploadLifecycle::new(crate::domain::ByteSize::from_bytes(1), crate::domain::ContentDigest::from_bytes(b"a"));
        lifecycle.record_part(crate::domain::ByteSize::from_bytes(1)).unwrap();
        lifecycle.begin_completion(crate::domain::ContentDigest::from_bytes(b"a")).unwrap();
        assert_eq!(lifecycle.state(), UploadState::Completing);
        lifecycle.complete().unwrap();
        assert_eq!(lifecycle.state(), UploadState::Completed);
    }
}
