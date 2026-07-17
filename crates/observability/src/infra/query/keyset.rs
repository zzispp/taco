use kernel::pagination::CursorDirection;
use sqlx::{Postgres, QueryBuilder};

use crate::{
    application::{ObservabilityError, ObservabilityResult, SystemLogCursorQuery, SystemLogCursorSlice, SystemLogExportSlice, SystemLogSnapshot},
    domain::{SystemLogDetail, SystemLogSummary},
};

pub(super) fn push_snapshot(builder: &mut QueryBuilder<Postgres>, snapshot: &SystemLogSnapshot) -> ObservabilityResult<()> {
    if snapshot.ingested_seq <= 0 {
        return Err(ObservabilityError::InvalidCursor);
    }
    builder.push(" AND ingested_seq <= ").push_bind(snapshot.ingested_seq);
    Ok(())
}

pub(super) fn push_boundary(builder: &mut QueryBuilder<Postgres>, page: &SystemLogCursorQuery) -> ObservabilityResult<()> {
    let Some(boundary) = page.boundary.as_ref() else {
        return Ok(());
    };
    if boundary.id.is_empty() {
        return Err(ObservabilityError::InvalidCursor);
    }
    builder
        .push(" AND (occurred_at,id) ")
        .push(comparison(page.direction))
        .push(" (")
        .push_bind(boundary.occurred_at()?)
        .push(",")
        .push_bind(boundary.id.clone())
        .push(")");
    Ok(())
}

pub(super) fn push_order(builder: &mut QueryBuilder<Postgres>, direction: CursorDirection) {
    builder
        .push(" ORDER BY occurred_at ")
        .push(physical_direction(direction))
        .push(",id ")
        .push(physical_direction(direction));
}

pub(super) fn push_limit(builder: &mut QueryBuilder<Postgres>, limit: u64) -> ObservabilityResult<()> {
    let limit = limit
        .checked_add(1)
        .and_then(|value| i64::try_from(value).ok())
        .ok_or_else(|| ObservabilityError::Infrastructure("system log cursor limit overflow".into()))?;
    builder.push(" LIMIT ").push_bind(limit);
    Ok(())
}

pub(super) fn slice(mut items: Vec<SystemLogSummary>, snapshot: SystemLogSnapshot, page: SystemLogCursorQuery) -> ObservabilityResult<SystemLogCursorSlice> {
    let limit = page_limit(page.limit)?;
    let has_extra = items.len() > limit;
    if has_extra {
        items.truncate(limit);
    }
    if page.direction == CursorDirection::Previous {
        items.reverse();
    }
    let (has_next, has_previous) = match page.direction {
        CursorDirection::Next => (has_extra, page.boundary.is_some()),
        CursorDirection::Previous => (page.boundary.is_some(), has_extra),
    };
    Ok(SystemLogCursorSlice {
        has_next,
        has_previous,
        items,
        snapshot: Some(snapshot),
    })
}

pub(super) fn slice_export(
    mut items: Vec<SystemLogDetail>,
    snapshot: SystemLogSnapshot,
    page: SystemLogCursorQuery,
) -> ObservabilityResult<SystemLogExportSlice> {
    let limit = page_limit(page.limit)?;
    let has_next = items.len() > limit;
    if has_next {
        items.truncate(limit);
    }
    if page.direction == CursorDirection::Previous {
        items.reverse();
    }
    Ok(SystemLogExportSlice {
        items,
        snapshot: Some(snapshot),
        has_next,
    })
}

fn page_limit(value: u64) -> ObservabilityResult<usize> {
    usize::try_from(value).map_err(|error| ObservabilityError::Infrastructure(format!("system log cursor limit conversion failed: {error}")))
}

fn comparison(direction: CursorDirection) -> &'static str {
    match direction {
        CursorDirection::Next => "<",
        CursorDirection::Previous => ">",
    }
}

fn physical_direction(direction: CursorDirection) -> &'static str {
    match direction {
        CursorDirection::Next => "DESC",
        CursorDirection::Previous => "ASC",
    }
}

#[cfg(test)]
mod tests {
    use kernel::pagination::CursorDirection;

    use crate::{
        application::{SystemLogCursorQuery, SystemLogSnapshot},
        domain::{SystemLogLevel, SystemLogSummary},
    };

    use super::slice;

    #[test]
    fn previous_slice_returns_rows_in_descending_logical_order() {
        let page = slice(
            vec![summary("one"), summary("two"), summary("three")],
            SystemLogSnapshot::new(1),
            SystemLogCursorQuery {
                limit: 2,
                direction: CursorDirection::Previous,
                boundary: Some(crate::application::SystemLogBoundary {
                    occurred_at_nanos: "0".into(),
                    id: "boundary".into(),
                }),
                snapshot: None,
            },
        )
        .unwrap();

        assert_eq!(page.items.into_iter().map(|item| item.id).collect::<Vec<_>>(), vec!["two", "one"]);
        assert!(page.has_next);
        assert!(page.has_previous);
    }

    #[test]
    fn previous_slice_on_the_first_page_has_no_previous_cursor() {
        let page = slice(
            vec![summary("one"), summary("two")],
            SystemLogSnapshot::new(1),
            SystemLogCursorQuery {
                limit: 2,
                direction: CursorDirection::Previous,
                boundary: Some(crate::application::SystemLogBoundary {
                    occurred_at_nanos: "0".into(),
                    id: "boundary".into(),
                }),
                snapshot: None,
            },
        )
        .unwrap();

        assert!(page.has_next);
        assert!(!page.has_previous);
    }

    fn summary(id: &str) -> SystemLogSummary {
        SystemLogSummary {
            id: id.into(),
            occurred_at: time::OffsetDateTime::UNIX_EPOCH,
            level: SystemLogLevel::Info,
            target: "test".into(),
            message: "test".into(),
        }
    }
}
