use audit_contract::{
    BusinessType, EndpointAccess, EndpointAudit, EndpointManifest, EndpointMethod, EndpointPermission, EndpointPermissionRequirement, EndpointSpec,
    OperationEndpointAudit, RequestCapture,
};

const ROLES: &str = "/api/system/roles";
const MENUS: &str = "/api/system/menus";

const fn permission(handler: &'static str, requirement: EndpointPermissionRequirement) -> EndpointAccess {
    EndpointAccess::Permission(EndpointPermission { handler, requirement })
}

const fn scoped_permission(handler: &'static str, requirement: EndpointPermissionRequirement) -> EndpointAccess {
    EndpointAccess::DataScopedPermission(EndpointPermission { handler, requirement })
}

const fn read(method: EndpointMethod, path: &'static str, access: EndpointAccess) -> EndpointSpec {
    EndpointSpec {
        method,
        path,
        access,
        audit: EndpointAudit::read_only_for(method),
    }
}

const fn operation(title_key: &'static str, business_type: BusinessType, handler: &'static str) -> EndpointAudit {
    EndpointAudit::Operation(OperationEndpointAudit {
        title_key,
        business_type,
        handler,
        request_capture: RequestCapture::Sanitized,
    })
}

pub(super) const NAVBAR: EndpointSpec = read(EndpointMethod::Get, "/api/navbar", EndpointAccess::Authenticated);

pub(super) const ROLES_LIST: EndpointSpec = read(
    EndpointMethod::Get,
    ROLES,
    scoped_permission("list_roles", EndpointPermissionRequirement::all_of(&["system:role:list"])),
);
pub(super) const ROLES_CREATE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Post,
    path: ROLES,
    access: permission("create_role", EndpointPermissionRequirement::all_of(&["system:role:add"])),
    audit: operation("audit.module.role", BusinessType::Insert, "rbac::create_role"),
};
pub(super) const ROLES_EXPORT: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Post,
    path: "/api/system/roles/export",
    access: scoped_permission("export_roles", EndpointPermissionRequirement::all_of(&["system:role:export"])),
    audit: operation("audit.module.role", BusinessType::Export, "rbac::export_roles"),
};
pub(super) const ROLES_OPTIONS: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/roles/options",
    permission("role_options", EndpointPermissionRequirement::all_of(&["system:role:list"])),
);
pub(super) const ROLES_DELETE_BATCH: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/roles/batch",
    access: permission("delete_roles", EndpointPermissionRequirement::all_of(&["system:role:remove"])),
    audit: operation("audit.module.role", BusinessType::Delete, "rbac::delete_roles"),
};
pub(super) const ROLE_GET: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/roles/{id}",
    permission("get_role", EndpointPermissionRequirement::all_of(&["system:role:query"])),
);
pub(super) const ROLE_REPLACE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Put,
    path: "/api/system/roles/{id}",
    access: permission("replace_role", EndpointPermissionRequirement::all_of(&["system:role:edit"])),
    audit: operation("audit.module.role", BusinessType::Update, "rbac::replace_role"),
};
pub(super) const ROLE_DELETE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/roles/{id}",
    access: permission("delete_role", EndpointPermissionRequirement::all_of(&["system:role:remove"])),
    audit: operation("audit.module.role", BusinessType::Delete, "rbac::delete_role"),
};
pub(super) const ROLE_STATUS: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Put,
    path: "/api/system/roles/{id}/status",
    access: permission("update_role_status", EndpointPermissionRequirement::all_of(&["system:role:edit"])),
    audit: operation("audit.module.role", BusinessType::Update, "rbac::update_role_status"),
};
pub(super) const ROLE_DATA_SCOPE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Put,
    path: "/api/system/roles/{id}/data-scope",
    access: permission("update_role_data_scope", EndpointPermissionRequirement::all_of(&["system:role:edit"])),
    audit: operation("audit.module.role", BusinessType::Update, "rbac::update_role_data_scope"),
};
pub(super) const ROLE_MENUS_GET: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/roles/{id}/menus",
    permission("role_menu_bindings", EndpointPermissionRequirement::all_of(&["system:role:query"])),
);
pub(super) const ROLE_MENUS_REPLACE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Put,
    path: "/api/system/roles/{id}/menus",
    access: permission("replace_role_menus", EndpointPermissionRequirement::all_of(&["system:role:edit"])),
    audit: operation("audit.module.role", BusinessType::Grant, "rbac::replace_role_menus"),
};
pub(super) const ROLE_DEPTS_GET: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/roles/{id}/depts",
    permission("role_dept_bindings", EndpointPermissionRequirement::all_of(&["system:role:query"])),
);
pub(super) const ROLE_DEPTS_REPLACE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Put,
    path: "/api/system/roles/{id}/depts",
    access: permission("replace_role_depts", EndpointPermissionRequirement::all_of(&["system:role:edit"])),
    audit: operation("audit.module.role", BusinessType::Grant, "rbac::replace_role_depts"),
};
pub(super) const ROLE_USERS_GET: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/roles/{id}/users",
    scoped_permission("role_users", EndpointPermissionRequirement::all_of(&["system:role:list"])),
);
pub(super) const ROLE_USERS_REPLACE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Put,
    path: "/api/system/roles/{id}/users",
    access: scoped_permission("replace_role_users", EndpointPermissionRequirement::all_of(&["system:role:edit"])),
    audit: operation("audit.module.role", BusinessType::Grant, "rbac::replace_role_users"),
};
pub(super) const ROLE_USERS_DELETE_BATCH: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/roles/{id}/users/batch",
    access: scoped_permission("delete_role_users", EndpointPermissionRequirement::all_of(&["system:role:remove"])),
    audit: operation("audit.module.role", BusinessType::Grant, "rbac::delete_role_users"),
};
pub(super) const ROLE_USER_DELETE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/roles/{id}/users/{user_id}",
    access: scoped_permission("delete_role_user", EndpointPermissionRequirement::all_of(&["system:role:remove"])),
    audit: operation("audit.module.role", BusinessType::Grant, "rbac::delete_role_user"),
};
pub(super) const ROLE_MENU_TREE: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/menus/role-tree-select/{id}",
    permission("role_menu_tree_select", EndpointPermissionRequirement::all_of(&["system:role:query"])),
);

pub(super) const MENUS_LIST: EndpointSpec = read(
    EndpointMethod::Get,
    MENUS,
    permission("list_menus", EndpointPermissionRequirement::all_of(&["system:menu:list"])),
);
pub(super) const MENUS_CREATE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Post,
    path: MENUS,
    access: permission("create_menu", EndpointPermissionRequirement::all_of(&["system:menu:add"])),
    audit: operation("audit.module.menu", BusinessType::Insert, "rbac::create_menu"),
};
pub(super) const MENUS_TREE: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/menus/tree",
    permission("list_menu_tree", EndpointPermissionRequirement::all_of(&["system:menu:list"])),
);
pub(super) const MENUS_TREE_SELECT: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/menus/tree-select",
    permission("menu_tree_select", EndpointPermissionRequirement::all_of(&["system:menu:list"])),
);
pub(super) const MENUS_SORT: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Put,
    path: "/api/system/menus/sort",
    access: permission("update_menu_sorts", EndpointPermissionRequirement::all_of(&["system:menu:edit"])),
    audit: operation("audit.module.menu", BusinessType::Update, "rbac::update_menu_sorts"),
};
pub(super) const MENU_GET: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/menus/{id}",
    permission("get_menu", EndpointPermissionRequirement::all_of(&["system:menu:query"])),
);
pub(super) const MENU_REPLACE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Put,
    path: "/api/system/menus/{id}",
    access: permission("replace_menu", EndpointPermissionRequirement::all_of(&["system:menu:edit"])),
    audit: operation("audit.module.menu", BusinessType::Update, "rbac::replace_menu"),
};
pub(super) const MENU_DELETE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/menus/{id}",
    access: permission("delete_menu", EndpointPermissionRequirement::all_of(&["system:menu:remove"])),
    audit: operation("audit.module.menu", BusinessType::Delete, "rbac::delete_menu"),
};
pub(super) const MENU_SORT: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Put,
    path: "/api/system/menus/{id}/sort",
    access: permission("update_menu_sort", EndpointPermissionRequirement::all_of(&["system:menu:edit"])),
    audit: operation("audit.module.menu", BusinessType::Update, "rbac::update_menu_sort"),
};

const ENDPOINTS: &[EndpointSpec] = &[
    NAVBAR,
    ROLES_LIST,
    ROLES_CREATE,
    ROLES_EXPORT,
    ROLES_OPTIONS,
    ROLES_DELETE_BATCH,
    ROLE_GET,
    ROLE_REPLACE,
    ROLE_DELETE,
    ROLE_STATUS,
    ROLE_DATA_SCOPE,
    ROLE_MENUS_GET,
    ROLE_MENUS_REPLACE,
    ROLE_DEPTS_GET,
    ROLE_DEPTS_REPLACE,
    ROLE_USERS_GET,
    ROLE_USERS_REPLACE,
    ROLE_USERS_DELETE_BATCH,
    ROLE_USER_DELETE,
    ROLE_MENU_TREE,
    MENUS_LIST,
    MENUS_CREATE,
    MENUS_TREE,
    MENUS_TREE_SELECT,
    MENUS_SORT,
    MENU_GET,
    MENU_REPLACE,
    MENU_DELETE,
    MENU_SORT,
];

const SEGMENTS: &[&[EndpointSpec]] = &[ENDPOINTS];

pub fn endpoint_specs() -> EndpointManifest {
    EndpointManifest::new(SEGMENTS)
}

#[cfg(test)]
mod tests {
    use audit_contract::{EndpointAccess, EndpointAudit};

    use super::endpoint_specs;

    #[test]
    fn endpoint_specs_cover_rbac_routes_and_management_actions() {
        let specs = endpoint_specs();
        let entries = specs.iter().collect::<Vec<_>>();

        specs.validate().unwrap();
        assert_eq!(entries.len(), 29);
        assert_eq!(entries.iter().filter(|spec| matches!(spec.audit, EndpointAudit::Operation(_))).count(), 17);
        assert_eq!(
            entries
                .iter()
                .filter(|spec| matches!(spec.access, EndpointAccess::DataScopedPermission(_)))
                .count(),
            6
        );
    }
}
