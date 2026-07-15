use audit_contract::{BusinessType, EndpointMethod, EndpointPermissionRequirement, EndpointSpec};

use super::{operation, permission, read, scoped_permission};

const DEPTS: &str = "/api/system/depts";
const POSTS: &str = "/api/system/posts";

pub(in crate::api) const DASHBOARD: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/dashboard",
    permission("get_server_dashboard", EndpointPermissionRequirement::all_of(&["system:dashboard:view"])),
);

pub(in crate::api) const DEPTS_LIST: EndpointSpec = read(
    EndpointMethod::Get,
    DEPTS,
    scoped_permission("list_depts", EndpointPermissionRequirement::all_of(&["system:dept:list"])),
);
pub(in crate::api) const DEPTS_CREATE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Post,
    path: DEPTS,
    access: permission("create_dept", EndpointPermissionRequirement::all_of(&["system:dept:add"])),
    audit: operation("audit.module.department", BusinessType::Insert, "system::create_dept"),
};
pub(in crate::api) const DEPTS_TREE_SELECT: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/depts/tree-select",
    scoped_permission("dept_tree_select", EndpointPermissionRequirement::all_of(&["system:dept:list"])),
);
pub(in crate::api) const DEPTS_EXCLUDE: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/depts/exclude/{id}",
    permission("exclude_dept_tree", EndpointPermissionRequirement::all_of(&["system:dept:list"])),
);
pub(in crate::api) const DEPTS_SORT: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Put,
    path: "/api/system/depts/sort",
    access: scoped_permission("update_dept_sorts", EndpointPermissionRequirement::all_of(&["system:dept:edit"])),
    audit: operation("audit.module.department", BusinessType::Update, "system::update_dept_sorts"),
};
pub(in crate::api) const DEPT_GET: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/depts/{id}",
    scoped_permission("get_dept", EndpointPermissionRequirement::all_of(&["system:dept:query"])),
);
pub(in crate::api) const DEPT_REPLACE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Put,
    path: "/api/system/depts/{id}",
    access: scoped_permission("replace_dept", EndpointPermissionRequirement::all_of(&["system:dept:edit"])),
    audit: operation("audit.module.department", BusinessType::Update, "system::replace_dept"),
};
pub(in crate::api) const DEPT_DELETE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/depts/{id}",
    access: scoped_permission("delete_dept", EndpointPermissionRequirement::all_of(&["system:dept:remove"])),
    audit: operation("audit.module.department", BusinessType::Delete, "system::delete_dept"),
};
pub(in crate::api) const DEPT_SORT: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Put,
    path: "/api/system/depts/{id}/sort",
    access: scoped_permission("update_dept_sort", EndpointPermissionRequirement::all_of(&["system:dept:edit"])),
    audit: operation("audit.module.department", BusinessType::Update, "system::update_dept_sort"),
};
pub(in crate::api) const ROLE_DEPT_TREE: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/roles/{id}/dept-tree-select",
    permission("role_dept_tree_select", EndpointPermissionRequirement::all_of(&["system:role:query"])),
);

pub(in crate::api) const POSTS_LIST: EndpointSpec = read(
    EndpointMethod::Get,
    POSTS,
    permission("list_posts", EndpointPermissionRequirement::all_of(&["system:post:list"])),
);
pub(in crate::api) const POSTS_CREATE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Post,
    path: POSTS,
    access: permission("create_post", EndpointPermissionRequirement::all_of(&["system:post:add"])),
    audit: operation("audit.module.post", BusinessType::Insert, "system::create_post"),
};
pub(in crate::api) const POSTS_EXPORT: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Post,
    path: "/api/system/posts/export",
    access: permission("export_posts", EndpointPermissionRequirement::all_of(&["system:post:export"])),
    audit: operation("audit.module.post", BusinessType::Export, "system::export_posts"),
};
pub(in crate::api) const POSTS_OPTIONS: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/posts/options",
    permission("post_options", EndpointPermissionRequirement::all_of(&["system:post:list"])),
);
pub(in crate::api) const POSTS_DELETE_BATCH: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/posts/batch",
    access: permission("delete_posts", EndpointPermissionRequirement::all_of(&["system:post:remove"])),
    audit: operation("audit.module.post", BusinessType::Delete, "system::delete_posts"),
};
pub(in crate::api) const POST_GET: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/posts/{id}",
    permission("get_post", EndpointPermissionRequirement::all_of(&["system:post:query"])),
);
pub(in crate::api) const POST_REPLACE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Put,
    path: "/api/system/posts/{id}",
    access: permission("replace_post", EndpointPermissionRequirement::all_of(&["system:post:edit"])),
    audit: operation("audit.module.post", BusinessType::Update, "system::replace_post"),
};
pub(in crate::api) const POST_DELETE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/posts/{id}",
    access: permission("delete_post", EndpointPermissionRequirement::all_of(&["system:post:remove"])),
    audit: operation("audit.module.post", BusinessType::Delete, "system::delete_post"),
};

pub(super) const ENDPOINTS: &[EndpointSpec] = &[
    DASHBOARD,
    DEPTS_LIST,
    DEPTS_CREATE,
    DEPTS_TREE_SELECT,
    DEPTS_EXCLUDE,
    DEPTS_SORT,
    DEPT_GET,
    DEPT_REPLACE,
    DEPT_DELETE,
    DEPT_SORT,
    ROLE_DEPT_TREE,
    POSTS_LIST,
    POSTS_CREATE,
    POSTS_EXPORT,
    POSTS_OPTIONS,
    POSTS_DELETE_BATCH,
    POST_GET,
    POST_REPLACE,
    POST_DELETE,
];
