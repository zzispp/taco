pub const JOB_COLUMNS: &str = r#"
job_id, job_name, job_group, task_key, task_params, params_schema_version,
repeatable, invoke_target, cron_expression, misfire_policy, concurrent, status,
schedule_revision, next_run_at, runtime_error_code, runtime_error_time,
create_by, create_time, update_by, update_time, remark
"#;

pub const EXECUTION_COLUMNS: &str = r#"
execution_id, job_id, job_revision, job_name, job_group, task_key, task_params,
params_schema_version, repeatable, invoke_target, concurrent, trigger_type,
scheduled_at, state, outcome, executor_epoch, requested_by, message_key,
message_params, error_key, error_params, start_time, end_time, create_time
"#;

pub const EXECUTION_LOG_SUMMARY_COLUMNS: &str = r#"
execution_id, job_id, job_name, job_group, task_key, invoke_target, trigger_type,
scheduled_at, outcome, message_key, message_params, error_key, error_params,
start_time, end_time, create_time, (detail_kind IS NOT NULL) AS has_detail
"#;

pub const EXECUTION_LOG_DETAIL_COLUMNS: &str = r#"
execution_id, job_id, job_name, job_group, task_key, invoke_target, trigger_type,
scheduled_at, outcome, message_key, message_params, error_key, error_params,
start_time, end_time, create_time, (detail_kind IS NOT NULL) AS has_detail,
job_revision, requested_by, task_params, detail_kind, detail_schema_version,
detail_payload
"#;

pub const INSERT_EXECUTION: &str = r#"
INSERT INTO sys_job_execution (
    execution_id, job_id, job_revision, job_name, job_group, task_key,
    task_params, params_schema_version, repeatable, invoke_target, concurrent,
    trigger_type, scheduled_at, state, outcome, executor_epoch, requested_by,
    message_key, message_params, error_key, error_params, start_time, end_time, create_time
) VALUES (
    $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,clock_timestamp()
)
"#;

pub const NOTIFY_CHANGE: &str = "SELECT pg_notify('scheduler_changed', $1)";

#[cfg(test)]
mod tests {
    use super::{EXECUTION_LOG_DETAIL_COLUMNS, EXECUTION_LOG_SUMMARY_COLUMNS};

    #[test]
    fn execution_log_summary_projection_excludes_unbounded_detail_payload() {
        assert!(!EXECUTION_LOG_SUMMARY_COLUMNS.contains("detail_payload"));
        assert!(!EXECUTION_LOG_SUMMARY_COLUMNS.contains("task_params"));
        assert!(EXECUTION_LOG_SUMMARY_COLUMNS.contains("has_detail"));
        assert!(EXECUTION_LOG_DETAIL_COLUMNS.contains("detail_payload"));
    }
}
