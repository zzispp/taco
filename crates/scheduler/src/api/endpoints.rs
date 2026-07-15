use audit_contract::{
    BusinessType, EndpointAccess, EndpointAudit, EndpointManifest, EndpointMethod, EndpointPermission, EndpointPermissionRequirement, EndpointSpec,
    OperationEndpointAudit, RequestCapture,
};

const JOBS: &str = "/api/system/jobs";
const JOB_LOGS: &str = "/api/system/job-logs";

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

const fn operation_without_request_capture(title_key: &'static str, business_type: BusinessType, handler: &'static str) -> EndpointAudit {
    EndpointAudit::Operation(OperationEndpointAudit {
        title_key,
        business_type,
        handler,
        request_capture: RequestCapture::None,
    })
}

pub(super) const JOBS_LIST: EndpointSpec = read(
    EndpointMethod::Get,
    JOBS,
    permission("list_jobs", EndpointPermissionRequirement::all_of(&["system:job:list"])),
);
pub(super) const JOBS_EXPORT: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Post,
    path: "/api/system/jobs/export",
    access: permission("export_jobs", EndpointPermissionRequirement::all_of(&["system:job:export"])),
    audit: operation("audit.module.job", BusinessType::Export, "scheduler::export_jobs"),
};
pub(super) const JOBS_IMPORTABLE: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/jobs/importable",
    permission("importable_tasks", EndpointPermissionRequirement::all_of(&["system:job:import"])),
);
pub(super) const JOBS_IMPORT: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Post,
    path: "/api/system/jobs/import",
    access: permission("import_job", EndpointPermissionRequirement::all_of(&["system:job:import"])),
    audit: operation_without_request_capture("audit.module.job", BusinessType::Import, "scheduler::import_job"),
};
pub(super) const JOBS_CRON_NEXT_TIMES: EndpointSpec = read(
    EndpointMethod::Post,
    "/api/system/jobs/cron/next-times",
    permission(
        "cron_next_times",
        EndpointPermissionRequirement::any_of(&["system:job:import", "system:job:edit"]),
    ),
);
pub(super) const JOBS_DELETE_BATCH: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/jobs/batch",
    access: permission("delete_jobs", EndpointPermissionRequirement::all_of(&["system:job:remove"])),
    audit: operation("audit.module.job", BusinessType::Delete, "scheduler::delete_jobs"),
};
pub(super) const JOB_GET: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/jobs/{id}",
    permission("get_job", EndpointPermissionRequirement::all_of(&["system:job:query"])),
);
pub(super) const JOB_REPLACE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Put,
    path: "/api/system/jobs/{id}",
    access: permission("replace_job", EndpointPermissionRequirement::all_of(&["system:job:edit"])),
    audit: operation_without_request_capture("audit.module.job", BusinessType::Update, "scheduler::replace_job"),
};
pub(super) const JOB_DELETE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/jobs/{id}",
    access: permission("delete_job", EndpointPermissionRequirement::all_of(&["system:job:remove"])),
    audit: operation("audit.module.job", BusinessType::Delete, "scheduler::delete_job"),
};
pub(super) const JOB_STATUS: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Put,
    path: "/api/system/jobs/{id}/status",
    access: permission("update_job_status", EndpointPermissionRequirement::all_of(&["system:job:changeStatus"])),
    audit: operation("audit.module.job", BusinessType::Update, "scheduler::update_job_status"),
};
pub(super) const JOB_RUN: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Post,
    path: "/api/system/jobs/{id}/run",
    access: permission("run_job", EndpointPermissionRequirement::all_of(&["system:job:run"])),
    audit: operation("audit.module.job", BusinessType::Update, "scheduler::run_job"),
};

pub(super) const JOB_LOGS_LIST: EndpointSpec = read(
    EndpointMethod::Get,
    JOB_LOGS,
    permission("list_job_logs", EndpointPermissionRequirement::all_of(&["system:job:log:list"])),
);
pub(super) const JOB_LOGS_EXPORT: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Post,
    path: "/api/system/job-logs/export",
    access: permission("export_job_logs", EndpointPermissionRequirement::all_of(&["system:job:log:export"])),
    audit: operation("audit.module.job_log", BusinessType::Export, "scheduler::export_job_logs"),
};
pub(super) const JOB_LOGS_CLEAN: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/job-logs/clean",
    access: permission("clear_job_logs", EndpointPermissionRequirement::all_of(&["system:job:log:remove"])),
    audit: operation("audit.module.job_log", BusinessType::Clean, "scheduler::clear_job_logs"),
};
pub(super) const JOB_LOGS_DELETE_BATCH: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/job-logs/batch",
    access: permission("delete_job_logs", EndpointPermissionRequirement::all_of(&["system:job:log:remove"])),
    audit: operation("audit.module.job_log", BusinessType::Delete, "scheduler::delete_job_logs"),
};
pub(super) const JOB_LOG_DETAIL: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/job-logs/{id}/detail",
    permission(
        "get_job_log_detail",
        EndpointPermissionRequirement::all_of(&["system:job:log:query", "system:job:log:detail"]),
    ),
);
pub(super) const JOB_LOG_GET: EndpointSpec = read(
    EndpointMethod::Get,
    "/api/system/job-logs/{id}",
    permission("get_job_log", EndpointPermissionRequirement::all_of(&["system:job:log:query"])),
);
pub(super) const JOB_LOG_DELETE: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Delete,
    path: "/api/system/job-logs/{id}",
    access: permission("delete_job_log", EndpointPermissionRequirement::all_of(&["system:job:log:remove"])),
    audit: operation("audit.module.job_log", BusinessType::Delete, "scheduler::delete_job_log"),
};

const ENDPOINTS: &[EndpointSpec] = &[
    JOBS_LIST,
    JOBS_EXPORT,
    JOBS_IMPORTABLE,
    JOBS_IMPORT,
    JOBS_CRON_NEXT_TIMES,
    JOBS_DELETE_BATCH,
    JOB_GET,
    JOB_REPLACE,
    JOB_DELETE,
    JOB_STATUS,
    JOB_RUN,
    JOB_LOGS_LIST,
    JOB_LOGS_EXPORT,
    JOB_LOGS_CLEAN,
    JOB_LOGS_DELETE_BATCH,
    JOB_LOG_DETAIL,
    JOB_LOG_GET,
    JOB_LOG_DELETE,
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
    fn endpoint_specs_cover_scheduler_routes_and_keep_cron_preview_read_only() {
        let specs = endpoint_specs();
        let entries = specs.iter().collect::<Vec<_>>();

        specs.validate().unwrap();
        assert_eq!(entries.len(), 18);
        assert_eq!(entries.iter().filter(|spec| matches!(spec.audit, EndpointAudit::Operation(_))).count(), 11);
        let cron_preview = entries
            .iter()
            .find(|spec| spec.method == EndpointMethod::Post && spec.path == "/api/system/jobs/cron/next-times")
            .unwrap();
        assert_eq!(cron_preview.audit, EndpointAudit::ExplicitReadOnly);
    }

    #[test]
    fn job_definition_writes_do_not_capture_http_task_configuration_in_the_operation_outbox() {
        let writes = endpoint_specs()
            .iter()
            .filter(|spec| {
                matches!(
                    (spec.method, spec.path),
                    (EndpointMethod::Post, "/api/system/jobs/import") | (EndpointMethod::Put, "/api/system/jobs/{id}")
                )
            })
            .filter_map(|spec| match spec.audit {
                EndpointAudit::Operation(operation) => Some(operation),
                EndpointAudit::ReadOnly | EndpointAudit::ExplicitReadOnly | EndpointAudit::Security => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(writes.len(), 2);
        assert!(writes.iter().all(|operation| operation.request_capture == RequestCapture::None));
    }
}
