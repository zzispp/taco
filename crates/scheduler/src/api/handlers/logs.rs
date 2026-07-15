use axum::{
    Json,
    extract::{Path, State},
    response::Response,
};
use kernel::pagination::CursorPage;
use rbac_macros::require_perms;
use types::http::{RequestJson, RequestQuery, current_locale, xlsx_file_attachment};

use super::support::successful_operation_audit;
use crate::api::{
    SchedulerApiError, SchedulerApiState,
    detail_presenter::execution_detail_response,
    dto::{BatchIdsRequest, ExecutionLogDetailResponse, ExecutionLogResponse, JobLogExportQuery, JobLogListQuery},
    export::{ExportRequest, export_job_logs as build_export_job_logs},
    input::{log_export_filter, log_filter, page_request},
    presenter::execution_response,
};
use crate::application::SchedulerResult;

type ApiResult<T> = Result<T, SchedulerApiError>;

#[require_perms("system:job:log:list")]
pub async fn list_job_logs(
    State(state): State<SchedulerApiState>,
    RequestQuery(query): RequestQuery<JobLogListQuery>,
) -> ApiResult<Json<CursorPage<ExecutionLogResponse>>> {
    let page = state
        .scheduler
        .page_job_logs(log_filter(&query)?, page_request(query.limit, query.cursor.clone()))
        .await?;
    Ok(Json(map_execution_page(page)?))
}

#[require_perms("system:job:log:query")]
pub async fn get_job_log(State(state): State<SchedulerApiState>, Path(id): Path<String>) -> ApiResult<Json<ExecutionLogResponse>> {
    Ok(Json(execution_response(state.scheduler.get_job_log(&id).await?, current_locale())?))
}

#[require_perms("system:job:log:query", "system:job:log:detail")]
pub async fn get_job_log_detail(State(state): State<SchedulerApiState>, Path(id): Path<String>) -> ApiResult<Json<ExecutionLogDetailResponse>> {
    let detail = state.scheduler.get_job_log_detail(&id).await?;
    Ok(Json(execution_detail_response(detail, current_locale())?))
}

#[require_perms("system:job:log:remove")]
pub async fn delete_job_log(
    State(state): State<SchedulerApiState>,
    audit_context: Option<axum::extract::Extension<audit_contract::OperationAuditContext>>,
    Path(id): Path<String>,
) -> ApiResult<Json<()>> {
    let audit = successful_operation_audit(audit_context)?;
    state.audited_scheduler.delete_job_log_with_audit(&id, audit.record()).await?;
    audit.mark_persisted();
    Ok(Json(()))
}

#[require_perms("system:job:log:remove")]
pub async fn delete_job_logs(
    State(state): State<SchedulerApiState>,
    audit_context: Option<axum::extract::Extension<audit_contract::OperationAuditContext>>,
    RequestJson(request): RequestJson<BatchIdsRequest>,
) -> ApiResult<Json<()>> {
    let audit = successful_operation_audit(audit_context)?;
    state.audited_scheduler.delete_job_logs_with_audit(request.ids, audit.record()).await?;
    audit.mark_persisted();
    Ok(Json(()))
}

#[require_perms("system:job:log:remove")]
pub async fn clear_job_logs(
    State(state): State<SchedulerApiState>,
    audit_context: Option<axum::extract::Extension<audit_contract::OperationAuditContext>>,
) -> ApiResult<Json<()>> {
    let audit = successful_operation_audit(audit_context)?;
    state.audited_scheduler.clear_job_logs_with_audit(audit.record()).await?;
    audit.mark_persisted();
    Ok(Json(()))
}

#[require_perms("system:job:log:export")]
pub async fn export_job_logs(State(state): State<SchedulerApiState>, RequestQuery(query): RequestQuery<JobLogExportQuery>) -> ApiResult<Response> {
    let filter = log_export_filter(query)?;
    let batch = state.export_config.export_batch_config().await?;
    let artifact = build_export_job_logs(ExportRequest {
        state: &state,
        filter,
        batch,
        locale: current_locale(),
    })
    .await?;
    Ok(xlsx_file_attachment("job_logs.xlsx", artifact))
}

fn map_execution_page(page: CursorPage<crate::application::ExecutionLogSummary>) -> SchedulerResult<CursorPage<ExecutionLogResponse>> {
    let locale = current_locale();
    Ok(CursorPage {
        items: page
            .items
            .into_iter()
            .map(|execution| execution_response(execution, locale))
            .collect::<SchedulerResult<_>>()?,
        next_cursor: page.next_cursor,
        previous_cursor: page.previous_cursor,
        has_next: page.has_next,
        has_previous: page.has_previous,
    })
}
