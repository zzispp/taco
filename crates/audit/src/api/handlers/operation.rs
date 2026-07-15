use axum::{
    Json,
    extract::{Extension, Path, State},
    response::Response,
};
use kernel::pagination::CursorPage;
use rbac_macros::require_perms;
use types::http::{ApiErrorResponse, RequestJson, RequestQuery, current_locale, xlsx_file_attachment};

use audit_contract::OperationAuditContext;

use crate::application::AuditResult;

use super::ApiResult;
use crate::api::{
    AuditApiState,
    dto::{BatchIdsRequest, OperationLogDetailResponse, OperationLogExportQuery, OperationLogListQuery, OperationLogSummaryResponse},
    export::{self, ExportRequest},
    input,
    openapi::AuditErrorResponses,
    presenter,
};

#[utoipa::path(
    get,
    path = "/system/operation-logs",
    tag = "audit-operation",
    params(OperationLogListQuery),
    responses(
        (status = 200, description = "Cursor-paginated operation logs", body = CursorPage<OperationLogSummaryResponse>),
        (status = 400, description = "Invalid operation log query", body = ApiErrorResponse),
        AuditErrorResponses
    ),
    security(("bearerAuth" = []))
)]
#[require_perms("system:operlog:list")]
pub(in crate::api) async fn list_operation_logs(
    State(state): State<AuditApiState>,
    RequestQuery(query): RequestQuery<OperationLogListQuery>,
) -> ApiResult<Json<CursorPage<OperationLogSummaryResponse>>> {
    let page = state
        .audit
        .page_operations(
            input::operation_filter(&query, current_locale())?,
            input::page(query.limit, query.cursor.clone()),
        )
        .await?;
    Ok(Json(map_operation_page(page)?))
}

#[utoipa::path(
    get,
    path = "/system/operation-logs/{id}",
    tag = "audit-operation",
    params(("id" = String, Path, description = "Operation log identifier")),
    responses(
        (status = 200, description = "Operation log detail", body = OperationLogDetailResponse),
        (status = 404, description = "Operation log not found", body = ApiErrorResponse),
        AuditErrorResponses
    ),
    security(("bearerAuth" = []))
)]
#[require_perms("system:operlog:query")]
pub(in crate::api) async fn get_operation_log(State(state): State<AuditApiState>, Path(id): Path<String>) -> ApiResult<Json<OperationLogDetailResponse>> {
    Ok(Json(presenter::operation_detail(state.audit.operation_detail(&id).await?, current_locale())?))
}

#[utoipa::path(
    delete,
    path = "/system/operation-logs/{id}",
    tag = "audit-operation",
    params(("id" = String, Path, description = "Operation log identifier")),
    responses(
        (status = 200, description = "Operation log deleted", body = ()),
        (status = 400, description = "Invalid operation log identifier", body = ApiErrorResponse),
        (status = 404, description = "Operation log not found", body = ApiErrorResponse),
        AuditErrorResponses
    ),
    security(("bearerAuth" = []))
)]
#[require_perms("system:operlog:remove")]
pub(in crate::api) async fn delete_operation_log(
    State(state): State<AuditApiState>,
    context: Option<Extension<OperationAuditContext>>,
    Path(id): Path<String>,
) -> ApiResult<Json<()>> {
    let context = super::required_operation_audit_context(context)?;
    let record = super::successful_operation_record(&context)?;
    state.audit.delete_operations_with_audit(vec![id], record).await?;
    context.mark_persisted();
    Ok(Json(()))
}

#[utoipa::path(
    delete,
    path = "/system/operation-logs/batch",
    tag = "audit-operation",
    request_body(content = BatchIdsRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Operation logs deleted", body = ()),
        (status = 400, description = "Invalid operation log identifiers", body = ApiErrorResponse),
        (status = 404, description = "An operation log was not found", body = ApiErrorResponse),
        AuditErrorResponses
    ),
    security(("bearerAuth" = []))
)]
#[require_perms("system:operlog:remove")]
pub(in crate::api) async fn delete_operation_logs(
    State(state): State<AuditApiState>,
    context: Option<Extension<OperationAuditContext>>,
    RequestJson(request): RequestJson<BatchIdsRequest>,
) -> ApiResult<Json<()>> {
    let context = super::required_operation_audit_context(context)?;
    let record = super::successful_operation_record(&context)?;
    state.audit.delete_operations_with_audit(request.ids, record).await?;
    context.mark_persisted();
    Ok(Json(()))
}

#[utoipa::path(
    delete,
    path = "/system/operation-logs/clean",
    tag = "audit-operation",
    responses((status = 200, description = "Operation logs cleared", body = ()), AuditErrorResponses),
    security(("bearerAuth" = []))
)]
#[require_perms("system:operlog:remove")]
pub(in crate::api) async fn clear_operation_logs(State(state): State<AuditApiState>, context: Option<Extension<OperationAuditContext>>) -> ApiResult<Json<()>> {
    let context = super::required_operation_audit_context(context)?;
    let record = super::successful_operation_record(&context)?;
    state.audit.clear_operations_with_audit(record).await?;
    context.mark_persisted();
    Ok(Json(()))
}

#[utoipa::path(
    post,
    path = "/system/operation-logs/export",
    tag = "audit-operation",
    params(OperationLogExportQuery),
    responses(
        (
            status = 200,
            description = "Operation log workbook",
            body = [u8],
            content_type = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            headers(("content-disposition" = String, description = "Attachment file name"))
        ),
        (status = 400, description = "Invalid operation log query", body = ApiErrorResponse),
        AuditErrorResponses
    ),
    security(("bearerAuth" = []))
)]
#[require_perms("system:operlog:export")]
pub(in crate::api) async fn export_operation_logs(
    State(state): State<AuditApiState>,
    RequestQuery(query): RequestQuery<OperationLogExportQuery>,
) -> ApiResult<Response> {
    let artifact = export::operation_logs(ExportRequest {
        filter: input::operation_export_filter(query, current_locale())?,
        batch: state.export_config.export_batch_config().await?,
        locale: current_locale(),
        state: &state,
    })
    .await?;
    Ok(xlsx_file_attachment("operation_logs.xlsx", artifact))
}

fn map_operation_page(page: CursorPage<crate::domain::OperationLogSummary>) -> AuditResult<CursorPage<OperationLogSummaryResponse>> {
    let locale = current_locale();
    Ok(CursorPage {
        items: page
            .items
            .into_iter()
            .map(|item| presenter::operation_summary(item, locale))
            .collect::<AuditResult<Vec<_>>>()?,
        next_cursor: page.next_cursor,
        previous_cursor: page.previous_cursor,
        has_next: page.has_next,
        has_previous: page.has_previous,
    })
}
