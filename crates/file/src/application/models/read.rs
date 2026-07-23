use crate::application::{ByteRange, ObjectStream};
use crate::domain::{ByteSize, FileId};
use crate::{FileError, FileResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestedByteRange {
    Inclusive { start: u64, end: u64 },
    From { start: u64 },
    Suffix { length: u64 },
}

impl RequestedByteRange {
    pub fn resolve(self, size: ByteSize) -> FileResult<ByteRange> {
        let size = size.bytes();
        if size == 0 {
            return Err(FileError::RangeNotSatisfiable);
        }
        match self {
            Self::Inclusive { start, end } => resolve_inclusive(start, end, size),
            Self::From { start } => ByteRange::new(start, size).and_then(|range| range.within(ByteSize::from_bytes(size))),
            Self::Suffix { length } => resolve_suffix(length, size),
        }
    }
}

fn resolve_inclusive(start: u64, end: u64, size: u64) -> FileResult<ByteRange> {
    if start > end || start >= size {
        return Err(FileError::RangeNotSatisfiable);
    }
    let end_exclusive = if end >= size { size } else { end + 1 };
    ByteRange::new(start, end_exclusive)
}

fn resolve_suffix(length: u64, size: u64) -> FileResult<ByteRange> {
    if length == 0 {
        return Err(FileError::RangeNotSatisfiable);
    }
    ByteRange::new(size.saturating_sub(length), size)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileContentMetadata {
    pub name: String,
    pub content_type: String,
    pub size: ByteSize,
    pub range: Option<ByteRange>,
    pub truncated: bool,
    pub accept_ranges: bool,
}

impl FileContentMetadata {
    pub fn response_size(&self) -> u64 {
        let selected = self.range.map_or(self.size.bytes(), ByteRange::byte_len);
        if self.truncated {
            selected.min(super::super::preview::TEXT_PREVIEW_MAX_BYTES)
        } else {
            selected
        }
    }
}

pub struct FileContent {
    pub metadata: FileContentMetadata,
    pub body: ObjectStream,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileReadRequest {
    pub id: FileId,
    pub range: Option<RequestedByteRange>,
}

#[cfg(test)]
mod tests {
    use super::RequestedByteRange;
    use crate::domain::ByteSize;

    #[test]
    fn requested_ranges_resolve_against_object_size() {
        let size = ByteSize::from_bytes(100);

        assert_eq!(RequestedByteRange::From { start: 20 }.resolve(size).unwrap().byte_len(), 80);
        assert_eq!(RequestedByteRange::Suffix { length: 20 }.resolve(size).unwrap().start(), 80);
        assert_eq!(RequestedByteRange::Suffix { length: 200 }.resolve(size).unwrap().start(), 0);
        assert_eq!(
            RequestedByteRange::Inclusive { start: 20, end: 200 }.resolve(size).unwrap().end_exclusive(),
            100
        );
    }

    #[test]
    fn requested_ranges_reject_empty_or_out_of_bounds_ranges() {
        let size = ByteSize::from_bytes(100);

        assert!(RequestedByteRange::Suffix { length: 0 }.resolve(size).is_err());
        assert!(RequestedByteRange::From { start: 100 }.resolve(size).is_err());
        assert!(RequestedByteRange::Inclusive { start: 5, end: 4 }.resolve(size).is_err());
    }
}
