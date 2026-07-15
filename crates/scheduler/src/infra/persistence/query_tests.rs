use kernel::pagination::CursorDirection;
use sqlx::{Postgres, QueryBuilder};

use crate::application::{SchedulerCursorPoint, SchedulerCursorQuery};

use super::cursor::{WindowSpec, push_window, slice};

#[test]
fn window_sql_uses_stable_tuple_and_bidirectional_order() {
    let point = SchedulerCursorPoint {
        created_at_nanos: "0".into(),
        id: "job-2".into(),
    };
    let mut builder = QueryBuilder::<Postgres>::new("SELECT * FROM sys_job WHERE TRUE");
    let page = SchedulerCursorQuery {
        limit: 20,
        direction: CursorDirection::Previous,
        boundary: Some(point.clone()),
        snapshot: Some(point.clone()),
    };
    push_window(
        &mut builder,
        WindowSpec {
            id_column: "job_id",
            snapshot: &point,
            page: &page,
        },
    )
    .unwrap();
    let sql = builder.sql();
    assert!(sql.as_str().contains("(create_time,job_id) > ("));
    assert!(sql.as_str().contains("(create_time,job_id) <= ("));
    assert!(sql.as_str().contains("ORDER BY create_time ASC,job_id ASC"));
    assert!(!sql.as_str().contains("OFFSET"));
}

#[test]
fn previous_query_rows_are_reversed_back_to_logical_descending_order() {
    let snapshot = SchedulerCursorPoint {
        created_at_nanos: "0".into(),
        id: "snapshot".into(),
    };
    let page = slice(
        vec![4, 5, 6],
        snapshot,
        SchedulerCursorQuery {
            limit: 2,
            direction: CursorDirection::Previous,
            boundary: Some(SchedulerCursorPoint {
                created_at_nanos: "0".into(),
                id: "boundary".into(),
            }),
            snapshot: None,
        },
    )
    .unwrap();

    assert_eq!(page.items, vec![5, 4]);
    assert!(page.has_next);
    assert!(page.has_previous);
}
