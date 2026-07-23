use sqlx::{Postgres, QueryBuilder};

use crate::{FileError, application::FileSpaceQuery, domain::ByteSize, error::keys};

use super::super::SpaceRecord;
use super::{PageCursor, SortDirection, SpaceSortField, SpaceSortSpec};

const DEFAULT_QUOTA: ByteSize = ByteSize::from_bytes(2_048);

fn cursor(sort_value: &str) -> PageCursor {
    PageCursor {
        sort_value: sort_value.into(),
        id: "space-1".into(),
        fingerprint: "fingerprint".into(),
        limit: 20,
    }
}

fn space_record(quota_override_bytes: Option<i64>) -> SpaceRecord {
    SpaceRecord {
        space_id: "space-1".into(),
        owner_user_id: "owner-1".into(),
        owner_name: "Owner".into(),
        department_name: Some("Engineering".into()),
        status: "active".into(),
        active_bytes: 1_024,
        trashed_bytes: 0,
        physical_bytes: 1_024,
        reserved_bytes: 512,
        quota_override_bytes,
        updated_at: time::OffsetDateTime::from_unix_timestamp(1_784_700_000).unwrap(),
    }
}

#[test]
fn space_sort_spec_accepts_each_supported_field_and_direction() {
    let fields = [
        ("owner_name", SpaceSortField::OwnerName),
        ("department_name", SpaceSortField::DepartmentName),
        ("status", SpaceSortField::Status),
        ("logical_asset_size", SpaceSortField::LogicalAssetSize),
        ("reserved_bytes", SpaceSortField::ReservedBytes),
        ("quota_bytes", SpaceSortField::QuotaBytes),
        ("updated_at", SpaceSortField::UpdatedAt),
    ];
    let directions = [("asc", SortDirection::Asc), ("desc", SortDirection::Desc)];

    for (sort_by, field) in fields {
        for (sort_order, direction) in directions {
            let filter = FileSpaceQuery {
                sort_by: Some(sort_by.into()),
                sort_order: Some(sort_order.into()),
                ..FileSpaceQuery::default()
            };

            assert_eq!(SpaceSortSpec::from_filter(&filter).unwrap(), SpaceSortSpec { field, direction });
        }
    }
}

#[test]
fn space_sort_spec_rejects_unknown_field_and_direction() {
    let unknown_field = FileSpaceQuery {
        sort_by: Some("size".into()),
        ..FileSpaceQuery::default()
    };
    let unknown_direction = FileSpaceQuery {
        sort_order: Some("descending".into()),
        ..FileSpaceQuery::default()
    };

    assert_eq!(
        SpaceSortSpec::from_filter(&unknown_field),
        Err(FileError::InvalidInput(keys::SORT_FIELD_INVALID))
    );
    assert_eq!(
        SpaceSortSpec::from_filter(&unknown_direction),
        Err(FileError::InvalidInput(keys::SORT_ORDER_INVALID))
    );
}

#[test]
fn space_sort_cursor_parses_numeric_and_timestamp_values() {
    for (sort_by, column) in [
        ("logical_asset_size", "s.active_bytes"),
        ("reserved_bytes", "s.reserved_bytes"),
        ("quota_bytes", "COALESCE(s.quota_override_bytes,"),
    ] {
        let spec = SpaceSortSpec::from_filter(&FileSpaceQuery {
            sort_by: Some(sort_by.into()),
            sort_order: Some("asc".into()),
            ..FileSpaceQuery::default()
        })
        .unwrap();
        let mut query = QueryBuilder::<Postgres>::new("SELECT 1");

        spec.push_cursor_bound(&mut query, Some(&cursor("512")), DEFAULT_QUOTA).unwrap();
        assert!(query.sql().as_str().contains(column));
    }

    let spec = SpaceSortSpec::from_filter(&FileSpaceQuery {
        sort_by: Some("updated_at".into()),
        ..FileSpaceQuery::default()
    })
    .unwrap();
    let mut query = QueryBuilder::<Postgres>::new("SELECT 1");
    spec.push_cursor_bound(&mut query, Some(&cursor("2026-07-22T12:30:45Z")), DEFAULT_QUOTA)
        .unwrap();

    assert!(query.sql().as_str().contains("s.updated_at"));
}

#[test]
fn space_sort_cursor_rejects_malformed_numeric_and_timestamp_values() {
    let numeric = SpaceSortSpec::from_filter(&FileSpaceQuery {
        sort_by: Some("reserved_bytes".into()),
        ..FileSpaceQuery::default()
    })
    .unwrap();
    let timestamp = SpaceSortSpec::from_filter(&FileSpaceQuery {
        sort_by: Some("updated_at".into()),
        ..FileSpaceQuery::default()
    })
    .unwrap();
    let mut numeric_query = QueryBuilder::<Postgres>::new("SELECT 1");
    let mut timestamp_query = QueryBuilder::<Postgres>::new("SELECT 1");

    assert_eq!(
        numeric.push_cursor_bound(&mut numeric_query, Some(&cursor("not-a-number")), DEFAULT_QUOTA),
        Err(FileError::InvalidInput(keys::CURSOR_MALFORMED))
    );
    assert_eq!(
        timestamp.push_cursor_bound(&mut timestamp_query, Some(&cursor("not-a-timestamp")), DEFAULT_QUOTA),
        Err(FileError::InvalidInput(keys::TIME_FILTER_INVALID))
    );
}

#[test]
fn quota_sort_cursor_uses_default_quota_when_the_space_has_no_override() {
    let spec = SpaceSortSpec::from_filter(&FileSpaceQuery {
        sort_by: Some("quota_bytes".into()),
        ..FileSpaceQuery::default()
    })
    .unwrap();

    assert_eq!(spec.cursor_value(&space_record(None), DEFAULT_QUOTA).unwrap(), "2048");
    assert_eq!(spec.cursor_value(&space_record(Some(4_096)), DEFAULT_QUOTA).unwrap(), "4096");
}
