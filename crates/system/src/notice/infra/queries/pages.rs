use kernel::pagination::{CursorDirection, CursorPage};
use sqlx::{Postgres, QueryBuilder};
use storage::{Database, StorageError, database::to_i64};
use time::OffsetDateTime;

use crate::{
    application::{SystemError, SystemResult, TimeIdPoint, point, point_time},
    notice::{
        NoticeListFilter, NoticeReader, NoticeReaderFilter, NoticeSummary,
        cursor::{NoticeCursorCodec, NoticeDecodedCursor},
    },
};

use super::super::{
    mapping::{reader, summary},
    records::{NoticeReaderRecord, NoticeSummaryRecord},
};
use crate::notice::infra::map_storage_error;

mod filters;
use filters::{notice_query, reader_query};

struct PageWindow {
    snapshot_time: OffsetDateTime,
    snapshot_id: String,
    boundary_time: Option<OffsetDateTime>,
    boundary_id: Option<String>,
    direction: CursorDirection,
    limit: u64,
    from_cursor: bool,
}

struct WindowColumns<'a> {
    time: &'a str,
    id: &'a str,
}

struct PageContext<'a> {
    codec: &'a NoticeCursorCodec,
    snapshot: &'a TimeIdPoint,
    window: &'a PageWindow,
}

struct ReaderSnapshotQuery<'a> {
    notice_id: &'a str,
    filter: &'a NoticeReaderFilter,
    decoded: Option<&'a NoticeDecodedCursor>,
}

trait PageRecord: Sized {
    type Item;

    fn point(&self) -> SystemResult<TimeIdPoint>;
    fn into_item(self) -> SystemResult<Self::Item>;
}

pub(super) async fn page_notices(database: &Database, filter: NoticeListFilter) -> SystemResult<CursorPage<NoticeSummary>> {
    let codec = NoticeCursorCodec::notices(&filter)?;
    let decoded = codec.decode(filter.page.cursor.as_deref())?;
    let snapshot = notice_snapshot(database, &filter, decoded.as_ref()).await?;
    let Some(snapshot) = snapshot else {
        return Ok(CursorPage::new(Vec::new(), None, None));
    };
    let window = page_window(decoded.as_ref(), &snapshot, filter.page.limit)?;
    let mut query = notice_query(&filter);
    push_window(
        &mut query,
        &window,
        WindowColumns {
            time: "create_time",
            id: "notice_id",
        },
    )?;
    let records = fetch::<NoticeSummaryRecord>(database, query).await?;
    build_page(
        records,
        PageContext {
            codec: &codec,
            snapshot: &snapshot,
            window: &window,
        },
    )
}

pub(super) async fn page_readers(database: &Database, notice_id: &str, filter: NoticeReaderFilter) -> SystemResult<CursorPage<NoticeReader>> {
    let codec = NoticeCursorCodec::readers(notice_id, &filter)?;
    let decoded = codec.decode(filter.page.cursor.as_deref())?;
    let snapshot = reader_snapshot(
        database,
        ReaderSnapshotQuery {
            notice_id,
            filter: &filter,
            decoded: decoded.as_ref(),
        },
    )
    .await?;
    let Some(snapshot) = snapshot else {
        return Ok(CursorPage::new(Vec::new(), None, None));
    };
    let window = page_window(decoded.as_ref(), &snapshot, filter.page.limit)?;
    let mut query = reader_query(notice_id, &filter);
    push_window(
        &mut query,
        &window,
        WindowColumns {
            time: "r.read_time",
            id: "r.user_id",
        },
    )?;
    let records = fetch::<NoticeReaderRecord>(database, query).await?;
    build_page(
        records,
        PageContext {
            codec: &codec,
            snapshot: &snapshot,
            window: &window,
        },
    )
}

async fn notice_snapshot(database: &Database, filter: &NoticeListFilter, decoded: Option<&NoticeDecodedCursor>) -> SystemResult<Option<TimeIdPoint>> {
    let mut query = notice_query(filter);
    query.push(" ORDER BY create_time DESC,notice_id DESC LIMIT 1");
    fetch_snapshot::<NoticeSummaryRecord>(database, query, decoded).await
}

async fn reader_snapshot(database: &Database, request: ReaderSnapshotQuery<'_>) -> SystemResult<Option<TimeIdPoint>> {
    if let Some(cursor) = request.decoded {
        return Ok(Some(cursor.snapshot.clone()));
    }
    let mut query = reader_query(request.notice_id, request.filter);
    query.push(" ORDER BY r.read_time DESC,r.user_id DESC LIMIT 1");
    fetch_snapshot::<NoticeReaderRecord>(database, query, None).await
}

async fn fetch_snapshot<R>(database: &Database, mut query: QueryBuilder<Postgres>, decoded: Option<&NoticeDecodedCursor>) -> SystemResult<Option<TimeIdPoint>>
where
    R: PageRecord + for<'row> sqlx::FromRow<'row, sqlx::postgres::PgRow> + Send + Unpin,
{
    if let Some(cursor) = decoded {
        return Ok(Some(cursor.snapshot.clone()));
    }
    query
        .build_query_as::<R>()
        .fetch_optional(database.pool())
        .await
        .map_err(StorageError::from)
        .map_err(map_storage_error)?
        .map(|record| record.point())
        .transpose()
}

async fn fetch<R>(database: &Database, mut query: QueryBuilder<Postgres>) -> SystemResult<Vec<R>>
where
    R: for<'row> sqlx::FromRow<'row, sqlx::postgres::PgRow> + Send + Unpin,
{
    query
        .build_query_as::<R>()
        .fetch_all(database.pool())
        .await
        .map_err(StorageError::from)
        .map_err(map_storage_error)
}

fn page_window(decoded: Option<&NoticeDecodedCursor>, snapshot: &TimeIdPoint, limit: u64) -> SystemResult<PageWindow> {
    let boundary = decoded.map(|cursor| &cursor.boundary);
    Ok(PageWindow {
        snapshot_time: point_time(snapshot)?,
        snapshot_id: snapshot.id.clone(),
        boundary_time: boundary.map(point_time).transpose()?,
        boundary_id: boundary.map(|point| point.id.clone()),
        direction: decoded.map_or(CursorDirection::Next, |cursor| cursor.direction),
        limit,
        from_cursor: decoded.is_some(),
    })
}

fn push_window(query: &mut QueryBuilder<Postgres>, window: &PageWindow, columns: WindowColumns<'_>) -> SystemResult<()> {
    query.push(" AND (").push(columns.time).push(",").push(columns.id).push(")<=(");
    query.push_bind(window.snapshot_time).push(",").push_bind(window.snapshot_id.clone()).push(")");
    if let (Some(time), Some(id)) = (window.boundary_time, &window.boundary_id) {
        let operator = if window.direction == CursorDirection::Next { "<" } else { ">" };
        query
            .push(" AND (")
            .push(columns.time)
            .push(",")
            .push(columns.id)
            .push(")")
            .push(operator)
            .push("(");
        query.push_bind(time).push(",").push_bind(id.clone()).push(")");
    }
    let order = if window.direction == CursorDirection::Next { "DESC" } else { "ASC" };
    query
        .push(" ORDER BY ")
        .push(columns.time)
        .push(" ")
        .push(order)
        .push(",")
        .push(columns.id)
        .push(" ")
        .push(order);
    let fetch_limit = window.limit.checked_add(1).ok_or_else(|| numeric_error("cursor limit overflow"))?;
    query.push(" LIMIT ").push_bind(to_i64(fetch_limit).map_err(numeric_error)?);
    Ok(())
}

fn build_page<R: PageRecord>(mut records: Vec<R>, context: PageContext<'_>) -> SystemResult<CursorPage<R::Item>> {
    let requested = usize::try_from(context.window.limit).map_err(numeric_error)?;
    let has_extra = records.len() > requested;
    records.truncate(requested);
    if context.window.direction == CursorDirection::Previous {
        records.reverse();
    }
    let (next, previous) = page_cursors(&records, &context, has_extra)?;
    let items = records.into_iter().map(PageRecord::into_item).collect::<SystemResult<Vec<_>>>()?;
    Ok(CursorPage::new(items, next, previous))
}

fn page_cursors<R: PageRecord>(records: &[R], context: &PageContext<'_>, has_extra: bool) -> SystemResult<(Option<String>, Option<String>)> {
    let Some(first) = records.first() else { return empty_cursors(context) };
    let last = records.last().expect("non-empty notice cursor page has a last row");
    let has_previous = context.window.from_cursor && (context.window.direction == CursorDirection::Next || has_extra);
    let has_next = has_extra || (context.window.from_cursor && context.window.direction == CursorDirection::Previous);
    let next = has_next
        .then(|| context.codec.encode(CursorDirection::Next, &last.point()?, context.snapshot))
        .transpose()?;
    let previous = has_previous
        .then(|| context.codec.encode(CursorDirection::Previous, &first.point()?, context.snapshot))
        .transpose()?;
    Ok((next, previous))
}

fn empty_cursors(context: &PageContext<'_>) -> SystemResult<(Option<String>, Option<String>)> {
    let Some(boundary) = context.window.boundary_time.zip(context.window.boundary_id.clone()) else {
        return Ok((None, None));
    };
    let boundary = point(boundary.0, boundary.1)?;
    match context.window.direction {
        CursorDirection::Next => Ok((None, Some(context.codec.encode(CursorDirection::Previous, &boundary, context.snapshot)?))),
        CursorDirection::Previous => Ok((Some(context.codec.encode(CursorDirection::Next, &boundary, context.snapshot)?), None)),
    }
}

impl PageRecord for NoticeSummaryRecord {
    type Item = NoticeSummary;

    fn point(&self) -> SystemResult<TimeIdPoint> {
        point(self.create_time, self.notice_id.clone())
    }

    fn into_item(self) -> SystemResult<Self::Item> {
        summary(self).map_err(map_storage_error)
    }
}

impl PageRecord for NoticeReaderRecord {
    type Item = NoticeReader;

    fn point(&self) -> SystemResult<TimeIdPoint> {
        point(self.read_time, self.user_id.clone())
    }

    fn into_item(self) -> SystemResult<Self::Item> {
        reader(self).map_err(map_storage_error)
    }
}

fn numeric_error(error: impl std::fmt::Display) -> SystemError {
    SystemError::Infrastructure(format!("notice cursor numeric conversion failed: {error}"))
}

#[cfg(test)]
mod tests;
