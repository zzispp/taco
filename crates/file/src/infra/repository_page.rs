use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use kernel::pagination::{CursorDirection, CursorPageRequest};
use serde::{Deserialize, Serialize};

use crate::error::keys;
use crate::{FileError, FileResult};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(in crate::infra) struct PageCursor {
    pub(super) sort_value: String,
    pub(super) id: String,
    pub(super) direction: CursorDirection,
    pub(super) fingerprint: String,
    pub(super) limit: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::infra) struct CursorBoundary {
    sort_value: String,
    id: String,
}

impl CursorBoundary {
    pub(in crate::infra) fn new(sort_value: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            sort_value: sort_value.into(),
            id: id.into(),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct FileCursorCodec<'a> {
    fingerprint: &'a str,
    limit: u64,
}

impl<'a> FileCursorCodec<'a> {
    pub(super) const fn new(fingerprint: &'a str, limit: u64) -> Self {
        Self { fingerprint, limit }
    }

    pub(super) fn encode(self, direction: CursorDirection, boundary: &CursorBoundary) -> String {
        let cursor = PageCursor {
            sort_value: boundary.sort_value.clone(),
            id: boundary.id.clone(),
            direction,
            fingerprint: self.fingerprint.to_owned(),
            limit: self.limit,
        };
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&cursor).expect("file cursor serialization is infallible"))
    }

    pub(super) fn decode(self, cursor: &str) -> FileResult<PageCursor> {
        let bytes = URL_SAFE_NO_PAD.decode(cursor).map_err(|_| FileError::InvalidInput(keys::CURSOR_MALFORMED))?;
        let value: PageCursor = serde_json::from_slice(&bytes).map_err(|_| FileError::InvalidInput(keys::CURSOR_MALFORMED))?;
        if value.fingerprint != self.fingerprint || value.limit != self.limit {
            return Err(FileError::InvalidInput(keys::CURSOR_QUERY_MISMATCH));
        }
        Ok(value)
    }
}

pub(in crate::infra) struct FilePageContext<'a> {
    codec: FileCursorCodec<'a>,
    cursor: Option<PageCursor>,
}

pub(in crate::infra) struct FilePageSlice<R> {
    pub(in crate::infra) records: Vec<R>,
    pub(in crate::infra) next_cursor: Option<String>,
    pub(in crate::infra) previous_cursor: Option<String>,
}

impl<'a> FilePageContext<'a> {
    pub(in crate::infra) fn new(cursor: Option<&str>, fingerprint: &'a str, page: &CursorPageRequest) -> FileResult<Self> {
        let codec = FileCursorCodec::new(fingerprint, page.limit);
        let cursor = cursor.map(|value| codec.decode(value)).transpose()?;
        Ok(Self { codec, cursor })
    }

    pub(in crate::infra) fn cursor(&self) -> Option<&PageCursor> {
        self.cursor.as_ref()
    }

    pub(in crate::infra) fn direction(&self) -> CursorDirection {
        self.cursor.as_ref().map_or(CursorDirection::Next, |cursor| cursor.direction)
    }

    pub(in crate::infra) fn query_limit(&self) -> FileResult<i64> {
        let limit = self.codec.limit.checked_add(1).ok_or(FileError::InvalidInput(keys::CURSOR_LIMIT_TOO_LARGE))?;
        i64::try_from(limit).map_err(|_| FileError::InvalidInput(keys::CURSOR_LIMIT_TOO_LARGE))
    }

    pub(in crate::infra) fn build_page<R, F>(&self, mut records: Vec<R>, boundary: F) -> FileResult<FilePageSlice<R>>
    where
        F: Fn(&R) -> FileResult<CursorBoundary>,
    {
        let requested = usize::try_from(self.codec.limit).map_err(|_| FileError::InvalidInput(keys::CURSOR_LIMIT_TOO_LARGE))?;
        let has_extra = records.len() > requested;
        records.truncate(requested);
        if self.direction() == CursorDirection::Previous {
            records.reverse();
        }
        let (next_cursor, previous_cursor) = self.page_cursors(&records, has_extra, boundary)?;
        Ok(FilePageSlice {
            records,
            next_cursor,
            previous_cursor,
        })
    }

    fn page_cursors<R, F>(&self, records: &[R], has_extra: bool, boundary: F) -> FileResult<(Option<String>, Option<String>)>
    where
        F: Fn(&R) -> FileResult<CursorBoundary>,
    {
        let Some(first) = records.first() else {
            return Ok(self.empty_page_cursors());
        };
        let last = records.last().expect("a non-empty file cursor page has a last record");
        let from_cursor = self.cursor.is_some();
        let has_previous = from_cursor && (self.direction() == CursorDirection::Next || has_extra);
        let has_next = has_extra || (from_cursor && self.direction() == CursorDirection::Previous);
        let next = has_next
            .then(|| boundary(last).map(|value| self.codec.encode(CursorDirection::Next, &value)))
            .transpose()?;
        let previous = has_previous
            .then(|| boundary(first).map(|value| self.codec.encode(CursorDirection::Previous, &value)))
            .transpose()?;
        Ok((next, previous))
    }

    fn empty_page_cursors(&self) -> (Option<String>, Option<String>) {
        let Some(cursor) = &self.cursor else {
            return (None, None);
        };
        let boundary = CursorBoundary::new(cursor.sort_value.clone(), cursor.id.clone());
        match cursor.direction {
            CursorDirection::Next => (None, Some(self.codec.encode(CursorDirection::Previous, &boundary))),
            CursorDirection::Previous => (Some(self.codec.encode(CursorDirection::Next, &boundary)), None),
        }
    }
}

#[cfg(test)]
#[path = "repository_page_tests.rs"]
mod tests;
