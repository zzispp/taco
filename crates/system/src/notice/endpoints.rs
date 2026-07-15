use audit_contract::{
    BusinessType, EndpointAccess, EndpointAudit, EndpointManifest, EndpointMethod, EndpointPermission, EndpointPermissionRequirement, EndpointSpec,
    OperationEndpointAudit, RequestCapture,
};

const NOTICES: &str = "/api/system/notices";

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

pub(in crate::notice) const NOTICES_LIST: EndpointSpec = read(
    EndpointMethod::Get,
    NOTICES,
    permission("list_notices", EndpointPermissionRequirement::all_of(&["system:notice:list"])),
);
pub(in crate::notice) const NOTICES_CREATE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Post,
    path: NOTICES,
    access: permission("create_notice", EndpointPermissionRequirement::all_of(&["system:notice:add"])),
    audit: operation("audit.module.notice", BusinessType::Insert, "system::create_notice"),
};
pub(in crate::notice) const NOTICES_TOP: EndpointSpec = read(EndpointMethod::Get, "/api/system/notices/top", EndpointAccess::Authenticated);
pub(in crate::notice) const NOTICES_READ_ALL: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Put,
    path: "/api/system/notices/read-all",
    access: EndpointAccess::Authenticated,
    audit: operation("audit.module.notice", BusinessType::Update, "system::mark_all_notices_read"),
};
pub(in crate::notice) const NOTICES_DELETE_BATCH: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/notices/batch",
    access: permission("delete_notices", EndpointPermissionRequirement::all_of(&["system:notice:remove"])),
    audit: operation("audit.module.notice", BusinessType::Delete, "system::delete_notices"),
};
pub(in crate::notice) const NOTICE_READ: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Put,
    path: "/api/system/notices/{id}/read",
    access: EndpointAccess::Authenticated,
    audit: operation("audit.module.notice", BusinessType::Update, "system::mark_notice_read"),
};
pub(in crate::notice) const NOTICE_READERS: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/notices/{id}/readers",
    permission("list_notice_readers", EndpointPermissionRequirement::all_of(&["system:notice:list"])),
);
pub(in crate::notice) const NOTICE_GET: EndpointSpec = read(EndpointMethod::Get, "/api/system/notices/{id}", EndpointAccess::Authenticated);
pub(in crate::notice) const NOTICE_REPLACE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Put,
    path: "/api/system/notices/{id}",
    access: permission("replace_notice", EndpointPermissionRequirement::all_of(&["system:notice:edit"])),
    audit: operation("audit.module.notice", BusinessType::Update, "system::replace_notice"),
};
pub(in crate::notice) const NOTICE_DELETE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/notices/{id}",
    access: permission("delete_notice", EndpointPermissionRequirement::all_of(&["system:notice:remove"])),
    audit: operation("audit.module.notice", BusinessType::Delete, "system::delete_notice"),
};

const ENDPOINTS: &[EndpointSpec] = &[
    NOTICES_LIST,
    NOTICES_CREATE,
    NOTICES_TOP,
    NOTICES_READ_ALL,
    NOTICES_DELETE_BATCH,
    NOTICE_READ,
    NOTICE_READERS,
    NOTICE_GET,
    NOTICE_REPLACE,
    NOTICE_DELETE,
];

const SEGMENTS: &[&[EndpointSpec]] = &[ENDPOINTS];

pub fn endpoint_specs() -> EndpointManifest {
    EndpointManifest::new(SEGMENTS)
}

#[cfg(test)]
mod tests {
    use audit_contract::{EndpointAccess, EndpointAudit, EndpointMethod};

    use super::endpoint_specs;

    #[test]
    fn endpoint_specs_cover_notice_routes_and_read_actions() {
        let specs = endpoint_specs();
        let entries = specs.iter().collect::<Vec<_>>();

        specs.validate().unwrap();
        assert_eq!(entries.len(), 10);
        assert_eq!(entries.iter().filter(|spec| matches!(spec.audit, EndpointAudit::Operation(_))).count(), 6);
        assert!(
            entries.iter().any(|spec| {
                spec.method == EndpointMethod::Put && spec.path == "/api/system/notices/read-all" && spec.access == EndpointAccess::Authenticated
            })
        );
        assert!(entries.iter().any(|spec| {
            spec.method == EndpointMethod::Put && spec.path == "/api/system/notices/{id}/read" && spec.access == EndpointAccess::Authenticated
        }));
    }
}
