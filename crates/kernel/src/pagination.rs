use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use utoipa::ToSchema;

pub const DEFAULT_CURSOR_LIMIT: u64 = 20;
pub const MIN_CURSOR_LIMIT: u64 = 1;
pub const MAX_CURSOR_LIMIT: u64 = 100;
const CURSOR_PROTOCOL_VERSION: u8 = 1;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, ToSchema)]
pub struct CursorPageRequest {
    #[serde(default = "default_cursor_limit")]
    #[schema(default = 20, minimum = 1, maximum = 100)]
    pub limit: u64,
    #[serde(default)]
    pub cursor: Option<String>,
}

impl Default for CursorPageRequest {
    fn default() -> Self {
        Self {
            limit: DEFAULT_CURSOR_LIMIT,
            cursor: None,
        }
    }
}

impl CursorPageRequest {
    pub fn validate(&self) -> Result<(), CursorRequestError> {
        if (MIN_CURSOR_LIMIT..=MAX_CURSOR_LIMIT).contains(&self.limit) {
            return Ok(());
        }
        Err(CursorRequestError::InvalidLimit)
    }
}

const fn default_cursor_limit() -> u64 {
    DEFAULT_CURSOR_LIMIT
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum CursorRequestError {
    #[error("cursor page limit must be between {MIN_CURSOR_LIMIT} and {MAX_CURSOR_LIMIT}")]
    InvalidLimit,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, ToSchema)]
pub struct CursorPage<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
    pub previous_cursor: Option<String>,
    pub has_next: bool,
    pub has_previous: bool,
}

impl<T> CursorPage<T> {
    pub fn new(items: Vec<T>, next_cursor: Option<String>, previous_cursor: Option<String>) -> Self {
        Self {
            items,
            has_next: next_cursor.is_some(),
            has_previous: previous_cursor.is_some(),
            next_cursor,
            previous_cursor,
        }
    }

    pub fn map<U>(self, mapper: impl FnMut(T) -> U) -> CursorPage<U> {
        CursorPage {
            items: self.items.into_iter().map(mapper).collect(),
            next_cursor: self.next_cursor,
            previous_cursor: self.previous_cursor,
            has_next: self.has_next,
            has_previous: self.has_previous,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CursorContext<'a> {
    pub resource: &'a str,
    pub sort: &'a str,
    pub filter_fingerprint: &'a str,
    pub scope_fingerprint: &'a str,
    pub limit: u64,
}

impl CursorContext<'_> {
    pub fn encode<B, S>(&self, direction: CursorDirection, boundary: &B, snapshot: &S) -> Result<String, CursorEncodeError>
    where
        B: Serialize,
        S: Serialize,
    {
        let envelope = CursorEnvelopeRef {
            version: CURSOR_PROTOCOL_VERSION,
            resource: self.resource,
            sort: self.sort,
            direction,
            boundary,
            filter_fingerprint: self.filter_fingerprint,
            scope_fingerprint: self.scope_fingerprint,
            limit: self.limit,
            snapshot,
        };
        Ok(URL_SAFE_NO_PAD.encode(serde_json::to_vec(&envelope)?))
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CursorDirection {
    Next,
    Previous,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedCursor<B, S> {
    pub direction: CursorDirection,
    pub boundary: B,
    pub snapshot: S,
}

#[derive(Debug, Error)]
pub enum CursorEncodeError {
    #[error("failed to serialize cursor payload")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum CursorDecodeError {
    #[error("cursor is malformed")]
    Malformed,
    #[error("cursor protocol version is unsupported")]
    UnsupportedVersion,
    #[error("cursor does not match the current request")]
    ContextMismatch,
}

#[derive(Deserialize)]
struct CursorEnvelope<B, S> {
    version: u8,
    resource: String,
    sort: String,
    direction: CursorDirection,
    boundary: B,
    filter_fingerprint: String,
    scope_fingerprint: String,
    limit: u64,
    snapshot: S,
}

#[derive(Serialize)]
struct CursorEnvelopeRef<'a, B, S> {
    version: u8,
    resource: &'a str,
    sort: &'a str,
    direction: CursorDirection,
    boundary: &'a B,
    filter_fingerprint: &'a str,
    scope_fingerprint: &'a str,
    limit: u64,
    snapshot: &'a S,
}

pub fn decode_cursor<B, S>(value: &str, context: &CursorContext<'_>) -> Result<DecodedCursor<B, S>, CursorDecodeError>
where
    B: for<'de> Deserialize<'de>,
    S: for<'de> Deserialize<'de>,
{
    let bytes = URL_SAFE_NO_PAD.decode(value).map_err(|_| CursorDecodeError::Malformed)?;
    let envelope: CursorEnvelope<B, S> = serde_json::from_slice(&bytes).map_err(|_| CursorDecodeError::Malformed)?;
    validate_cursor_context(&envelope, context)?;
    Ok(DecodedCursor {
        direction: envelope.direction,
        boundary: envelope.boundary,
        snapshot: envelope.snapshot,
    })
}

pub fn cursor_fingerprint<T: Serialize>(value: &T) -> Result<String, CursorEncodeError> {
    let canonical = serde_json::to_vec(value)?;
    Ok(URL_SAFE_NO_PAD.encode(Sha256::digest(canonical)))
}

fn validate_cursor_context<B, S>(envelope: &CursorEnvelope<B, S>, context: &CursorContext<'_>) -> Result<(), CursorDecodeError> {
    if envelope.version != CURSOR_PROTOCOL_VERSION {
        return Err(CursorDecodeError::UnsupportedVersion);
    }
    let matches = envelope.resource == context.resource
        && envelope.sort == context.sort
        && envelope.filter_fingerprint == context.filter_fingerprint
        && envelope.scope_fingerprint == context.scope_fingerprint
        && envelope.limit == context.limit;
    if !matches {
        return Err(CursorDecodeError::ContextMismatch);
    }
    Ok(())
}

#[cfg(test)]
#[path = "pagination_tests.rs"]
mod tests;
