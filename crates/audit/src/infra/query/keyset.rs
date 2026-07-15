use kernel::pagination::CursorDirection;
use sqlx::{Postgres, QueryBuilder};

use crate::{
    application::{
        AuditCursorQuery, AuditCursorSlice, AuditResult, AuditSnapshot, LoginCursorBoundary, LoginCursorValue, OperationCursorBoundary, OperationCursorValue,
    },
    domain::{LoginLogFilter, LoginSortField, OperationLogFilter, OperationSortField, SortDirection},
};

use crate::application::AuditError;

pub(super) fn push_snapshot(builder: &mut QueryBuilder<Postgres>, id_column: &'static str, snapshot: &AuditSnapshot) -> AuditResult<()> {
    if snapshot.id.is_empty() {
        return Err(AuditError::InvalidCursor);
    }
    builder
        .push(" AND (ingested_at,")
        .push(id_column)
        .push(") <= (")
        .push_bind(snapshot.ingested_at()?)
        .push(",")
        .push_bind(snapshot.id.clone())
        .push(")");
    Ok(())
}

pub(super) fn push_operation_boundary(
    builder: &mut QueryBuilder<Postgres>,
    filter: &OperationLogFilter,
    page: &AuditCursorQuery<OperationCursorBoundary>,
) -> AuditResult<()> {
    let Some(boundary) = page.boundary.as_ref() else { return Ok(()) };
    push_tuple_prefix(
        builder,
        TupleSpec {
            column: filter.sort_field.column(),
            id_column: "oper_id",
            comparison: comparison(filter.sort_direction, page.direction),
        },
        &boundary.id,
    )?;
    match (filter.sort_field, &boundary.value) {
        (OperationSortField::OperationTime, OperationCursorValue::Time(value)) => builder.push_bind(parse_time(value)?),
        (OperationSortField::BusinessType | OperationSortField::Status, OperationCursorValue::SmallInt(value)) => builder.push_bind(*value),
        (OperationSortField::OperatorName, OperationCursorValue::Text(value)) => builder.push_bind(value.clone()),
        (OperationSortField::CostTime, OperationCursorValue::BigInt(value)) => builder.push_bind(*value),
        _ => return Err(AuditError::InvalidCursor),
    };
    push_tuple_suffix(builder, &boundary.id);
    Ok(())
}

pub(super) fn push_login_boundary(
    builder: &mut QueryBuilder<Postgres>,
    filter: &LoginLogFilter,
    page: &AuditCursorQuery<LoginCursorBoundary>,
) -> AuditResult<()> {
    let Some(boundary) = page.boundary.as_ref() else { return Ok(()) };
    push_tuple_prefix(
        builder,
        TupleSpec {
            column: filter.sort_field.column(),
            id_column: "info_id",
            comparison: comparison(filter.sort_direction, page.direction),
        },
        &boundary.id,
    )?;
    match (filter.sort_field, &boundary.value) {
        (LoginSortField::LoginTime, LoginCursorValue::Time(value)) => builder.push_bind(parse_time(value)?),
        (LoginSortField::Username | LoginSortField::IpAddress, LoginCursorValue::Text(value)) => builder.push_bind(value.clone()),
        (LoginSortField::Status, LoginCursorValue::SmallInt(value)) => builder.push_bind(*value),
        _ => return Err(AuditError::InvalidCursor),
    };
    push_tuple_suffix(builder, &boundary.id);
    Ok(())
}

pub(super) fn push_operation_order(builder: &mut QueryBuilder<Postgres>, filter: &OperationLogFilter, direction: CursorDirection) {
    push_order(
        builder,
        OrderSpec {
            column: filter.sort_field.column(),
            id_column: "oper_id",
            direction: physical_direction(filter.sort_direction, direction),
        },
    );
}

pub(super) fn push_login_order(builder: &mut QueryBuilder<Postgres>, filter: &LoginLogFilter, direction: CursorDirection) {
    push_order(
        builder,
        OrderSpec {
            column: filter.sort_field.column(),
            id_column: "info_id",
            direction: physical_direction(filter.sort_direction, direction),
        },
    );
}

pub(super) fn push_limit(builder: &mut QueryBuilder<Postgres>, limit: u64) -> AuditResult<()> {
    let query_limit = limit
        .checked_add(1)
        .and_then(|value| i64::try_from(value).ok())
        .ok_or_else(|| AuditError::Infrastructure("audit cursor limit overflow".into()))?;
    builder.push(" LIMIT ").push_bind(query_limit);
    Ok(())
}

pub(super) fn slice<T, B>(mut items: Vec<T>, snapshot: AuditSnapshot, page: AuditCursorQuery<B>) -> AuditResult<AuditCursorSlice<T>> {
    let limit = usize::try_from(page.limit).map_err(|error| AuditError::Infrastructure(format!("audit cursor limit conversion failed: {error}")))?;
    let has_extra = items.len() > limit;
    if has_extra {
        items.truncate(limit);
    }
    if page.direction == CursorDirection::Previous {
        items.reverse();
    }
    if items.is_empty() {
        return Ok(AuditCursorSlice {
            items,
            snapshot: Some(snapshot),
            has_next: false,
            has_previous: false,
        });
    }
    let had_boundary = page.boundary.is_some();
    let (has_next, has_previous) = match page.direction {
        CursorDirection::Next => (has_extra, had_boundary),
        CursorDirection::Previous => (had_boundary, has_extra),
    };
    Ok(AuditCursorSlice {
        items,
        snapshot: Some(snapshot),
        has_next,
        has_previous,
    })
}

struct TupleSpec {
    column: &'static str,
    id_column: &'static str,
    comparison: &'static str,
}

fn push_tuple_prefix(builder: &mut QueryBuilder<Postgres>, spec: TupleSpec, id: &str) -> AuditResult<()> {
    if id.is_empty() {
        return Err(AuditError::InvalidCursor);
    }
    builder
        .push(" AND (")
        .push(spec.column)
        .push(",")
        .push(spec.id_column)
        .push(") ")
        .push(spec.comparison)
        .push(" (");
    Ok(())
}

fn push_tuple_suffix(builder: &mut QueryBuilder<Postgres>, id: &str) {
    builder.push(",").push_bind(id.to_owned()).push(")");
}

struct OrderSpec {
    column: &'static str,
    id_column: &'static str,
    direction: &'static str,
}

fn push_order(builder: &mut QueryBuilder<Postgres>, spec: OrderSpec) {
    builder
        .push(" ORDER BY ")
        .push(spec.column)
        .push(" ")
        .push(spec.direction)
        .push(",")
        .push(spec.id_column)
        .push(" ")
        .push(spec.direction);
}

fn comparison(direction: SortDirection, cursor: CursorDirection) -> &'static str {
    match physical_direction(direction, cursor) {
        "ASC" => ">",
        _ => "<",
    }
}

fn physical_direction(direction: SortDirection, cursor: CursorDirection) -> &'static str {
    match (direction, cursor) {
        (SortDirection::Asc, CursorDirection::Next) | (SortDirection::Desc, CursorDirection::Previous) => "ASC",
        (SortDirection::Desc, CursorDirection::Next) | (SortDirection::Asc, CursorDirection::Previous) => "DESC",
    }
}

fn parse_time(value: &str) -> AuditResult<time::OffsetDateTime> {
    let nanos = value.parse::<i128>().map_err(|_| AuditError::InvalidCursor)?;
    time::OffsetDateTime::from_unix_timestamp_nanos(nanos).map_err(|_| AuditError::InvalidCursor)
}

#[cfg(test)]
mod tests {
    use kernel::pagination::CursorDirection;
    use sqlx::{Postgres, QueryBuilder};

    use crate::{
        application::{AuditCursorQuery, AuditSnapshot, OperationCursorBoundary, OperationCursorValue},
        domain::{OperationLogFilter, SortDirection},
    };

    use super::{comparison, physical_direction, push_operation_boundary, push_operation_order, push_snapshot, slice};

    #[test]
    fn direction_matrix_matches_bidirectional_keyset_semantics() {
        assert_eq!(
            (
                comparison(SortDirection::Asc, CursorDirection::Next),
                physical_direction(SortDirection::Asc, CursorDirection::Next)
            ),
            (">", "ASC")
        );
        assert_eq!(
            (
                comparison(SortDirection::Asc, CursorDirection::Previous),
                physical_direction(SortDirection::Asc, CursorDirection::Previous)
            ),
            ("<", "DESC")
        );
        assert_eq!(
            (
                comparison(SortDirection::Desc, CursorDirection::Next),
                physical_direction(SortDirection::Desc, CursorDirection::Next)
            ),
            ("<", "DESC")
        );
        assert_eq!(
            (
                comparison(SortDirection::Desc, CursorDirection::Previous),
                physical_direction(SortDirection::Desc, CursorDirection::Previous)
            ),
            (">", "ASC")
        );
    }

    #[test]
    fn operation_window_sql_contains_snapshot_boundary_and_stable_order() {
        let filter = OperationLogFilter::default();
        let snapshot = AuditSnapshot {
            ingested_at_nanos: "0".into(),
            id: "snapshot".into(),
        };
        let query = AuditCursorQuery {
            limit: 20,
            direction: CursorDirection::Next,
            boundary: Some(OperationCursorBoundary {
                value: OperationCursorValue::Time("0".into()),
                id: "boundary".into(),
            }),
            snapshot: Some(snapshot.clone()),
        };
        let mut builder = QueryBuilder::<Postgres>::new("SELECT * FROM sys_oper_log WHERE TRUE");

        push_snapshot(&mut builder, "oper_id", &snapshot).unwrap();
        push_operation_boundary(&mut builder, &filter, &query).unwrap();
        push_operation_order(&mut builder, &filter, query.direction);

        let sql = builder.sql();
        assert!(sql.as_str().contains("(ingested_at,oper_id) <= ("));
        assert!(sql.as_str().contains("(oper_time,oper_id) < ("));
        assert!(sql.as_str().contains("ORDER BY oper_time DESC,oper_id DESC"));
        assert!(!sql.as_str().contains("OFFSET"));
    }

    #[test]
    fn previous_rows_are_reversed_to_the_logical_order() {
        let snapshot = AuditSnapshot {
            ingested_at_nanos: "0".into(),
            id: "snapshot".into(),
        };
        let page = slice(
            vec![4, 5, 6],
            snapshot,
            AuditCursorQuery {
                limit: 2,
                direction: CursorDirection::Previous,
                boundary: Some("boundary"),
                snapshot: None,
            },
        )
        .unwrap();

        assert_eq!(page.items, vec![5, 4]);
        assert!(page.has_next);
        assert!(page.has_previous);
    }
}
