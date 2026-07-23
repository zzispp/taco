use audit_contract::{
    BusinessType, EndpointAccess, EndpointAudit, EndpointManifest, EndpointMethod, EndpointPermission, EndpointPermissionRequirement, EndpointSpec,
    OperationEndpointAudit, RequestCapture,
};

const OPERATION_LOGS: &str = "/api/system/operation-logs";
const LOGIN_LOGS: &str = "/api/system/login-logs";

const fn permission(handler: &'static str, requirement: EndpointPermissionRequirement) -> EndpointAccess {
    EndpointAccess::Permission(EndpointPermission { handler, requirement })
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

pub(in crate::api) const OPERATION_LOGS_LIST: EndpointSpec = read(
    EndpointMethod::Get,
    OPERATION_LOGS,
    permission("list_operation_logs", EndpointPermissionRequirement::all_of(&["system:operlog:list"])),
);
pub(in crate::api) const OPERATION_LOGS_EXPORT: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Post,
    path: "/api/system/operation-logs/export",
    access: permission("export_operation_logs", EndpointPermissionRequirement::all_of(&["system:operlog:export"])),
    audit: operation("audit.module.operation_log", BusinessType::Export, "audit::export_operation_logs"),
};
pub(in crate::api) const OPERATION_LOGS_CLEAN: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/operation-logs/clean",
    access: permission("clear_operation_logs", EndpointPermissionRequirement::all_of(&["system:operlog:remove"])),
    audit: operation("audit.module.operation_log", BusinessType::Clean, "audit::clear_operation_logs"),
};
pub(in crate::api) const OPERATION_LOGS_DELETE_BATCH: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/operation-logs/batch",
    access: permission("delete_operation_logs", EndpointPermissionRequirement::all_of(&["system:operlog:remove"])),
    audit: operation("audit.module.operation_log", BusinessType::Delete, "audit::delete_operation_logs"),
};
pub(in crate::api) const OPERATION_LOG_GET: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/operation-logs/{id}",
    permission("get_operation_log", EndpointPermissionRequirement::all_of(&["system:operlog:query"])),
);
pub(in crate::api) const OPERATION_LOG_DELETE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/operation-logs/{id}",
    access: permission("delete_operation_log", EndpointPermissionRequirement::all_of(&["system:operlog:remove"])),
    audit: operation("audit.module.operation_log", BusinessType::Delete, "audit::delete_operation_log"),
};

pub(in crate::api) const LOGIN_LOGS_LIST: EndpointSpec = read(
    EndpointMethod::Get,
    LOGIN_LOGS,
    permission("list_login_logs", EndpointPermissionRequirement::all_of(&["system:logininfor:list"])),
);
pub(in crate::api) const LOGIN_LOGS_EXPORT: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Post,
    path: "/api/system/login-logs/export",
    access: permission("export_login_logs", EndpointPermissionRequirement::all_of(&["system:logininfor:export"])),
    audit: operation("audit.module.login_log", BusinessType::Export, "audit::export_login_logs"),
};
pub(in crate::api) const LOGIN_LOGS_CLEAN: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/login-logs/clean",
    access: permission("clear_login_logs", EndpointPermissionRequirement::all_of(&["system:logininfor:remove"])),
    audit: operation("audit.module.login_log", BusinessType::Clean, "audit::clear_login_logs"),
};
pub(in crate::api) const LOGIN_LOGS_DELETE_BATCH: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/login-logs/batch",
    access: permission("delete_login_logs", EndpointPermissionRequirement::all_of(&["system:logininfor:remove"])),
    audit: operation("audit.module.login_log", BusinessType::Delete, "audit::delete_login_logs"),
};
pub(in crate::api) const LOGIN_LOG_DELETE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/login-logs/{id}",
    access: permission("delete_login_log", EndpointPermissionRequirement::all_of(&["system:logininfor:remove"])),
    audit: operation("audit.module.login_log", BusinessType::Delete, "audit::delete_login_log"),
};
pub(in crate::api) const LOGIN_UNLOCK: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Put,
    path: "/api/system/login-logs/{username}/unlock",
    access: permission("unlock_login", EndpointPermissionRequirement::all_of(&["system:logininfor:unlock"])),
    audit: operation("audit.module.login_log", BusinessType::Other, "audit::unlock_login"),
};

const ENDPOINTS: &[EndpointSpec] = &[
    OPERATION_LOGS_LIST,
    OPERATION_LOGS_EXPORT,
    OPERATION_LOGS_CLEAN,
    OPERATION_LOGS_DELETE_BATCH,
    OPERATION_LOG_GET,
    OPERATION_LOG_DELETE,
    LOGIN_LOGS_LIST,
    LOGIN_LOGS_EXPORT,
    LOGIN_LOGS_CLEAN,
    LOGIN_LOGS_DELETE_BATCH,
    LOGIN_LOG_DELETE,
    LOGIN_UNLOCK,
];

const SEGMENTS: &[&[EndpointSpec]] = &[ENDPOINTS];

pub fn endpoint_specs() -> EndpointManifest {
    EndpointManifest::new(SEGMENTS)
}

#[cfg(test)]
mod tests {
    use audit_contract::{EndpointAudit, EndpointMethod, RequestCapture};

    use super::endpoint_specs;

    #[test]
    fn endpoint_specs_cover_all_audit_management_methods() {
        let specs = endpoint_specs();
        let entries = specs.iter().collect::<Vec<_>>();

        specs.validate().unwrap();
        assert_eq!(entries.len(), 12);
        assert_eq!(entries.iter().filter(|spec| matches!(spec.audit, EndpointAudit::Operation(_))).count(), 9);
        assert!(
            entries
                .iter()
                .any(|spec| { spec.method == EndpointMethod::Get && spec.path == "/api/system/operation-logs/{id}" })
        );
        assert!(
            entries
                .iter()
                .any(|spec| { spec.method == EndpointMethod::Delete && spec.path == "/api/system/operation-logs/{id}" })
        );
        assert!(
            entries
                .iter()
                .filter_map(|spec| match spec.audit {
                    EndpointAudit::Operation(operation) | EndpointAudit::Download(operation) => Some(operation),
                    EndpointAudit::ReadOnly | EndpointAudit::ExplicitReadOnly | EndpointAudit::Security => None,
                })
                .all(|operation| operation.request_capture == RequestCapture::Sanitized)
        );
    }
}
