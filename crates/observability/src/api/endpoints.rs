use audit_contract::{
    BusinessType, EndpointAccess, EndpointAudit, EndpointManifest, EndpointMethod, EndpointPermission, EndpointPermissionRequirement, EndpointSpec,
    OperationEndpointAudit, RequestCapture,
};

const SYSTEM_LOGS: &str = "/api/system/system-logs";

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

const fn operation(business_type: BusinessType, handler: &'static str) -> EndpointAudit {
    EndpointAudit::Operation(OperationEndpointAudit {
        title_key: "observability.module.system_log",
        business_type,
        handler,
        request_capture: RequestCapture::Sanitized,
    })
}

pub(in crate::api) const SYSTEM_LOGS_LIST: EndpointSpec = read(
    EndpointMethod::Get,
    SYSTEM_LOGS,
    permission("list_system_logs", EndpointPermissionRequirement::all_of(&["system:systemlog:list"])),
);
pub(in crate::api) const SYSTEM_LOGS_EXPORT: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Post,
    path: "/api/system/system-logs/export",
    access: permission("export_system_logs", EndpointPermissionRequirement::all_of(&["system:systemlog:export"])),
    audit: operation(BusinessType::Export, "observability::export_system_logs"),
};
pub(in crate::api) const SYSTEM_LOGS_CLEAN_COUNT: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/system-logs/clean/count",
    permission(
        "count_system_logs_for_cleanup",
        EndpointPermissionRequirement::all_of(&["system:systemlog:remove"]),
    ),
);
pub(in crate::api) const SYSTEM_LOGS_CLEAN: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/system-logs/clean",
    access: permission("clean_system_logs", EndpointPermissionRequirement::all_of(&["system:systemlog:remove"])),
    audit: operation(BusinessType::Clean, "observability::clean_system_logs"),
};
pub(in crate::api) const SYSTEM_LOGS_CLEAN_EXECUTION: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/system-logs/clean/executions/{execution_id}",
    permission(
        "get_system_log_cleanup_execution",
        EndpointPermissionRequirement::all_of(&["system:systemlog:remove"]),
    ),
);
pub(in crate::api) const SYSTEM_LOGS_DELETE_BATCH: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/system-logs/batch",
    access: permission("delete_system_logs", EndpointPermissionRequirement::all_of(&["system:systemlog:remove"])),
    audit: operation(BusinessType::Delete, "observability::delete_system_logs"),
};
pub(in crate::api) const SYSTEM_LOG_GET: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/system-logs/{id}",
    permission("get_system_log", EndpointPermissionRequirement::all_of(&["system:systemlog:query"])),
);
pub(in crate::api) const SYSTEM_LOG_DELETE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/system-logs/{id}",
    access: permission("delete_system_log", EndpointPermissionRequirement::all_of(&["system:systemlog:remove"])),
    audit: operation(BusinessType::Delete, "observability::delete_system_log"),
};

const ENDPOINTS: &[EndpointSpec] = &[
    SYSTEM_LOGS_LIST,
    SYSTEM_LOGS_EXPORT,
    SYSTEM_LOGS_CLEAN_COUNT,
    SYSTEM_LOGS_CLEAN,
    SYSTEM_LOGS_CLEAN_EXECUTION,
    SYSTEM_LOGS_DELETE_BATCH,
    SYSTEM_LOG_GET,
    SYSTEM_LOG_DELETE,
];

pub fn endpoint_specs() -> EndpointManifest {
    EndpointManifest::new(&[ENDPOINTS])
}

#[cfg(test)]
mod tests {
    use audit_contract::{EndpointAudit, EndpointMethod};

    use super::endpoint_specs;

    #[test]
    fn endpoints_cover_all_system_log_management_operations() {
        let specs = endpoint_specs();
        let entries = specs.iter().collect::<Vec<_>>();

        specs.validate().unwrap();
        assert_eq!(entries.len(), 8);
        assert_eq!(entries.iter().filter(|spec| matches!(spec.audit, EndpointAudit::Operation(_))).count(), 4);
        assert!(
            entries
                .iter()
                .any(|spec| spec.method == EndpointMethod::Get && spec.path == "/api/system/system-logs/clean/count")
        );
    }
}
