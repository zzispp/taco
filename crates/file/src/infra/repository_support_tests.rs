use kernel::pagination::CursorPageRequest;
use sqlx::{Postgres, QueryBuilder};

use crate::{
    FileError,
    application::{FileAccessScope, FileListQuery, FileScopeMode},
    domain::SpaceId,
    error::keys,
};

use super::{EntrySortSpec, PageCursor, VIRTUAL_SPACE_CTE, decode_cursor, entry_query, normalize_list_filter, page_fingerprint, scope_query};

fn scope_sql(scope: &FileAccessScope) -> String {
    let mut query = QueryBuilder::<Postgres>::new("SELECT 1 WHERE");
    scope_query(&mut query, scope, "s");
    query.sql().as_str().to_owned()
}

fn entry_sql(filter: &FileListQuery) -> String {
    let mut query = QueryBuilder::<Postgres>::new("SELECT 1");
    entry_query(&mut query, &FileAccessScope::all("actor"), filter).unwrap();
    query.sql().as_str().to_owned()
}

fn scope(mode: FileScopeMode, department_ids: Vec<String>) -> FileAccessScope {
    FileAccessScope::scoped("actor", mode, Some("dept-1".into()), department_ids)
}

#[test]
fn scoped_queries_always_include_the_actor_and_bound_department_ancestors() {
    let department = scope_sql(&scope(FileScopeMode::Department, Vec::new()));
    assert!(department.contains("s.owner_user_id=$1"));
    assert!(department.contains("scoped_owner.del_flag='2' THEN s.owner_dept_id ELSE scoped_owner.dept_id END"));
    assert!(department.contains("scoped_owner.user_id=s.owner_user_id)=$2"));

    let descendants = scope_sql(&scope(FileScopeMode::DepartmentAndChildren, Vec::new()));
    assert!(descendants.contains("s.owner_user_id=$1"));
    assert!(descendants.contains("(',' || child.ancestors || ',') LIKE '%,' || $3 || ',%'"));
    assert!(!descendants.contains(")) )"));
    assert!(!descendants.contains("child.ancestors LIKE '%'"));

    let custom = scope_sql(&scope(FileScopeMode::Custom, Vec::new()));
    assert!(custom.contains("s.owner_user_id=$1"));
    assert!(custom.contains("OR FALSE"));
}

#[test]
fn cursor_fingerprint_ignores_cursor_and_canonicalizes_department_order() {
    let actor_a = scope(FileScopeMode::Custom, vec!["dept-2".into(), "dept-1".into()]);
    let actor_b = scope(FileScopeMode::Custom, vec!["dept-1".into(), "dept-2".into()]);
    let first = FileListQuery::default();
    let second = FileListQuery {
        cursor: Some("next-page-token".into()),
        ..first.clone()
    };

    assert_eq!(page_fingerprint(&actor_a, &first), page_fingerprint(&actor_a, &second));
    assert_eq!(page_fingerprint(&actor_a, &first), page_fingerprint(&actor_b, &first));

    let page = CursorPageRequest { limit: 20, cursor: None };
    let token = super::encode_cursor("2026-01-01T00:00:00Z", "entry", &page_fingerprint(&actor_a, &first), &page);
    assert!(decode_cursor(Some(&token), &page_fingerprint(&actor_a, &second), &page).is_ok());
}

#[test]
fn list_filter_defaults_to_active_entries() {
    assert_eq!(normalize_list_filter(FileListQuery::default()).trashed, Some(false));
    assert_eq!(
        normalize_list_filter(FileListQuery {
            trashed: Some(true),
            ..FileListQuery::default()
        })
        .trashed,
        Some(true)
    );
}

#[test]
fn unscoped_name_search_spans_all_directories_in_the_selected_space() {
    let sql = entry_sql(&FileListQuery {
        space_id: Some(SpaceId::new("space-1").unwrap()),
        search: Some("zwj.yaml".into()),
        trashed: Some(false),
        ..FileListQuery::default()
    });

    assert!(sql.contains("e.space_id=$"));
    assert!(sql.contains("e.name ILIKE '%' || $"));
    assert!(!sql.contains("e.parent_id"));
}

#[test]
fn repository_entry_query_rejects_unknown_kind_before_database_execution() {
    let filter = FileListQuery {
        kind: Some("shortcut".into()),
        ..FileListQuery::default()
    };
    let mut query = QueryBuilder::<Postgres>::new("SELECT 1");

    let result = entry_query(&mut query, &FileAccessScope::all("actor"), &filter);

    assert_eq!(result, Err(FileError::InvalidInput(keys::ENTRY_TYPE_INVALID)));
}

#[test]
fn sort_spec_emits_keyset_order_and_rejects_unknown_values() {
    let filter = FileListQuery {
        sort_by: Some("name".into()),
        sort_order: Some("asc".into()),
        ..FileListQuery::default()
    };
    let spec = EntrySortSpec::from_filter(&filter).unwrap();
    let mut query = QueryBuilder::<Postgres>::new("SELECT 1");
    spec.push_order(&mut query);
    assert_eq!(query.sql().as_str(), "SELECT 1 ORDER BY e.normalized_name ASC,e.entry_id ASC");

    let cursor = PageCursor {
        sort_value: "alpha".into(),
        id: "entry".into(),
        fingerprint: "fp".into(),
        limit: 20,
    };
    spec.push_cursor_bound(&mut query, Some(&cursor)).unwrap();
    assert!(query.sql().as_str().contains("AND (e.normalized_name,e.entry_id)>("));

    let invalid = FileListQuery {
        sort_by: Some("size".into()),
        ..FileListQuery::default()
    };
    assert!(EntrySortSpec::from_filter(&invalid).is_err());
}

#[test]
fn virtual_space_projection_is_user_complete_and_zero_filled() {
    assert!(VIRTUAL_SPACE_CTE.contains("FROM sys_user u LEFT JOIN file_space fs"));
    assert!(VIRTUAL_SPACE_CTE.contains("COALESCE(fs.active_bytes,0)"));
    assert!(VIRTUAL_SPACE_CTE.contains("COALESCE(fs.trashed_bytes,0)"));
    assert!(VIRTUAL_SPACE_CTE.contains("COALESCE(fs.reserved_bytes,0)"));
    assert!(VIRTUAL_SPACE_CTE.contains("COALESCE(fs.space_id,u.user_id)"));
    assert!(VIRTUAL_SPACE_CTE.contains("u.del_flag='2' THEN 'archived'"));
}
