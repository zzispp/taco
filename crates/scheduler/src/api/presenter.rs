use chrono::{DateTime, Utc};
use types::http::{Locale, format_utc_rfc3339_millis, translate_message, translate_message_with_params};

use crate::{
    application::{ExecutionLogSummary, ImportableTask, JobView, SchedulerError, SchedulerResult, tasks::sanitize_http_invoke_target},
    domain::{LocalizedMessage, RuntimeErrorState, TaskParamFormSpec},
};

use super::dto::{ExecutionLogResponse, ImportableTaskResponse, JobResponse, ParamFieldResponse, ParamUiResponse, RuntimeErrorResponse, TaskParamFormResponse};
use super::presentation::{concurrent_policy, execution_outcome, job_status, registry_status, runtime_error, trigger_type};

pub fn job_response(view: JobView, locale: Locale) -> SchedulerResult<JobResponse> {
    let job = view.job;
    let runtime_error = job.runtime_error.map(|error| runtime_error_response(error, locale)).transpose()?;
    let invoke_target = sanitize_http_invoke_target(&job.task_key, &job.invoke_target);
    Ok(JobResponse {
        job_id: job.id,
        job_name: job.name,
        job_group: job.group,
        task_key: job.task_key,
        task_params: job.task_params,
        params_schema_version: job.params_schema_version,
        repeatable: job.repeatable,
        invoke_target,
        cron_expression: job.cron_expression,
        misfire_policy: job.misfire_policy.code().into(),
        concurrent: concurrent_policy(job.concurrent).wire_value.into(),
        status: job_status(job.status).wire_value.into(),
        registry_status: registry_status(view.registry_status).wire_value.into(),
        param_form: view.param_form.map(|form| param_form_response(form, locale)),
        schedule_revision: job.schedule_revision,
        next_run_at: job.next_run_at.map(rfc3339).transpose()?,
        runtime_error,
        create_by: job.create_by,
        create_time: rfc3339(job.create_time)?,
        update_by: job.update_by,
        update_time: job.update_time.map(rfc3339).transpose()?,
        remark: job.remark,
    })
}

pub fn importable_response(task: ImportableTask, locale: Locale) -> ImportableTaskResponse {
    ImportableTaskResponse {
        task_key: task.task_key.into(),
        name: translate_message(locale, task.name_key),
        group: task.group.into(),
        group_label: translate_message(locale, task.group_key),
        description: translate_message(locale, task.description_key),
        repeatable: task.repeatable,
        default_params: task.default_params,
        param_form: param_form_response(task.param_form, locale),
    }
}

pub fn execution_response(summary: ExecutionLogSummary, locale: Locale) -> SchedulerResult<ExecutionLogResponse> {
    let invoke_target = sanitize_http_invoke_target(&summary.task_key, &summary.invoke_target);
    Ok(ExecutionLogResponse {
        execution_id: summary.id,
        job_id: summary.job_id,
        job_name: summary.job_name,
        job_group: summary.job_group,
        task_key: summary.task_key,
        invoke_target,
        has_detail: summary.has_detail,
        job_message: localized_message(locale, &summary.message),
        trigger_type: trigger_type(summary.trigger).wire_value.into(),
        scheduled_at: rfc3339(summary.scheduled_at)?,
        status: execution_outcome(summary.outcome).wire_value.into(),
        exception_info: summary.error.as_ref().map(|error| localized_message(locale, error)),
        start_time: summary.start_time.map(rfc3339).transpose()?,
        end_time: rfc3339(summary.end_time)?,
        create_time: rfc3339(summary.create_time)?,
    })
}

pub fn param_form_response(form: TaskParamFormSpec, locale: Locale) -> TaskParamFormResponse {
    let fields = form
        .ui
        .fields
        .into_iter()
        .map(|field| ParamFieldResponse {
            path: field.path,
            label: translate_message(locale, &field.label_key),
            widget: field.widget,
            placeholder: field.placeholder_key.map(|key| translate_message(locale, &key)),
            help: field.help_key.map(|key| translate_message(locale, &key)),
            options: field.options,
            disabled_when: field.disabled_when,
        })
        .collect();
    TaskParamFormResponse {
        schema_version: form.schema_version,
        schema: form.schema,
        ui: ParamUiResponse { fields },
    }
}

pub fn localized_message(locale: Locale, message: &LocalizedMessage) -> String {
    let params = message.params.iter().map(|(key, value)| (key.as_str(), value.clone())).collect::<Vec<_>>();
    translate_message_with_params(locale, &message.key, &params)
}

fn runtime_error_response(error: RuntimeErrorState, locale: Locale) -> SchedulerResult<RuntimeErrorResponse> {
    let presentation = runtime_error(error.code);
    Ok(RuntimeErrorResponse {
        code: presentation.wire_value.into(),
        message: presentation.localized(locale),
        occurred_at: rfc3339(error.occurred_at)?,
    })
}

pub(super) fn rfc3339(value: DateTime<Utc>) -> SchedulerResult<String> {
    let nanos = i128::from(value.timestamp()) * 1_000_000_000 + i128::from(value.timestamp_subsec_nanos());
    let value = time::OffsetDateTime::from_unix_timestamp_nanos(nanos)
        .map_err(|error| SchedulerError::Infrastructure(format!("scheduler wire timestamp is out of range: {error}")))?;
    format_utc_rfc3339_millis(value).map_err(|error| SchedulerError::Infrastructure(format!("scheduler wire timestamp formatting failed: {error}")))
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use serde_json::json;
    use types::http::Locale;

    use crate::{
        application::{ExecutionLogSummary, JobView},
        domain::{
            ConcurrentPolicy, ExecutionOutcome, Job, JobStatus, LocalizedMessage, MisfirePolicy, RegistryStatus, RuntimeErrorCode, RuntimeErrorState,
            TriggerType,
        },
    };

    use super::{execution_response, job_response};

    #[test]
    fn terminal_log_response_keeps_nullable_start_time_and_localizes_message() {
        let response = execution_response(execution_summary(), Locale::En).unwrap();

        assert_eq!(response.execution_id, "execution-1");
        assert_eq!(response.trigger_type, "misfire");
        assert_eq!(response.status, "2");
        assert_eq!(response.job_message, "Skipped because the occurrence was missed and the policy is do nothing");
        assert!(response.has_detail);
        assert_eq!(response.start_time, None);
        assert_eq!(response.end_time, "2026-07-10T08:31:00.000Z");
        let value = serde_json::to_value(response).unwrap();
        assert_eq!(value.get("task_params"), None);
        assert_eq!(value.get("detail"), None);
        assert_eq!(value.get("payload"), None);
    }

    #[test]
    fn runtime_error_response_uses_the_typed_domain_code() {
        let mut view = job_view();
        view.job.runtime_error = Some(RuntimeErrorState {
            code: RuntimeErrorCode::InvalidCron,
            occurred_at: fixed_time("2026-07-10T08:30:00Z"),
        });

        let response = job_response(view, Locale::En).unwrap().runtime_error.unwrap();

        assert_eq!(response.code, "invalid_cron");
        assert_eq!(response.occurred_at, "2026-07-10T08:30:00.000Z");
    }

    fn execution_summary() -> ExecutionLogSummary {
        ExecutionLogSummary {
            id: "execution-1".into(),
            job_id: "job-1".into(),
            job_name: "job".into(),
            job_group: "system".into(),
            task_key: "task".into(),
            invoke_target: "task".into(),
            trigger: TriggerType::Misfire,
            scheduled_at: fixed_time("2026-07-10T08:30:00Z"),
            outcome: ExecutionOutcome::Skipped,
            message: LocalizedMessage::new("scheduler.execution.skipped_misfire"),
            error: None,
            start_time: None,
            end_time: fixed_time("2026-07-10T08:31:00Z"),
            create_time: fixed_time("2026-07-10T08:30:00Z"),
            has_detail: true,
        }
    }

    fn job_view() -> JobView {
        JobView {
            job: Job {
                id: "job-1".into(),
                name: "job".into(),
                group: "system".into(),
                task_key: "task".into(),
                task_params: json!({}),
                params_schema_version: 1,
                repeatable: false,
                invoke_target: "task".into(),
                cron_expression: "0 * * * * *".into(),
                misfire_policy: MisfirePolicy::DoNothing,
                concurrent: ConcurrentPolicy::Disallow,
                status: JobStatus::Normal,
                schedule_revision: 1,
                next_run_at: None,
                runtime_error: None,
                create_by: "tester".into(),
                create_time: fixed_time("2026-07-10T08:30:00Z"),
                update_by: "tester".into(),
                update_time: None,
                remark: None,
            },
            registry_status: RegistryStatus::Ok,
            param_form: None,
        }
    }

    fn fixed_time(value: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(value).unwrap().with_timezone(&Utc)
    }
}
