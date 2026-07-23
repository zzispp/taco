use audit_contract::{
    BusinessType, EndpointAccess, EndpointAudit, EndpointManifest, EndpointMethod, EndpointPermission, EndpointPermissionRequirement, EndpointSpec,
    OperationEndpointAudit, RequestCapture,
};

const fn scoped(handler: &'static str, permission: &'static [&'static str]) -> EndpointAccess {
    EndpointAccess::DataScopedPermission(EndpointPermission {
        handler,
        requirement: EndpointPermissionRequirement::all_of(permission),
    })
}

const fn permission(handler: &'static str, value: &'static [&'static str]) -> EndpointAccess {
    EndpointAccess::Permission(EndpointPermission {
        handler,
        requirement: EndpointPermissionRequirement::all_of(value),
    })
}

const fn scoped_any(handler: &'static str, values: &'static [&'static str]) -> EndpointAccess {
    EndpointAccess::DataScopedPermission(EndpointPermission {
        handler,
        requirement: EndpointPermissionRequirement::any_of(values),
    })
}

const fn read(path: &'static str, access: EndpointAccess) -> EndpointSpec {
    EndpointSpec {
        method: EndpointMethod::Get,
        path,
        access,
        audit: EndpointAudit::ReadOnly,
    }
}

const fn download(path: &'static str, access: EndpointAccess, handler: &'static str) -> EndpointSpec {
    EndpointSpec {
        method: EndpointMethod::Get,
        path,
        access,
        audit: EndpointAudit::Download(OperationEndpointAudit {
            title_key: "audit.module.file",
            business_type: BusinessType::Export,
            handler,
            request_capture: RequestCapture::None,
        }),
    }
}

const fn write(method: EndpointMethod, path: &'static str, access: EndpointAccess, business_type: BusinessType, handler: &'static str) -> EndpointSpec {
    EndpointSpec {
        method,
        path,
        access,
        audit: EndpointAudit::Operation(OperationEndpointAudit {
            title_key: "audit.module.file",
            business_type,
            handler,
            request_capture: RequestCapture::Sanitized,
        }),
    }
}

const fn quiet_write(method: EndpointMethod, path: &'static str, access: EndpointAccess) -> EndpointSpec {
    EndpointSpec {
        method,
        path,
        access,
        audit: EndpointAudit::ExplicitReadOnly,
    }
}

pub(super) const FILES_LIST: EndpointSpec = read("/api/system/files", scoped("list_files", &["file:asset:list"]));
pub(super) const FILE_DIRECTORY_TRAIL: EndpointSpec = read("/api/system/files/{id}/directory-trail", scoped("file_directory_trail", &["file:asset:list"]));
pub(super) const FILES_OVERVIEW: EndpointSpec = read("/api/system/files/overview", scoped("file_overview", &["file:asset:query"]));
pub(super) const FILE_SPACES_LIST: EndpointSpec = read("/api/system/file-spaces", scoped("list_file_spaces", &["file:space:list"]));
pub(super) const FILE_SPACE_UPDATE: EndpointSpec = write(
    EndpointMethod::Put,
    "/api/system/file-spaces/{id}",
    scoped("update_file_space", &["file:space:quota"]),
    BusinessType::Update,
    "file::update_space",
);
pub(super) const FILE_GET: EndpointSpec = read("/api/system/files/{id}", scoped("get_file", &["file:asset:query"]));
pub(super) const FILE_UPDATE: EndpointSpec = write(
    EndpointMethod::Put,
    "/api/system/files/{id}",
    scoped("update_file", &["file:asset:edit"]),
    BusinessType::Update,
    "file::update_entry",
);
pub(super) const FOLDER_CREATE: EndpointSpec = write(
    EndpointMethod::Post,
    "/api/system/files/folders",
    scoped("create_file_folder", &["file:folder:add"]),
    BusinessType::Insert,
    "file::create_folder",
);
pub(super) const FILE_CONTENT: EndpointSpec = download(
    "/api/system/files/{id}/content",
    scoped("download_file", &["file:asset:download"]),
    "file::download",
);
pub(super) const FILE_PREVIEW: EndpointSpec = read("/api/system/files/{id}/preview", scoped("preview_file", &["file:asset:query"]));
pub(super) const FILE_THUMBNAIL: EndpointSpec = read("/api/system/files/{id}/thumbnail", scoped("thumbnail_file", &["file:asset:query"]));
pub(super) const FILE_TRASH: EndpointSpec = write(
    EndpointMethod::Post,
    "/api/system/files/{id}/trash",
    scoped("trash_file", &["file:asset:remove"]),
    BusinessType::Delete,
    "file::trash",
);
pub(super) const FILE_RESTORE: EndpointSpec = write(
    EndpointMethod::Post,
    "/api/system/files/{id}/restore",
    scoped("restore_file", &["file:asset:restore"]),
    BusinessType::Update,
    "file::restore",
);
pub(super) const FILE_PURGE: EndpointSpec = write(
    EndpointMethod::Delete,
    "/api/system/files/{id}/purge",
    scoped("purge_file", &["file:asset:purge"]),
    BusinessType::Delete,
    "file::purge",
);
pub(super) const FILES_TRASH_BATCH: EndpointSpec = write(
    EndpointMethod::Post,
    "/api/system/files/trash/batch",
    scoped("trash_files", &["file:asset:remove"]),
    BusinessType::Delete,
    "file::trash_batch",
);
pub(super) const FILES_RESTORE_BATCH: EndpointSpec = write(
    EndpointMethod::Post,
    "/api/system/files/trash/restore/batch",
    scoped("restore_files", &["file:asset:restore"]),
    BusinessType::Update,
    "file::restore_batch",
);
pub(super) const FILES_PURGE_BATCH: EndpointSpec = write(
    EndpointMethod::Post,
    "/api/system/files/trash/purge/batch",
    scoped("purge_files", &["file:asset:purge"]),
    BusinessType::Delete,
    "file::purge_batch",
);
pub(super) const PROVIDERS_LIST: EndpointSpec = read("/api/system/file-providers", permission("list_file_providers", &["file:provider:query"]));

pub(super) const UPLOAD_SESSIONS_CREATE: EndpointSpec = quiet_write(
    EndpointMethod::Post,
    "/api/system/file-upload-sessions",
    scoped("create_upload_session", &["file:asset:upload"]),
);
pub(super) const UPLOAD_SESSION_GET: EndpointSpec = read(
    "/api/system/file-upload-sessions/{id}",
    scoped_any("get_upload_session", &["file:asset:upload", "file:upload:manage"]),
);
pub(super) const UPLOAD_SESSION_PART: EndpointSpec = quiet_write(
    EndpointMethod::Put,
    "/api/system/file-upload-sessions/{id}/parts/{part_number}",
    scoped("write_upload_part", &["file:asset:upload"]),
);
pub(super) const UPLOAD_SESSION_COMPLETE: EndpointSpec = write(
    EndpointMethod::Post,
    "/api/system/file-upload-sessions/{id}/complete",
    scoped("complete_upload_session", &["file:asset:upload"]),
    BusinessType::Import,
    "file::complete_upload",
);
pub(super) const UPLOAD_SESSION_CANCEL: EndpointSpec = write(
    EndpointMethod::Delete,
    "/api/system/file-upload-sessions/{id}",
    scoped_any("cancel_upload_session", &["file:asset:upload", "file:upload:manage"]),
    BusinessType::Delete,
    "file::cancel_upload",
);

const ENDPOINTS: &[EndpointSpec] = &[
    FILES_LIST,
    FILE_DIRECTORY_TRAIL,
    FILES_OVERVIEW,
    FILE_SPACES_LIST,
    FILE_SPACE_UPDATE,
    FILE_GET,
    FILE_UPDATE,
    FOLDER_CREATE,
    FILE_CONTENT,
    FILE_PREVIEW,
    FILE_THUMBNAIL,
    FILE_TRASH,
    FILE_RESTORE,
    FILE_PURGE,
    FILES_TRASH_BATCH,
    FILES_RESTORE_BATCH,
    FILES_PURGE_BATCH,
    PROVIDERS_LIST,
    UPLOAD_SESSIONS_CREATE,
    UPLOAD_SESSION_GET,
    UPLOAD_SESSION_PART,
    UPLOAD_SESSION_COMPLETE,
    UPLOAD_SESSION_CANCEL,
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
    fn file_endpoint_manifest_covers_core_and_resumable_routes() {
        let manifest = endpoint_specs();
        manifest.validate().unwrap();
        assert_eq!(manifest.iter().count(), 23);
        let directory_trail = manifest
            .iter()
            .copied()
            .find(|spec| spec.path == "/api/system/files/{id}/directory-trail")
            .unwrap();
        assert_eq!(directory_trail.method, EndpointMethod::Get);
        assert_eq!(directory_trail.audit, EndpointAudit::ReadOnly);
        let EndpointAccess::DataScopedPermission(permission) = directory_trail.access else {
            panic!("directory trail must use a data-scoped permission");
        };
        assert_eq!(permission.handler, "file_directory_trail");
        assert_eq!(permission.requirement.values(), &["file:asset:list"]);
        assert!(matches!(
            manifest
                .iter()
                .find(|spec| spec.path == "/api/system/files/{id}/content")
                .map(|spec| spec.audit),
            Some(EndpointAudit::Download(_))
        ));
        assert!(matches!(
            manifest
                .iter()
                .find(|spec| spec.path == "/api/system/files/{id}/preview")
                .map(|spec| spec.audit),
            Some(EndpointAudit::ReadOnly)
        ));
        assert!(matches!(
            manifest
                .iter()
                .find(|spec| spec.path == "/api/system/files/{id}/thumbnail")
                .map(|spec| spec.audit),
            Some(EndpointAudit::ReadOnly)
        ));
        assert!(!manifest.iter().any(|spec| spec.path == "/api/system/files/upload"));
        assert!(matches!(
            manifest
                .iter()
                .find(|spec| spec.path == "/api/system/file-upload-sessions")
                .map(|spec| spec.audit),
            Some(EndpointAudit::ExplicitReadOnly)
        ));
    }
}
