use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::error::keys;
use crate::{FileError, FileResult};

pub(super) fn format_time(value: OffsetDateTime) -> String {
    value.format(&Rfc3339).unwrap_or_else(|_| value.unix_timestamp().to_string())
}

pub(super) fn parse_time(value: &str) -> FileResult<OffsetDateTime> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|_| FileError::InvalidInput(keys::TIME_FILTER_INVALID))
}
