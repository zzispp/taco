use kernel::pagination::{CursorContext, CursorDecodeError, CursorDirection, CursorEncodeError, DecodedCursor, cursor_fingerprint, decode_cursor};
use serde_json::{Value, json};

use crate::application::{SystemError, SystemResult, TimeIdPoint, point_time};

use super::{NoticeListFilter, NoticeReaderFilter};

const NOTICE_RESOURCE: &str = "system.notices";
const NOTICE_SORT: &str = "create_time:desc,notice_id:desc";
const READER_RESOURCE: &str = "system.notice_readers";
const READER_SORT: &str = "read_time:desc,user_id:desc";

pub(super) type NoticeDecodedCursor = DecodedCursor<TimeIdPoint, TimeIdPoint>;

pub(super) struct NoticeCursorCodec {
    resource: &'static str,
    sort: &'static str,
    filter_fingerprint: String,
    scope_fingerprint: String,
    limit: u64,
}

struct NoticeCursorSpec {
    resource: &'static str,
    sort: &'static str,
    filter: Value,
    scope: Value,
    limit: u64,
}

impl NoticeCursorCodec {
    pub(super) fn notices(filter: &NoticeListFilter) -> SystemResult<Self> {
        Self::new(NoticeCursorSpec {
            resource: NOTICE_RESOURCE,
            sort: NOTICE_SORT,
            filter: json!({
                "notice_title": filter.notice_title,
                "create_by": filter.create_by,
                "notice_type": filter.notice_type,
            }),
            scope: json!({"mode": "global"}),
            limit: filter.page.limit,
        })
    }

    pub(super) fn readers(notice_id: &str, filter: &NoticeReaderFilter) -> SystemResult<Self> {
        Self::new(NoticeCursorSpec {
            resource: READER_RESOURCE,
            sort: READER_SORT,
            filter: json!({"search_value": filter.search_value}),
            scope: json!({"notice_id": notice_id}),
            limit: filter.page.limit,
        })
    }

    pub(super) fn decode(&self, cursor: Option<&str>) -> SystemResult<Option<NoticeDecodedCursor>> {
        cursor
            .map(|value| decode_cursor(value, &self.context()).map_err(decode_error))
            .transpose()?
            .map(validate_decoded)
            .transpose()
    }

    pub(super) fn encode(&self, direction: CursorDirection, boundary: &TimeIdPoint, snapshot: &TimeIdPoint) -> SystemResult<String> {
        self.context().encode(direction, boundary, snapshot).map_err(encode_error)
    }

    fn new(spec: NoticeCursorSpec) -> SystemResult<Self> {
        Ok(Self {
            resource: spec.resource,
            sort: spec.sort,
            filter_fingerprint: fingerprint(&spec.filter)?,
            scope_fingerprint: fingerprint(&spec.scope)?,
            limit: spec.limit,
        })
    }

    fn context(&self) -> CursorContext<'_> {
        CursorContext {
            resource: self.resource,
            sort: self.sort,
            filter_fingerprint: &self.filter_fingerprint,
            scope_fingerprint: &self.scope_fingerprint,
            limit: self.limit,
        }
    }
}

fn validate_decoded(cursor: NoticeDecodedCursor) -> SystemResult<NoticeDecodedCursor> {
    if cursor.boundary.id.trim().is_empty() || cursor.snapshot.id.trim().is_empty() {
        return Err(SystemError::InvalidCursor);
    }
    point_time(&cursor.boundary)?;
    point_time(&cursor.snapshot)?;
    if (cursor.boundary.time_micros, cursor.boundary.id.as_str()) > (cursor.snapshot.time_micros, cursor.snapshot.id.as_str()) {
        return Err(SystemError::InvalidCursor);
    }
    Ok(cursor)
}

fn fingerprint(value: &Value) -> SystemResult<String> {
    cursor_fingerprint(value).map_err(encode_error)
}

fn encode_error(error: CursorEncodeError) -> SystemError {
    SystemError::Infrastructure(error.to_string())
}

fn decode_error(_error: CursorDecodeError) -> SystemError {
    SystemError::InvalidCursor
}

#[cfg(test)]
mod tests;
