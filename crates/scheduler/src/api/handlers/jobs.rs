use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::Response,
};
use kernel::pagination::CursorPage;
use rbac::api::CurrentUser;
use rbac_macros::{require_any_perms, require_perms};
use types::http::{RequestJson, RequestQuery, current_locale, xlsx_file_attachment};

use super::support::successful_operation_audit;
use crate::api::{
    SchedulerApiError, SchedulerApiState,
    dto::{
        BatchIdsRequest, CronNextTimesRequest, CronNextTimesResponse, ImportJobRequest, ImportableTaskResponse, JobExportQuery, JobListQuery, JobResponse,
        ReplaceJobRequest, RunJobResponse, UpdateJobStatusRequest,
    },
    export::{ExportRequest, export_jobs as build_export_jobs},
    input::{import_command, job_export_filter, job_filter, page_request, replace_command, status_command},
    presenter::{importable_response, job_response, rfc3339},
};
use crate::application::SchedulerResult;

type ApiResult<T> = Result<T, SchedulerApiError>;
type AuthenticatedJobMutation<T> = (
    Extension<CurrentUser>,
    Option<Extension<audit_contract::OperationAuditContext>>,
    Path<String>,
    RequestJson<T>,
);
type ImportJobRequestContext = (
    State<SchedulerApiState>,
    Extension<CurrentUser>,
    Option<Extension<audit_contract::OperationAuditContext>>,
    RequestJson<ImportJobRequest>,
);
type RunJobRequestContext = (
    State<SchedulerApiState>,
    Extension<CurrentUser>,
    Option<Extension<audit_contract::OperationAuditContext>>,
    Path<String>,
);

#[require_perms("system:job:list")]
pub async fn list_jobs(State(state): State<SchedulerApiState>, RequestQuery(query): RequestQuery<JobListQuery>) -> ApiResult<Json<CursorPage<JobResponse>>> {
    let page = state
        .scheduler
        .page_jobs(job_filter(&query)?, page_request(query.limit, query.cursor.clone()))
        .await?;
    Ok(Json(map_job_page(page)?))
}

#[require_perms("system:job:query")]
pub async fn get_job(State(state): State<SchedulerApiState>, Path(id): Path<String>) -> ApiResult<Json<JobResponse>> {
    Ok(Json(job_response(state.scheduler.get_job(&id).await?, current_locale())?))
}

#[require_perms("system:job:import")]
pub async fn importable_tasks(State(state): State<SchedulerApiState>) -> ApiResult<Json<Vec<ImportableTaskResponse>>> {
    let locale = current_locale();
    let tasks = state
        .scheduler
        .importable_tasks()
        .await?
        .into_iter()
        .map(|task| importable_response(task, locale))
        .collect();
    Ok(Json(tasks))
}

#[require_perms("system:job:import")]
pub async fn import_job((State(state), Extension(user), audit_context, RequestJson(request)): ImportJobRequestContext) -> ApiResult<Json<JobResponse>> {
    let command = import_command(request, user.username)?;
    let audit = successful_operation_audit(audit_context)?;
    let job = state.audited_scheduler.import_job_with_audit(command, audit.record()).await?;
    audit.mark_persisted();
    Ok(Json(job_response(job, current_locale())?))
}

#[require_perms("system:job:edit")]
pub async fn replace_job(State(state): State<SchedulerApiState>, mutation: AuthenticatedJobMutation<ReplaceJobRequest>) -> ApiResult<Json<JobResponse>> {
    let (Extension(user), audit_context, Path(id), RequestJson(request)) = mutation;
    let command = replace_command(id, request, user.username)?;
    let audit = successful_operation_audit(audit_context)?;
    let job = state.audited_scheduler.replace_job_with_audit(command, audit.record()).await?;
    audit.mark_persisted();
    Ok(Json(job_response(job, current_locale())?))
}

#[require_perms("system:job:changeStatus")]
pub async fn update_job_status(
    State(state): State<SchedulerApiState>,
    mutation: AuthenticatedJobMutation<UpdateJobStatusRequest>,
) -> ApiResult<Json<JobResponse>> {
    let (Extension(user), audit_context, Path(id), RequestJson(request)) = mutation;
    let command = status_command(id, request, user.username)?;
    let audit = successful_operation_audit(audit_context)?;
    let job = state.audited_scheduler.update_job_status_with_audit(command, audit.record()).await?;
    audit.mark_persisted();
    Ok(Json(job_response(job, current_locale())?))
}

#[require_perms("system:job:run")]
pub async fn run_job((State(state), Extension(user), audit_context, Path(id)): RunJobRequestContext) -> ApiResult<(StatusCode, Json<RunJobResponse>)> {
    let audit = successful_operation_audit(audit_context)?;
    let execution_id = state.audited_scheduler.run_job_with_audit(&id, &user.username, audit.record()).await?;
    audit.mark_persisted();
    Ok(accepted_run(execution_id))
}

fn accepted_run(execution_id: String) -> (StatusCode, Json<RunJobResponse>) {
    (StatusCode::ACCEPTED, Json(RunJobResponse { accepted: true, execution_id }))
}

#[require_perms("system:job:remove")]
pub async fn delete_job(
    State(state): State<SchedulerApiState>,
    audit_context: Option<Extension<audit_contract::OperationAuditContext>>,
    Path(id): Path<String>,
) -> ApiResult<Json<()>> {
    let audit = successful_operation_audit(audit_context)?;
    state.audited_scheduler.delete_job_with_audit(&id, audit.record()).await?;
    audit.mark_persisted();
    Ok(Json(()))
}

#[require_perms("system:job:remove")]
pub async fn delete_jobs(
    State(state): State<SchedulerApiState>,
    audit_context: Option<Extension<audit_contract::OperationAuditContext>>,
    RequestJson(request): RequestJson<BatchIdsRequest>,
) -> ApiResult<Json<()>> {
    let audit = successful_operation_audit(audit_context)?;
    state.audited_scheduler.delete_jobs_with_audit(request.ids, audit.record()).await?;
    audit.mark_persisted();
    Ok(Json(()))
}

#[require_any_perms("system:job:import", "system:job:edit")]
pub async fn cron_next_times(
    State(state): State<SchedulerApiState>,
    RequestJson(request): RequestJson<CronNextTimesRequest>,
) -> ApiResult<Json<CronNextTimesResponse>> {
    let times = state
        .scheduler
        .cron_next_times(&request.expression, request.count)
        .await?
        .into_iter()
        .map(rfc3339)
        .collect::<SchedulerResult<Vec<_>>>()?;
    Ok(Json(CronNextTimesResponse { times }))
}

#[require_perms("system:job:export")]
pub async fn export_jobs(State(state): State<SchedulerApiState>, RequestQuery(query): RequestQuery<JobExportQuery>) -> ApiResult<Response> {
    let filter = job_export_filter(query)?;
    let batch = state.export_config.export_batch_config().await?;
    let artifact = build_export_jobs(ExportRequest {
        state: &state,
        filter,
        batch,
        locale: current_locale(),
    })
    .await?;
    Ok(xlsx_file_attachment("jobs.xlsx", artifact))
}

fn map_job_page(page: CursorPage<crate::application::JobView>) -> SchedulerResult<CursorPage<JobResponse>> {
    let locale = current_locale();
    Ok(CursorPage {
        items: page.items.into_iter().map(|job| job_response(job, locale)).collect::<SchedulerResult<_>>()?,
        next_cursor: page.next_cursor,
        previous_cursor: page.previous_cursor,
        has_next: page.has_next,
        has_previous: page.has_previous,
    })
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use serde_json::json;

    use super::accepted_run;

    #[test]
    fn manual_run_response_uses_the_accepted_contract() {
        let (status, response) = accepted_run("execution-1".into());

        assert_eq!(status, StatusCode::ACCEPTED);
        assert_eq!(
            serde_json::to_value(response.0).unwrap(),
            json!({
                "accepted": true,
                "execution_id": "execution-1",
            })
        );
    }
}
