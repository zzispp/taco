use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::Response,
};
use kernel::pagination::CursorPage;
use rbac::api::CurrentUser;
use rbac_macros::require_perms;
use types::http::{ApiErrorResponse, RequestJson, RequestQuery, current_locale, xlsx_file_attachment};

use crate::application::{ManualSystemLogCleanupRequest, ObservabilityResult, SystemLogExportRequest};

use super::{
    SystemLogApiError, SystemLogApiState,
    dto::{
        BatchIdsRequest, SystemLogCleanupAcceptedResponse, SystemLogCleanupCountResponse, SystemLogCleanupExecutionResponse, SystemLogCleanupQuery,
        SystemLogDetailResponse, SystemLogExportQuery, SystemLogListQuery, SystemLogSummaryResponse,
    },
    export::system_log_export_layout,
    input,
    openapi::SystemLogErrorResponses,
    presenter,
    support::successful_operation_audit,
};

type ApiResult<T> = Result<T, SystemLogApiError>;

#[utoipa::path(
    get,
    path = "/system/system-logs",
    tag = "observability-system-log",
    params(SystemLogListQuery),
    responses(
        (status = 200, description = "Cursor-paginated system logs", body = CursorPage<SystemLogSummaryResponse>),
        (status = 400, description = "Invalid system log query", body = ApiErrorResponse),
        SystemLogErrorResponses
    ),
    security(("bearerAuth" = []))
)]
#[require_perms("system:systemlog:list")]
pub(in crate::api) async fn list_system_logs(
    State(state): State<SystemLogApiState>,
    RequestQuery(query): RequestQuery<SystemLogListQuery>,
) -> ApiResult<Json<CursorPage<SystemLogSummaryResponse>>> {
    let page = state.logs.page(input::list_filter(&query)?, input::page(query.limit, query.cursor)).await?;
    Ok(Json(map_page(page)?))
}

#[utoipa::path(
    get,
    path = "/system/system-logs/{id}",
    tag = "observability-system-log",
    params(("id" = String, Path, description = "System log identifier")),
    responses(
        (status = 200, description = "System log detail", body = SystemLogDetailResponse),
        (status = 404, description = "System log not found", body = ApiErrorResponse),
        SystemLogErrorResponses
    ),
    security(("bearerAuth" = []))
)]
#[require_perms("system:systemlog:query")]
pub(in crate::api) async fn get_system_log(State(state): State<SystemLogApiState>, Path(id): Path<String>) -> ApiResult<Json<SystemLogDetailResponse>> {
    Ok(Json(presenter::detail(state.logs.detail(&id).await?)?))
}

#[utoipa::path(
    delete,
    path = "/system/system-logs/{id}",
    tag = "observability-system-log",
    params(("id" = String, Path, description = "System log identifier")),
    responses(
        (status = 200, description = "System log deleted", body = ()),
        (status = 400, description = "Invalid system log identifier", body = ApiErrorResponse),
        (status = 404, description = "System log not found", body = ApiErrorResponse),
        SystemLogErrorResponses
    ),
    security(("bearerAuth" = []))
)]
#[require_perms("system:systemlog:remove")]
pub(in crate::api) async fn delete_system_log(
    State(state): State<SystemLogApiState>,
    audit_context: Option<Extension<audit_contract::OperationAuditContext>>,
    Path(id): Path<String>,
) -> ApiResult<Json<()>> {
    let audit = successful_operation_audit(audit_context)?;
    state.logs.delete_with_audit(vec![id], audit.record()).await?;
    audit.mark_persisted();
    Ok(Json(()))
}

#[utoipa::path(
    delete,
    path = "/system/system-logs/batch",
    tag = "observability-system-log",
    request_body(content = BatchIdsRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "System logs deleted", body = ()),
        (status = 400, description = "Invalid system log identifiers", body = ApiErrorResponse),
        (status = 404, description = "A system log was not found", body = ApiErrorResponse),
        SystemLogErrorResponses
    ),
    security(("bearerAuth" = []))
)]
#[require_perms("system:systemlog:remove")]
pub(in crate::api) async fn delete_system_logs(
    State(state): State<SystemLogApiState>,
    audit_context: Option<Extension<audit_contract::OperationAuditContext>>,
    RequestJson(request): RequestJson<BatchIdsRequest>,
) -> ApiResult<Json<()>> {
    let audit = successful_operation_audit(audit_context)?;
    state.logs.delete_with_audit(request.ids, audit.record()).await?;
    audit.mark_persisted();
    Ok(Json(()))
}

#[utoipa::path(
    get,
    path = "/system/system-logs/clean/count",
    tag = "observability-system-log",
    params(SystemLogCleanupQuery),
    responses(
        (status = 200, description = "Matching system log count", body = SystemLogCleanupCountResponse),
        (status = 400, description = "A cleanup time range is required", body = ApiErrorResponse),
        SystemLogErrorResponses
    ),
    security(("bearerAuth" = []))
)]
#[require_perms("system:systemlog:remove")]
pub(in crate::api) async fn count_system_logs_for_cleanup(
    State(state): State<SystemLogApiState>,
    RequestQuery(query): RequestQuery<SystemLogCleanupQuery>,
) -> ApiResult<Json<SystemLogCleanupCountResponse>> {
    let count = state.logs.count(input::cleanup_filter(&query)?).await?;
    Ok(Json(SystemLogCleanupCountResponse { count }))
}

#[utoipa::path(
    delete,
    path = "/system/system-logs/clean",
    tag = "observability-system-log",
    params(SystemLogCleanupQuery),
    responses(
        (status = 202, description = "System-log cleanup accepted", body = SystemLogCleanupAcceptedResponse),
        (status = 400, description = "A cleanup time range is required", body = ApiErrorResponse),
        SystemLogErrorResponses
    ),
    security(("bearerAuth" = []))
)]
#[require_perms("system:systemlog:remove")]
pub(in crate::api) async fn clean_system_logs(
    State(state): State<SystemLogApiState>,
    Extension(user): Extension<CurrentUser>,
    audit_context: Option<Extension<audit_contract::OperationAuditContext>>,
    RequestQuery(query): RequestQuery<SystemLogCleanupQuery>,
) -> ApiResult<(StatusCode, Json<SystemLogCleanupAcceptedResponse>)> {
    let audit = successful_operation_audit(audit_context)?;
    let execution_id = state
        .cleanup_executions
        .enqueue_manual_cleanup(ManualSystemLogCleanupRequest {
            filter: input::cleanup_filter(&query)?,
            requested_by: user.username,
            audit: audit.record(),
        })
        .await?;
    audit.mark_persisted();
    Ok((StatusCode::ACCEPTED, Json(SystemLogCleanupAcceptedResponse { accepted: true, execution_id })))
}

#[utoipa::path(
    get,
    path = "/system/system-logs/clean/executions/{execution_id}",
    tag = "observability-system-log",
    params(("execution_id" = String, Path, description = "Cleanup execution identifier")),
    responses(
        (status = 200, description = "System-log cleanup execution", body = SystemLogCleanupExecutionResponse),
        (status = 404, description = "Cleanup execution was not found", body = ApiErrorResponse),
        SystemLogErrorResponses
    ),
    security(("bearerAuth" = []))
)]
#[require_perms("system:systemlog:remove")]
pub(in crate::api) async fn get_system_log_cleanup_execution(
    State(state): State<SystemLogApiState>,
    Path(execution_id): Path<String>,
) -> ApiResult<Json<SystemLogCleanupExecutionResponse>> {
    Ok(Json(presenter::cleanup_execution(
        state.cleanup_executions.cleanup_execution(&execution_id).await?,
    )))
}

#[utoipa::path(
    post,
    path = "/system/system-logs/export",
    tag = "observability-system-log",
    params(SystemLogExportQuery),
    responses(
        (
            status = 200,
            description = "System log workbook",
            body = [u8],
            content_type = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            headers(("content-disposition" = String, description = "Attachment file name"))
        ),
        (status = 400, description = "An export time range is required", body = ApiErrorResponse),
        SystemLogErrorResponses
    ),
    security(("bearerAuth" = []))
)]
#[require_perms("system:systemlog:export")]
pub(in crate::api) async fn export_system_logs(
    State(state): State<SystemLogApiState>,
    RequestQuery(query): RequestQuery<SystemLogExportQuery>,
) -> ApiResult<Response> {
    let locale = current_locale();
    let artifact = state
        .exporter
        .export_xlsx(SystemLogExportRequest {
            filter: input::export_filter(&query)?,
            batch: state.export_config.export_batch_config().await?,
            layout: system_log_export_layout(locale),
        })
        .await?;
    Ok(xlsx_file_attachment("system_logs.xlsx", artifact))
}

fn map_page(page: CursorPage<crate::domain::SystemLogSummary>) -> ObservabilityResult<CursorPage<SystemLogSummaryResponse>> {
    Ok(CursorPage {
        items: page.items.into_iter().map(presenter::summary).collect::<ObservabilityResult<Vec<_>>>()?,
        next_cursor: page.next_cursor,
        previous_cursor: page.previous_cursor,
        has_next: page.has_next,
        has_previous: page.has_previous,
    })
}
