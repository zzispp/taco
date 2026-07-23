use crate::error::keys;
use crate::{FileError, FileResult};
use serde::{Deserialize, Serialize};
use std::fmt;

const ZERO_BYTES: u64 = 0;

#[derive(Clone, Copy, Debug, Default, Deserialize, Hash, PartialEq, Eq, Ord, PartialOrd, Serialize)]
pub struct ByteSize(u64);

impl ByteSize {
    pub const ZERO: Self = Self(ZERO_BYTES);

    pub const fn from_bytes(bytes: u64) -> Self {
        Self(bytes)
    }

    pub const fn bytes(self) -> u64 {
        self.0
    }

    pub fn checked_add(self, other: Self) -> FileResult<Self> {
        self.0.checked_add(other.0).map(Self).ok_or(FileError::SizeMismatch)
    }
}

impl fmt::Display for ByteSize {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.bytes().fmt(formatter)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum ProviderCapacity {
    Bounded { total_bytes: ByteSize, available_bytes: ByteSize },
    UsageBased { used_bytes: ByteSize },
}

pub type Capacity = ProviderCapacity;

impl ProviderCapacity {
    pub fn bounded(total_bytes: ByteSize, available_bytes: ByteSize) -> FileResult<Self> {
        if available_bytes > total_bytes {
            return Err(FileError::InvalidInput(keys::AVAILABLE_CAPACITY_EXCEEDS_TOTAL));
        }
        Ok(Self::Bounded { total_bytes, available_bytes })
    }

    pub const fn total(self) -> Option<ByteSize> {
        match self {
            Self::Bounded { total_bytes, .. } => Some(total_bytes),
            Self::UsageBased { .. } => None,
        }
    }

    pub const fn available(self) -> Option<ByteSize> {
        match self {
            Self::Bounded { available_bytes, .. } => Some(available_bytes),
            Self::UsageBased { .. } => None,
        }
    }

    pub const fn used(self) -> ByteSize {
        match self {
            Self::Bounded { total_bytes, available_bytes } => ByteSize::from_bytes(total_bytes.bytes() - available_bytes.bytes()),
            Self::UsageBased { used_bytes } => used_bytes,
        }
    }

    pub fn ensure_available(self, requested: ByteSize) -> FileResult<()> {
        let Some(available) = self.available() else {
            return Ok(());
        };
        if requested > available {
            return Err(FileError::CapacityExceeded { requested, available });
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quota {
    limit: Option<ByteSize>,
    used: ByteSize,
}

impl Quota {
    pub const fn unlimited() -> Self {
        Self {
            limit: None,
            used: ByteSize::ZERO,
        }
    }

    pub const fn with_limit(limit: ByteSize) -> Self {
        Self {
            limit: Some(limit),
            used: ByteSize::ZERO,
        }
    }

    pub const fn limit(self) -> Option<ByteSize> {
        self.limit
    }

    pub const fn used(self) -> ByteSize {
        self.used
    }

    pub fn is_over_quota(self) -> bool {
        matches!(self.limit, Some(limit) if self.used > limit)
    }

    pub fn set_limit(&mut self, limit: Option<ByteSize>) {
        self.limit = limit;
    }

    pub fn remaining(self) -> Option<ByteSize> {
        self.limit.map(|limit| ByteSize::from_bytes(limit.bytes().saturating_sub(self.used.bytes())))
    }

    pub fn reserve(&mut self, requested: ByteSize) -> FileResult<()> {
        if let Some(remaining) = self.remaining()
            && requested > remaining
        {
            return Err(FileError::QuotaExceeded { requested, remaining });
        }
        self.used = self.used.checked_add(requested)?;
        Ok(())
    }

    pub fn release(&mut self, released: ByteSize) -> FileResult<()> {
        if released > self.used {
            return Err(FileError::InvalidInput(keys::QUOTA_RELEASE_EXCEEDS_USAGE));
        }
        self.used = ByteSize::from_bytes(self.used.bytes() - released.bytes());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_rejects_requests_larger_than_available_space() {
        let capacity = Capacity::bounded(ByteSize::from_bytes(10), ByteSize::from_bytes(4)).unwrap();

        let error = capacity.ensure_available(ByteSize::from_bytes(5)).unwrap_err();

        assert_eq!(
            error,
            FileError::CapacityExceeded {
                requested: ByteSize::from_bytes(5),
                available: ByteSize::from_bytes(4),
            }
        );
    }

    #[test]
    fn usage_based_capacity_does_not_fabricate_a_total() {
        let capacity = Capacity::UsageBased {
            used_bytes: ByteSize::from_bytes(7),
        };

        assert_eq!(capacity.total(), None);
        assert_eq!(capacity.available(), None);
        capacity.ensure_available(ByteSize::from_bytes(100)).unwrap();
    }

    #[test]
    fn quota_tracks_reservations_and_releases() {
        let mut quota = Quota::with_limit(ByteSize::from_bytes(10));
        quota.reserve(ByteSize::from_bytes(6)).unwrap();
        assert_eq!(quota.remaining(), Some(ByteSize::from_bytes(4)));

        quota.release(ByteSize::from_bytes(2)).unwrap();

        assert_eq!(quota.used(), ByteSize::from_bytes(4));
        assert_eq!(quota.remaining(), Some(ByteSize::from_bytes(6)));
    }

    #[test]
    fn lowering_a_quota_exposes_over_quota_without_discarding_usage() {
        let mut quota = Quota::with_limit(ByteSize::from_bytes(10));
        quota.reserve(ByteSize::from_bytes(8)).unwrap();
        quota.set_limit(Some(ByteSize::from_bytes(4)));

        assert!(quota.is_over_quota());
        assert_eq!(
            quota.reserve(ByteSize::from_bytes(1)).unwrap_err(),
            FileError::QuotaExceeded {
                requested: ByteSize::from_bytes(1),
                remaining: ByteSize::ZERO,
            }
        );
    }
}
