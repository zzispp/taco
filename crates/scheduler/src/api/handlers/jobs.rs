use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::Response,
};
use kernel::pagination::Page;
use rbac::api::CurrentUser;
use rbac_macros::{require_any_perms, require_perms};
use types::http::{RequestJson, RequestQuery, current_locale, xlsx_attachment};

use crate::api::{
    SchedulerApiError, SchedulerApiState,
    dto::{
        BatchIdsRequest, CronNextTimesRequest, CronNextTimesResponse, ImportJobRequest, ImportableTaskResponse, JobListQuery, JobResponse, ReplaceJobRequest,
        RunJobResponse, UpdateJobStatusRequest,
    },
    export::{ExportRequest, export_jobs as build_export_jobs},
    input::{import_command, job_filter, page_request, replace_command, status_command},
    presenter::{importable_response, job_response},
};

type ApiResult<T> = Result<T, SchedulerApiError>;
type AuthenticatedJobMutation<T> = (Extension<CurrentUser>, Path<String>, RequestJson<T>);

#[require_perms("system:job:list")]
pub async fn list_jobs(State(state): State<SchedulerApiState>, RequestQuery(query): RequestQuery<JobListQuery>) -> ApiResult<Json<Page<JobResponse>>> {
    let page = state
        .scheduler
        .page_jobs(job_filter(&query)?, page_request(query.page, query.page_size))
        .await?;
    Ok(Json(map_job_page(page)))
}

#[require_perms("system:job:query")]
pub async fn get_job(State(state): State<SchedulerApiState>, Path(id): Path<String>) -> ApiResult<Json<JobResponse>> {
    Ok(Json(job_response(state.scheduler.get_job(&id).await?, current_locale())))
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
pub async fn import_job(
    State(state): State<SchedulerApiState>,
    Extension(user): Extension<CurrentUser>,
    RequestJson(request): RequestJson<ImportJobRequest>,
) -> ApiResult<Json<JobResponse>> {
    let job = state.scheduler.import_job(import_command(request, user.username)?).await?;
    Ok(Json(job_response(job, current_locale())))
}

#[require_perms("system:job:edit")]
pub async fn replace_job(State(state): State<SchedulerApiState>, mutation: AuthenticatedJobMutation<ReplaceJobRequest>) -> ApiResult<Json<JobResponse>> {
    let (Extension(user), Path(id), RequestJson(request)) = mutation;
    let job = state.scheduler.replace_job(replace_command(id, request, user.username)?).await?;
    Ok(Json(job_response(job, current_locale())))
}

#[require_perms("system:job:changeStatus")]
pub async fn update_job_status(
    State(state): State<SchedulerApiState>,
    mutation: AuthenticatedJobMutation<UpdateJobStatusRequest>,
) -> ApiResult<Json<JobResponse>> {
    let (Extension(user), Path(id), RequestJson(request)) = mutation;
    let job = state.scheduler.update_job_status(status_command(id, request, user.username)?).await?;
    Ok(Json(job_response(job, current_locale())))
}

#[require_perms("system:job:run")]
pub async fn run_job(
    State(state): State<SchedulerApiState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> ApiResult<(StatusCode, Json<RunJobResponse>)> {
    let execution_id = state.scheduler.run_job(&id, &user.username).await?;
    Ok(accepted_run(execution_id))
}

fn accepted_run(execution_id: String) -> (StatusCode, Json<RunJobResponse>) {
    (StatusCode::ACCEPTED, Json(RunJobResponse { accepted: true, execution_id }))
}

#[require_perms("system:job:remove")]
pub async fn delete_job(State(state): State<SchedulerApiState>, Path(id): Path<String>) -> ApiResult<Json<()>> {
    state.scheduler.delete_job(&id).await?;
    Ok(Json(()))
}

#[require_perms("system:job:remove")]
pub async fn delete_jobs(State(state): State<SchedulerApiState>, RequestJson(request): RequestJson<BatchIdsRequest>) -> ApiResult<Json<()>> {
    state.scheduler.delete_jobs(request.ids).await?;
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
        .map(|time| time.to_rfc3339())
        .collect();
    Ok(Json(CronNextTimesResponse { times }))
}

#[require_perms("system:job:export")]
pub async fn export_jobs(State(state): State<SchedulerApiState>, RequestQuery(query): RequestQuery<JobListQuery>) -> ApiResult<Response> {
    let filter = job_filter(&query)?;
    let batch = state.export_config.export_batch_config().await?;
    let bytes = build_export_jobs(ExportRequest {
        state: &state,
        filter,
        batch,
        locale: current_locale(),
    })
    .await?;
    Ok(xlsx_attachment("jobs.xlsx", bytes))
}

fn map_job_page(page: Page<crate::application::JobView>) -> Page<JobResponse> {
    let locale = current_locale();
    Page {
        items: page.items.into_iter().map(|job| job_response(job, locale)).collect(),
        total: page.total,
        page: page.page,
        page_size: page.page_size,
    }
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
