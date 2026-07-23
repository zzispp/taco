use axum::http::{HeaderMap, header};

use crate::application::RequestedByteRange;
use crate::error::keys;
use crate::{FileError, FileResult};

pub fn parse_read_range(headers: &HeaderMap) -> FileResult<Option<RequestedByteRange>> {
    let Some(value) = headers.get(header::RANGE) else {
        return Ok(None);
    };
    let value = value.to_str().map_err(|_| FileError::InvalidInput(keys::RANGE_HEADER_INVALID))?.trim();
    let value = value.strip_prefix("bytes=").ok_or(FileError::RangeNotSatisfiable)?;
    if value.contains(',') {
        return Err(FileError::RangeNotSatisfiable);
    }
    let (start, end) = value.split_once('-').ok_or(FileError::RangeNotSatisfiable)?;
    parse_single_range(start, end).map(Some)
}

fn parse_single_range(start: &str, end: &str) -> FileResult<RequestedByteRange> {
    if start.is_empty() {
        let length = parse_number(end)?;
        return Ok(RequestedByteRange::Suffix { length });
    }
    let start = parse_number(start)?;
    if end.is_empty() {
        return Ok(RequestedByteRange::From { start });
    }
    Ok(RequestedByteRange::Inclusive {
        start,
        end: parse_number(end)?,
    })
}

fn parse_number(value: &str) -> FileResult<u64> {
    value.parse().map_err(|_| FileError::RangeNotSatisfiable)
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue, header};

    use super::parse_read_range;
    use crate::application::RequestedByteRange;

    #[test]
    fn accepts_open_ended_suffix_and_bounded_ranges() {
        assert_eq!(parse("bytes=0-").unwrap(), Some(RequestedByteRange::From { start: 0 }));
        assert_eq!(parse("bytes=-500").unwrap(), Some(RequestedByteRange::Suffix { length: 500 }));
        assert_eq!(parse("bytes=4-9").unwrap(), Some(RequestedByteRange::Inclusive { start: 4, end: 9 }));
    }

    #[test]
    fn rejects_multiple_or_empty_ranges() {
        assert!(parse("bytes=0-1,2-3").is_err());
        assert!(parse("bytes=-").is_err());
    }

    fn parse(value: &str) -> crate::FileResult<Option<RequestedByteRange>> {
        let mut headers = HeaderMap::new();
        headers.insert(header::RANGE, HeaderValue::from_str(value).unwrap());
        parse_read_range(&headers)
    }
}
