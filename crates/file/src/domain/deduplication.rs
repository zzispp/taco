use std::collections::{BTreeMap, BTreeSet};

use crate::FileError;
use crate::domain::{ContentDigest, SpaceId, StoredObjectId};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ContentReuseScope {
    spaces: BTreeSet<SpaceId>,
}

impl ContentReuseScope {
    pub fn from_spaces(spaces: impl IntoIterator<Item = SpaceId>) -> Self {
        Self {
            spaces: spaces.into_iter().collect(),
        }
    }

    pub fn includes(&self, space_id: &SpaceId) -> bool {
        self.spaces.contains(space_id)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeduplicationDecision {
    Store,
    Reuse(StoredObjectId),
}

#[derive(Clone, Debug)]
struct StoredObjectCandidate {
    object_id: StoredObjectId,
    size: crate::domain::ByteSize,
    visible_spaces: BTreeSet<SpaceId>,
}

#[derive(Clone, Debug, Default)]
pub struct DeduplicationIndex {
    objects: BTreeMap<ContentDigest, Vec<StoredObjectCandidate>>,
}

impl DeduplicationIndex {
    pub fn decide(&self, digest: ContentDigest, size: crate::domain::ByteSize, scope: &ContentReuseScope) -> DeduplicationDecision {
        self.objects
            .get(&digest)
            .into_iter()
            .flatten()
            .find(|candidate| candidate.size == size && candidate.visible_spaces.iter().any(|space| scope.includes(space)))
            .map_or(DeduplicationDecision::Store, |candidate| DeduplicationDecision::Reuse(candidate.object_id))
    }

    pub fn register(&mut self, object_id: StoredObjectId, digest: ContentDigest, size: crate::domain::ByteSize, space_id: SpaceId) -> Result<(), FileError> {
        let candidates = self.objects.entry(digest).or_default();
        if candidates.iter().any(|candidate| candidate.object_id == object_id) {
            return Err(FileError::NameConflict);
        }
        candidates.push(StoredObjectCandidate {
            object_id,
            size,
            visible_spaces: BTreeSet::from([space_id]),
        });
        Ok(())
    }

    pub fn add_visible_space(&mut self, object_id: StoredObjectId, space_id: SpaceId) -> Result<(), FileError> {
        let candidate = self
            .objects
            .values_mut()
            .flatten()
            .find(|candidate| candidate.object_id == object_id)
            .ok_or(FileError::NotFound)?;
        candidate.visible_spaces.insert(space_id);
        Ok(())
    }

    pub fn remove(&mut self, object_id: StoredObjectId) -> bool {
        let mut removed = false;
        for candidates in self.objects.values_mut() {
            let before = candidates.len();
            candidates.retain(|candidate| candidate.object_id != object_id);
            removed |= before != candidates.len();
        }
        self.objects.retain(|_, candidates| !candidates.is_empty());
        removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_reuse_is_limited_to_visible_spaces() {
        let mut index = DeduplicationIndex::default();
        let source_space = SpaceId::new("source-user").unwrap();
        let other_space = SpaceId::new("other-user").unwrap();
        let digest = ContentDigest::from_bytes(b"same");
        let object = StoredObjectId::new();
        index
            .register(object, digest, crate::domain::ByteSize::from_bytes(4), source_space.clone())
            .unwrap();

        assert_eq!(
            index.decide(
                digest,
                crate::domain::ByteSize::from_bytes(4),
                &ContentReuseScope::from_spaces([source_space.clone()])
            ),
            DeduplicationDecision::Reuse(object)
        );
        assert_eq!(
            index.decide(digest, crate::domain::ByteSize::from_bytes(4), &ContentReuseScope::from_spaces([other_space])),
            DeduplicationDecision::Store
        );
    }
}
