use axum::extract::Query;
use rbac::{
    api::CurrentUser,
    domain::{DataScope, DataScopeFilter},
};

use crate::application::FileScopeMode;

use super::{FileListParams, FileSpaceParams, UPLOAD_MANAGEMENT_PERMISSION, file_scope, normalize_content_type};

#[test]
fn content_type_is_canonicalized_before_preview_policy_uses_it() {
    assert_eq!(normalize_content_type(Some(" Image/PNG; charset=binary ".into())).unwrap(), "image/png");
    assert!(normalize_content_type(Some("invalid".into())).is_err());
}

#[test]
fn file_list_query_accepts_explicit_cursor_page_fields() {
    let uri = "/?limit=50&cursor=next-page&search=report".parse().unwrap();
    let Query(params) = Query::<FileListParams>::try_from_uri(&uri).unwrap();
    let (query, page) = params.into_query().unwrap();

    assert_eq!(page.limit, 50);
    assert_eq!(page.cursor.as_deref(), Some("next-page"));
    assert_eq!(query.search.as_deref(), Some("report"));
}

#[test]
fn file_list_query_accepts_file_and_folder_kinds() {
    for kind in ["file", "folder"] {
        let uri = format!("/?kind={kind}").parse().unwrap();
        let Query(params) = Query::<FileListParams>::try_from_uri(&uri).unwrap();
        let (query, _) = params.into_query().unwrap();

        assert_eq!(query.kind.as_deref(), Some(kind));
    }
}

#[test]
fn file_space_query_accepts_explicit_cursor_page_fields() {
    let uri = "/?limit=100&cursor=next-space&status=active".parse().unwrap();
    let Query(params) = Query::<FileSpaceParams>::try_from_uri(&uri).unwrap();
    let (query, page) = params.into_query().unwrap();

    assert_eq!(page.limit, 100);
    assert_eq!(page.cursor.as_deref(), Some("next-space"));
    assert_eq!(query.status.as_deref(), Some("active"));
}

#[test]
fn only_explicit_upload_management_permission_can_manage_uploads() {
    let current_user = CurrentUser {
        id: "owner-user".into(),
        username: "owner".into(),
        role_keys: vec!["admin".into()],
        permissions: Vec::new(),
        dept_id: None,
    };
    let filter = DataScopeFilter {
        data_scope: DataScope::Department,
        user_id: current_user.id.clone(),
        dept_id: Some("dept-1".into()),
        dept_ids: Vec::new(),
    };

    let denied_scope = file_scope(&current_user, &filter);
    assert_eq!(denied_scope.mode, FileScopeMode::Department);
    assert_eq!(denied_scope.department_id.as_deref(), Some("dept-1"));
    assert!(!denied_scope.can_manage_uploads);

    let authorized_user = CurrentUser {
        permissions: vec![UPLOAD_MANAGEMENT_PERMISSION.into()],
        ..current_user
    };
    assert!(file_scope(&authorized_user, &filter).can_manage_uploads);
}
