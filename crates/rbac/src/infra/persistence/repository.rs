use kernel::pagination::{Page, PageSliceRequest};
use storage::{Database, StorageError, StorageResult};

pub(super) const ROLE_COLUMNS: &str = r#"
    code,
    name,
    description,
    enabled,
    system,
    sort_order,
    created_at,
    updated_at
"#;
pub(super) const API_PERMISSION_COLUMNS: &str = r#"
    id,
    code,
    method,
    path_pattern,
    name,
    "group",
    enabled,
    system,
    created_at,
    updated_at
"#;
pub(super) const MENU_SECTION_COLUMNS: &str = r#"
    id,
    code,
    subheader,
    sort_order,
    enabled,
    created_at,
    updated_at
"#;
pub(super) const MENU_ITEM_COLUMNS: &str = r#"
    id,
    section_id,
    parent_id,
    code,
    title,
    route_path,
    icon,
    caption,
    deep_match,
    sort_order,
    enabled,
    created_at,
    updated_at
"#;
pub(super) const ROLE_API_BINDING_COLUMNS: &str = r#"
    role_code,
    api_permission_id,
    created_at,
    updated_at
"#;
pub(super) const ROLE_MENU_BINDING_COLUMNS: &str = r#"
    role_code,
    menu_item_id,
    created_at,
    updated_at
"#;

#[derive(Clone)]
pub struct RbacStore {
    pub(super) database: Database,
}

impl RbacStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

pub(super) fn rbac_page<T>(items: Vec<T>, total: u64, request: PageSliceRequest) -> Page<T> {
    Page {
        items,
        total,
        page: request.page,
        page_size: request.page_size,
    }
}

pub(super) fn ensure_rows_affected(rows_affected: u64) -> StorageResult<()> {
    if rows_affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}
