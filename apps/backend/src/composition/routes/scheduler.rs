use ::rbac::application::{PermissionRequirement, RoutePermissionRule};

use super::{DELETE, GET, POST, PUT, RouteRuleSpec, from_specs};

pub(super) fn routes() -> Vec<RoutePermissionRule> {
    let mut rules = job_routes();
    rules.extend(job_log_routes());
    rules
}

fn job_routes() -> Vec<RoutePermissionRule> {
    from_specs(&[
        rule_spec!(GET, "/api/system/jobs", "system:job:list", "list_jobs"),
        rule_spec!(POST, "/api/system/jobs/export", "system:job:export", "export_jobs"),
        rule_spec!(GET, "/api/system/jobs/importable", "system:job:import", "importable_tasks"),
        rule_spec!(POST, "/api/system/jobs/import", "system:job:import", "import_job"),
        RouteRuleSpec {
            methods: POST,
            path_pattern: "/api/system/jobs/cron/next-times",
            requirement: PermissionRequirement::any_of(&["system:job:import", "system:job:edit"]),
            handler: "cron_next_times",
        },
        rule_spec!(DELETE, "/api/system/jobs/batch", "system:job:remove", "delete_jobs"),
        rule_spec!(GET, "/api/system/jobs/{id}", "system:job:query", "get_job"),
        rule_spec!(PUT, "/api/system/jobs/{id}", "system:job:edit", "replace_job"),
        rule_spec!(DELETE, "/api/system/jobs/{id}", "system:job:remove", "delete_job"),
        rule_spec!(PUT, "/api/system/jobs/{id}/status", "system:job:changeStatus", "update_job_status"),
        rule_spec!(POST, "/api/system/jobs/{id}/run", "system:job:run", "run_job"),
    ])
}

fn job_log_routes() -> Vec<RoutePermissionRule> {
    from_specs(&[
        rule_spec!(GET, "/api/system/job-logs", "system:job:log:list", "list_job_logs"),
        rule_spec!(POST, "/api/system/job-logs/export", "system:job:log:export", "export_job_logs"),
        rule_spec!(DELETE, "/api/system/job-logs/clean", "system:job:log:remove", "clear_job_logs"),
        rule_spec!(DELETE, "/api/system/job-logs/batch", "system:job:log:remove", "delete_job_logs"),
        RouteRuleSpec {
            methods: GET,
            path_pattern: "/api/system/job-logs/{id}/detail",
            requirement: PermissionRequirement::all_of(&["system:job:log:query", "system:job:log:detail"]),
            handler: "get_job_log_detail",
        },
        rule_spec!(GET, "/api/system/job-logs/{id}", "system:job:log:query", "get_job_log"),
        rule_spec!(DELETE, "/api/system/job-logs/{id}", "system:job:log:remove", "delete_job_log"),
    ])
}
