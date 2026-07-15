use audit_contract::{
    BusinessType, EndpointAccess, EndpointAudit, EndpointMethod, EndpointPermission, EndpointPermissionRequirement, EndpointSpec, OperationEndpointAudit,
    RequestCapture,
};

macro_rules! read_only {
    ($method:ident, $path:literal, $access:expr) => {
        EndpointSpec {
            method: EndpointMethod::$method,
            path: $path,
            access: $access,
            audit: EndpointAudit::read_only_for(EndpointMethod::$method),
        }
    };
}

macro_rules! security {
    ($method:ident, $path:literal, $access:expr) => {
        EndpointSpec {
            method: EndpointMethod::$method,
            path: $path,
            access: $access,
            audit: EndpointAudit::Security,
        }
    };
}

macro_rules! permission_read_only {
    ($method:ident, $path:literal, $permission:literal, $handler:literal) => {
        read_only!(
            $method,
            $path,
            EndpointAccess::Permission(EndpointPermission {
                handler: $handler,
                requirement: EndpointPermissionRequirement::all_of(&[$permission]),
            })
        )
    };
}

macro_rules! scoped_permission_read_only {
    ($method:ident, $path:literal, $permission:literal, $handler:literal) => {
        read_only!(
            $method,
            $path,
            EndpointAccess::DataScopedPermission(EndpointPermission {
                handler: $handler,
                requirement: EndpointPermissionRequirement::all_of(&[$permission]),
            })
        )
    };
}

macro_rules! operation {
    ($method:ident, $path:literal, $access:expr, $title:literal, $business:expr, $handler:literal) => {
        EndpointSpec {
            method: EndpointMethod::$method,
            path: $path,
            access: $access,
            audit: EndpointAudit::Operation(OperationEndpointAudit {
                title_key: $title,
                business_type: $business,
                handler: $handler,
                request_capture: RequestCapture::Sanitized,
            }),
        }
    };
}

macro_rules! permission_operation {
    ($method:ident, $path:literal, $permission:literal, $permission_handler:literal, $title:literal, $business:expr, $handler:literal) => {
        operation!(
            $method,
            $path,
            EndpointAccess::Permission(EndpointPermission {
                handler: $permission_handler,
                requirement: EndpointPermissionRequirement::all_of(&[$permission]),
            }),
            $title,
            $business,
            $handler
        )
    };
}

macro_rules! scoped_permission_operation {
    ($method:ident, $path:literal, $permission:literal, $permission_handler:literal, $title:literal, $business:expr, $handler:literal) => {
        operation!(
            $method,
            $path,
            EndpointAccess::DataScopedPermission(EndpointPermission {
                handler: $permission_handler,
                requirement: EndpointPermissionRequirement::all_of(&[$permission]),
            }),
            $title,
            $business,
            $handler
        )
    };
}

pub(super) const AUTH_SIGN_UP: EndpointSpec = security!(Post, "/api/auth/sign-up", EndpointAccess::Public);
pub(super) const AUTH_SIGN_IN: EndpointSpec = security!(Post, "/api/auth/sign-in", EndpointAccess::Public);
pub(super) const AUTH_REFRESH: EndpointSpec = security!(Post, "/api/auth/refresh", EndpointAccess::Public);
pub(super) const AUTH_LOGOUT: EndpointSpec = security!(Post, "/api/auth/logout", EndpointAccess::Public);
pub(super) const AUTH_ME: EndpointSpec = read_only!(Get, "/api/auth/me", EndpointAccess::SelfAuthenticated);

pub(super) const PROFILE_GET: EndpointSpec = read_only!(Get, "/api/account/profile", EndpointAccess::Authenticated);
pub(super) const PROFILE_UPDATE: EndpointSpec = operation!(
    Put,
    "/api/account/profile",
    EndpointAccess::Authenticated,
    "audit.module.profile",
    BusinessType::Update,
    "user::update_account_profile"
);
pub(super) const PROFILE_PASSWORD: EndpointSpec = operation!(
    Put,
    "/api/account/profile/password",
    EndpointAccess::Authenticated,
    "audit.module.profile",
    BusinessType::Update,
    "user::change_account_password"
);
pub(super) const PROFILE_AVATAR: EndpointSpec = operation!(
    Post,
    "/api/account/profile/avatar",
    EndpointAccess::Authenticated,
    "audit.module.profile",
    BusinessType::Update,
    "user::upload_account_avatar"
);

pub(super) const ONLINE_LIST: EndpointSpec = scoped_permission_read_only!(Get, "/api/system/online/list", "system:online:list", "list_online_sessions");
pub(super) const ONLINE_FORCE_LOGOUT: EndpointSpec = scoped_permission_operation!(
    Delete,
    "/api/system/online/{token_id}",
    "system:online:forceLogout",
    "force_logout_online_session",
    "audit.module.online",
    BusinessType::Force,
    "user::force_logout_online_session"
);

pub(super) const USERS_LIST: EndpointSpec = scoped_permission_read_only!(Get, "/api/system/users", "system:user:list", "list_users");
pub(super) const USERS_CREATE: EndpointSpec = permission_operation!(
    Post,
    "/api/system/users",
    "system:user:add",
    "create_user",
    "audit.module.user",
    BusinessType::Insert,
    "user::create_user"
);
pub(super) const USERS_EXPORT: EndpointSpec = scoped_permission_operation!(
    Post,
    "/api/system/users/export",
    "system:user:export",
    "export_users",
    "audit.module.user",
    BusinessType::Export,
    "user::export_users"
);
pub(super) const USERS_IMPORT: EndpointSpec = permission_operation!(
    Post,
    "/api/system/users/import",
    "system:user:import",
    "import_users",
    "audit.module.user",
    BusinessType::Import,
    "user::import_users"
);
pub(super) const USERS_IMPORT_TEMPLATE: EndpointSpec =
    permission_read_only!(Post, "/api/system/users/import-template", "system:user:import", "user_import_template");
pub(super) const USERS_DEPT_TREE: EndpointSpec = permission_read_only!(Get, "/api/system/users/dept-tree", "system:user:list", "user_dept_tree");
pub(super) const USERS_FORM_OPTIONS: EndpointSpec = permission_read_only!(Get, "/api/system/users/form-options", "system:user:list", "user_form_options");
pub(super) const USERS_DELETE_BATCH: EndpointSpec = scoped_permission_operation!(
    Delete,
    "/api/system/users/batch",
    "system:user:remove",
    "delete_users",
    "audit.module.user",
    BusinessType::Delete,
    "user::delete_users"
);
pub(super) const USER_GET: EndpointSpec = scoped_permission_read_only!(Get, "/api/system/users/{id}", "system:user:query", "get_user");
pub(super) const USER_REPLACE: EndpointSpec = scoped_permission_operation!(
    Put,
    "/api/system/users/{id}",
    "system:user:edit",
    "replace_user",
    "audit.module.user",
    BusinessType::Update,
    "user::replace_user"
);
pub(super) const USER_DELETE: EndpointSpec = scoped_permission_operation!(
    Delete,
    "/api/system/users/{id}",
    "system:user:remove",
    "delete_user",
    "audit.module.user",
    BusinessType::Delete,
    "user::delete_user"
);
pub(super) const USER_RESET_PASSWORD: EndpointSpec = scoped_permission_operation!(
    Put,
    "/api/system/users/{id}/password",
    "system:user:resetPwd",
    "reset_user_password",
    "audit.module.user",
    BusinessType::Update,
    "user::reset_user_password"
);
pub(super) const USER_UPDATE_STATUS: EndpointSpec = scoped_permission_operation!(
    Put,
    "/api/system/users/{id}/status",
    "system:user:edit",
    "update_user_status",
    "audit.module.user",
    BusinessType::Update,
    "user::update_user_status"
);
pub(super) const USER_ROLES: EndpointSpec = scoped_permission_read_only!(Get, "/api/system/users/{id}/roles", "system:user:query", "user_roles");
pub(super) const USER_REPLACE_ROLES: EndpointSpec = scoped_permission_operation!(
    Put,
    "/api/system/users/{id}/roles",
    "system:user:edit",
    "replace_user_roles",
    "audit.module.user",
    BusinessType::Grant,
    "user::replace_user_roles"
);

const SPECS: &[EndpointSpec] = &[
    AUTH_SIGN_UP,
    AUTH_SIGN_IN,
    AUTH_REFRESH,
    AUTH_LOGOUT,
    AUTH_ME,
    PROFILE_GET,
    PROFILE_UPDATE,
    PROFILE_PASSWORD,
    PROFILE_AVATAR,
    ONLINE_LIST,
    ONLINE_FORCE_LOGOUT,
    USERS_LIST,
    USERS_CREATE,
    USERS_EXPORT,
    USERS_IMPORT,
    USERS_IMPORT_TEMPLATE,
    USERS_DEPT_TREE,
    USERS_FORM_OPTIONS,
    USERS_DELETE_BATCH,
    USER_GET,
    USER_REPLACE,
    USER_DELETE,
    USER_RESET_PASSWORD,
    USER_UPDATE_STATUS,
    USER_ROLES,
    USER_REPLACE_ROLES,
];

pub fn endpoint_specs() -> &'static [EndpointSpec] {
    SPECS
}
