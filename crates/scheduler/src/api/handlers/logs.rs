use axum::{
    Json,
    extract::{Path, State},
    response::Response,
};
use kernel::pagination::Page;
use rbac_macros::require_perms;
use types::http::{RequestJson, RequestQuery, current_locale, xlsx_attachment};

use crate::api::{
    SchedulerApiError, SchedulerApiState,
    detail_presenter::execution_detail_response,
    dto::{BatchIdsRequest, ExecutionLogDetailResponse, ExecutionLogResponse, JobLogListQuery},
    export::{ExportRequest, export_job_logs as build_export_job_logs},
    input::{log_filter, page_request},
    presenter::execution_response,
};

type ApiResult<T> = Result<T, SchedulerApiError>;

#[require_perms("system:job:log:list")]
pub async fn list_job_logs(
    State(state): State<SchedulerApiState>,
    RequestQuery(query): RequestQuery<JobLogListQuery>,
) -> ApiResult<Json<Page<ExecutionLogResponse>>> {
    let page = state
        .scheduler
        .page_job_logs(log_filter(&query)?, page_request(query.page, query.page_size))
        .await?;
    Ok(Json(map_execution_page(page)))
}

#[require_perms("system:job:log:query")]
pub async fn get_job_log(State(state): State<SchedulerApiState>, Path(id): Path<String>) -> ApiResult<Json<ExecutionLogResponse>> {
    Ok(Json(execution_response(state.scheduler.get_job_log(&id).await?, current_locale())))
}

#[require_perms("system:job:log:query", "system:job:log:detail")]
pub async fn get_job_log_detail(State(state): State<SchedulerApiState>, Path(id): Path<String>) -> ApiResult<Json<ExecutionLogDetailResponse>> {
    let detail = state.scheduler.get_job_log_detail(&id).await?;
    Ok(Json(execution_detail_response(detail, current_locale())))
}

#[require_perms("system:job:log:remove")]
pub async fn delete_job_log(State(state): State<SchedulerApiState>, Path(id): Path<String>) -> ApiResult<Json<()>> {
    state.scheduler.delete_job_log(&id).await?;
    Ok(Json(()))
}

#[require_perms("system:job:log:remove")]
pub async fn delete_job_logs(State(state): State<SchedulerApiState>, RequestJson(request): RequestJson<BatchIdsRequest>) -> ApiResult<Json<()>> {
    state.scheduler.delete_job_logs(request.ids).await?;
    Ok(Json(()))
}

#[require_perms("system:job:log:remove")]
pub async fn clear_job_logs(State(state): State<SchedulerApiState>) -> ApiResult<Json<()>> {
    state.scheduler.clear_job_logs().await?;
    Ok(Json(()))
}

#[require_perms("system:job:log:export")]
pub async fn export_job_logs(State(state): State<SchedulerApiState>, RequestQuery(query): RequestQuery<JobLogListQuery>) -> ApiResult<Response> {
    let filter = log_filter(&query)?;
    let batch = state.export_config.export_batch_config().await?;
    let bytes = build_export_job_logs(ExportRequest {
        state: &state,
        filter,
        batch,
        locale: current_locale(),
    })
    .await?;
    Ok(xlsx_attachment("job_logs.xlsx", bytes))
}

fn map_execution_page(page: Page<crate::application::ExecutionLogSummary>) -> Page<ExecutionLogResponse> {
    let locale = current_locale();
    Page {
        items: page.items.into_iter().map(|execution| execution_response(execution, locale)).collect(),
        total: page.total,
        page: page.page,
        page_size: page.page_size,
    }
}
