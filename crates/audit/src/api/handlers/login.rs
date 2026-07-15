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
    dto::{BatchIdsRequest, LoginLogExportQuery, LoginLogListQuery, LoginLogResponse},
    export::{self, ExportRequest},
    input,
    openapi::AuditErrorResponses,
    presenter,
};

#[utoipa::path(
    get,
    path = "/system/login-logs",
    tag = "audit-login",
    params(LoginLogListQuery),
    responses(
        (status = 200, description = "Cursor-paginated login logs", body = CursorPage<LoginLogResponse>),
        (status = 400, description = "Invalid login log query", body = ApiErrorResponse),
        AuditErrorResponses
    ),
    security(("bearerAuth" = []))
)]
#[require_perms("system:logininfor:list")]
pub(in crate::api) async fn list_login_logs(
    State(state): State<AuditApiState>,
    RequestQuery(query): RequestQuery<LoginLogListQuery>,
) -> ApiResult<Json<CursorPage<LoginLogResponse>>> {
    let page = state
        .audit
        .page_logins(input::login_filter(&query)?, input::page(query.limit, query.cursor.clone()))
        .await?;
    Ok(Json(map_login_page(page)?))
}

#[utoipa::path(
    delete,
    path = "/system/login-logs/{id}",
    tag = "audit-login",
    params(("id" = String, Path, description = "Login log identifier")),
    responses(
        (status = 200, description = "Login log deleted", body = ()),
        (status = 400, description = "Invalid login log identifier", body = ApiErrorResponse),
        (status = 404, description = "Login log not found", body = ApiErrorResponse),
        AuditErrorResponses
    ),
    security(("bearerAuth" = []))
)]
#[require_perms("system:logininfor:remove")]
pub(in crate::api) async fn delete_login_log(
    State(state): State<AuditApiState>,
    context: Option<Extension<OperationAuditContext>>,
    Path(id): Path<String>,
) -> ApiResult<Json<()>> {
    let context = super::required_operation_audit_context(context)?;
    let record = super::successful_operation_record(&context)?;
    state.audit.delete_logins_with_audit(vec![id], record).await?;
    context.mark_persisted();
    Ok(Json(()))
}

#[utoipa::path(
    delete,
    path = "/system/login-logs/batch",
    tag = "audit-login",
    request_body(content = BatchIdsRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Login logs deleted", body = ()),
        (status = 400, description = "Invalid login log identifiers", body = ApiErrorResponse),
        (status = 404, description = "A login log was not found", body = ApiErrorResponse),
        AuditErrorResponses
    ),
    security(("bearerAuth" = []))
)]
#[require_perms("system:logininfor:remove")]
pub(in crate::api) async fn delete_login_logs(
    State(state): State<AuditApiState>,
    context: Option<Extension<OperationAuditContext>>,
    RequestJson(request): RequestJson<BatchIdsRequest>,
) -> ApiResult<Json<()>> {
    let context = super::required_operation_audit_context(context)?;
    let record = super::successful_operation_record(&context)?;
    state.audit.delete_logins_with_audit(request.ids, record).await?;
    context.mark_persisted();
    Ok(Json(()))
}

#[utoipa::path(
    delete,
    path = "/system/login-logs/clean",
    tag = "audit-login",
    responses((status = 200, description = "Login logs cleared", body = ()), AuditErrorResponses),
    security(("bearerAuth" = []))
)]
#[require_perms("system:logininfor:remove")]
pub(in crate::api) async fn clear_login_logs(State(state): State<AuditApiState>, context: Option<Extension<OperationAuditContext>>) -> ApiResult<Json<()>> {
    let context = super::required_operation_audit_context(context)?;
    let record = super::successful_operation_record(&context)?;
    state.audit.clear_logins_with_audit(record).await?;
    context.mark_persisted();
    Ok(Json(()))
}

#[utoipa::path(
    post,
    path = "/system/login-logs/export",
    tag = "audit-login",
    params(LoginLogExportQuery),
    responses(
        (
            status = 200,
            description = "Login log workbook",
            body = [u8],
            content_type = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            headers(("content-disposition" = String, description = "Attachment file name"))
        ),
        (status = 400, description = "Invalid login log query", body = ApiErrorResponse),
        AuditErrorResponses
    ),
    security(("bearerAuth" = []))
)]
#[require_perms("system:logininfor:export")]
pub(in crate::api) async fn export_login_logs(
    State(state): State<AuditApiState>,
    RequestQuery(query): RequestQuery<LoginLogExportQuery>,
) -> ApiResult<Response> {
    let artifact = export::login_logs(ExportRequest {
        filter: input::login_export_filter(query)?,
        batch: state.export_config.export_batch_config().await?,
        locale: current_locale(),
        state: &state,
    })
    .await?;
    Ok(xlsx_file_attachment("login_logs.xlsx", artifact))
}

#[utoipa::path(
    put,
    path = "/system/login-logs/{username}/unlock",
    tag = "audit-login",
    params(("username" = String, Path, description = "Account username")),
    responses(
        (status = 200, description = "Account login lock cleared", body = ()),
        (status = 404, description = "Account was not found", body = ApiErrorResponse),
        AuditErrorResponses
    ),
    security(("bearerAuth" = []))
)]
#[require_perms("system:logininfor:unlock")]
pub(in crate::api) async fn unlock_login(State(state): State<AuditApiState>, Path(username): Path<String>) -> ApiResult<Json<()>> {
    state.unlocker.unlock(&username).await?;
    Ok(Json(()))
}

fn map_login_page(page: CursorPage<crate::domain::LoginLog>) -> AuditResult<CursorPage<LoginLogResponse>> {
    let locale = current_locale();
    Ok(CursorPage {
        items: page
            .items
            .into_iter()
            .map(|item| presenter::login(item, locale))
            .collect::<AuditResult<Vec<_>>>()?,
        next_cursor: page.next_cursor,
        previous_cursor: page.previous_cursor,
        has_next: page.has_next,
        has_previous: page.has_previous,
    })
}
