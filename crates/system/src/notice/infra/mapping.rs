use storage::{StorageError, StorageResult};
use time::OffsetDateTime;

use crate::notice::{Notice, NoticeReader, NoticeSummary, NoticeTopItem};

use super::records::{NoticeReaderRecord, NoticeRecord, NoticeSummaryRecord, NoticeTopRecord};

pub(super) fn notice(record: NoticeRecord) -> StorageResult<Notice> {
    Ok(Notice {
        notice_id: record.notice_id,
        notice_title: record.notice_title,
        notice_type: record.notice_type,
        notice_content: record.notice_content,
        status: record.status,
        create_by: record.create_by,
        create_time: wire_time(record.create_time)?,
        update_by: record.update_by,
        update_time: record.update_time.map(wire_time).transpose()?,
        remark: record.remark,
    })
}

pub(super) fn summary(record: NoticeSummaryRecord) -> StorageResult<NoticeSummary> {
    Ok(NoticeSummary {
        notice_id: record.notice_id,
        notice_title: record.notice_title,
        notice_type: record.notice_type,
        status: record.status,
        create_by: record.create_by,
        create_time: wire_time(record.create_time)?,
    })
}

pub(super) fn top(record: NoticeTopRecord) -> StorageResult<NoticeTopItem> {
    Ok(NoticeTopItem {
        notice_id: record.notice_id,
        notice_title: record.notice_title,
        notice_type: record.notice_type,
        create_by: record.create_by,
        create_time: wire_time(record.create_time)?,
        is_read: record.is_read,
    })
}

pub(super) fn reader(record: NoticeReaderRecord) -> StorageResult<NoticeReader> {
    Ok(NoticeReader {
        user_id: record.user_id,
        user_name: record.user_name,
        nick_name: record.nick_name,
        dept_name: record.dept_name,
        phonenumber: record.phonenumber,
        read_time: wire_time(record.read_time)?,
    })
}

fn wire_time(value: OffsetDateTime) -> StorageResult<String> {
    types::http::format_utc_rfc3339_millis(value).map_err(|error| StorageError::Database(error.to_string()))
}

#[cfg(test)]
mod tests {
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};

    use super::wire_time;

    #[test]
    fn notice_wire_time_is_utc_with_fixed_milliseconds() {
        let value = OffsetDateTime::parse("2026-07-15T12:34:56.123456789+08:00", &Rfc3339).unwrap();
        assert_eq!(wire_time(value).unwrap(), "2026-07-15T04:34:56.123Z");
    }
}
