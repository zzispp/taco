use std::sync::Arc;

use audit_contract::OperationAuditContext;
use axum::routing::{delete, get, put};
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
};
use kernel::pagination::{CursorPage, CursorPageRequest};
use rbac::api::CurrentUser;
use rbac_macros::require_perms;
use serde::Deserialize;
use types::http::{RequestJson, RequestQuery};
use types::system::BatchIdsInput;
use utoipa::IntoParams;

use crate::api::{SystemApiError, operation_audit::successful_operation_audit};

use super::{
    Notice, NoticeAuditedUseCase, NoticeInput, NoticeListFilter, NoticeReader, NoticeReaderFilter, NoticeSummary, NoticeTopResponse, NoticeUseCase,
    ReplaceNoticeCommand,
};

const NOTICE_QUERY_PERMISSION: &str = "system:notice:query";

#[derive(Clone)]
pub struct NoticeApiState {
    pub notices: Arc<dyn NoticeUseCase>,
    pub notices_audited: Arc<dyn NoticeAuditedUseCase>,
}

impl NoticeApiState {
    pub fn new(notices: Arc<dyn NoticeUseCase>, notices_audited: Arc<dyn NoticeAuditedUseCase>) -> Self {
        Self { notices, notices_audited }
    }
}

pub fn create_router(state: NoticeApiState) -> Router {
    use super::endpoints::{NOTICE_READ, NOTICE_READERS, NOTICE_REPLACE, NOTICES_CREATE, NOTICES_DELETE_BATCH, NOTICES_READ_ALL, NOTICES_TOP};

    Router::new()
        .route(NOTICES_CREATE.api_route_path(), get(list_notices).post(create_notice))
        .route(NOTICES_TOP.api_route_path(), get(top_notices))
        .route(NOTICES_READ_ALL.api_route_path(), put(mark_all_notices_read))
        .route(NOTICES_DELETE_BATCH.api_route_path(), delete(delete_notices))
        .route(NOTICE_READ.api_route_path(), put(mark_notice_read))
        .route(NOTICE_READERS.api_route_path(), get(list_notice_readers))
        .route(NOTICE_REPLACE.api_route_path(), get(get_notice).put(replace_notice).delete(delete_notice))
        .with_state(state)
}

type ApiResult<T> = Result<Json<T>, SystemApiError>;
type CreateNoticeRequest = (
    State<NoticeApiState>,
    Extension<CurrentUser>,
    Option<Extension<OperationAuditContext>>,
    RequestJson<NoticeInput>,
);
type ReplaceNoticeRequest = (
    State<NoticeApiState>,
    Extension<CurrentUser>,
    Option<Extension<OperationAuditContext>>,
    Path<String>,
    RequestJson<NoticeInput>,
);
type MarkNoticeReadRequest = (
    State<NoticeApiState>,
    Extension<CurrentUser>,
    Option<Extension<OperationAuditContext>>,
    Path<String>,
);

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(deny_unknown_fields)]
pub struct NoticeListQuery {
    #[serde(default = "default_cursor_limit")]
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub limit: u64,
    #[serde(default)]
    pub cursor: Option<String>,
    pub notice_title: Option<String>,
    pub create_by: Option<String>,
    pub notice_type: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(deny_unknown_fields)]
pub struct NoticeReaderQuery {
    #[serde(default = "default_cursor_limit")]
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub limit: u64,
    #[serde(default)]
    pub cursor: Option<String>,
    pub search_value: Option<String>,
    pub user_name: Option<String>,
}

#[require_perms("system:notice:list")]
pub async fn list_notices(State(state): State<NoticeApiState>, RequestQuery(query): RequestQuery<NoticeListQuery>) -> ApiResult<CursorPage<NoticeSummary>> {
    Ok(Json(state.notices.page_notices(query.into()).await?))
}

pub async fn get_notice(State(state): State<NoticeApiState>, Extension(current_user): Extension<CurrentUser>, Path(id): Path<String>) -> ApiResult<Notice> {
    let can_view_closed = can_view_closed_notice(&current_user);
    Ok(Json(state.notices.get_notice(&id, can_view_closed).await?))
}

fn can_view_closed_notice(current_user: &CurrentUser) -> bool {
    current_user.admin
        || current_user
            .permissions
            .iter()
            .any(|permission| matches!(permission.as_str(), constants::system::ALL_PERMISSION | NOTICE_QUERY_PERMISSION))
}

#[require_perms("system:notice:add")]
pub async fn create_notice(request: CreateNoticeRequest) -> ApiResult<Notice> {
    let (State(state), Extension(current_user), audit_context, RequestJson(payload)) = request;
    let audit = successful_operation_audit(audit_context)?;
    let notice = state
        .notices_audited
        .create_notice_with_audit(payload, current_user.username, audit.record())
        .await?;
    audit.mark_persisted();
    Ok(Json(notice))
}

#[require_perms("system:notice:edit")]
pub async fn replace_notice(request: ReplaceNoticeRequest) -> ApiResult<Notice> {
    let (State(state), Extension(current_user), audit_context, Path(id), RequestJson(payload)) = request;
    let audit = successful_operation_audit(audit_context)?;
    let notice = state
        .notices_audited
        .replace_notice_with_audit(
            ReplaceNoticeCommand {
                id,
                input: payload,
                operator: current_user.username,
            },
            audit.record(),
        )
        .await?;
    audit.mark_persisted();
    Ok(Json(notice))
}

#[require_perms("system:notice:remove")]
pub async fn delete_notice(
    State(state): State<NoticeApiState>,
    audit_context: Option<Extension<OperationAuditContext>>,
    Path(id): Path<String>,
) -> ApiResult<()> {
    let audit = successful_operation_audit(audit_context)?;
    state.notices_audited.delete_notice_with_audit(&id, audit.record()).await?;
    audit.mark_persisted();
    Ok(Json(()))
}

#[require_perms("system:notice:remove")]
pub async fn delete_notices(
    State(state): State<NoticeApiState>,
    audit_context: Option<Extension<OperationAuditContext>>,
    RequestJson(payload): RequestJson<BatchIdsInput>,
) -> ApiResult<()> {
    let audit = successful_operation_audit(audit_context)?;
    state.notices_audited.delete_notices_with_audit(payload.ids, audit.record()).await?;
    audit.mark_persisted();
    Ok(Json(()))
}

pub async fn top_notices(State(state): State<NoticeApiState>, Extension(current_user): Extension<CurrentUser>) -> ApiResult<NoticeTopResponse> {
    Ok(Json(state.notices.top_notices(&current_user.id).await?))
}

pub async fn mark_notice_read(request: MarkNoticeReadRequest) -> ApiResult<()> {
    let (State(state), Extension(current_user), audit_context, Path(id)) = request;
    let audit = successful_operation_audit(audit_context)?;
    state.notices_audited.mark_read_with_audit(&id, &current_user.id, audit.record()).await?;
    audit.mark_persisted();
    Ok(Json(()))
}

pub async fn mark_all_notices_read(
    State(state): State<NoticeApiState>,
    Extension(current_user): Extension<CurrentUser>,
    audit_context: Option<Extension<OperationAuditContext>>,
) -> ApiResult<()> {
    let audit = successful_operation_audit(audit_context)?;
    state.notices_audited.mark_all_read_with_audit(&current_user.id, audit.record()).await?;
    audit.mark_persisted();
    Ok(Json(()))
}

#[require_perms("system:notice:list")]
pub async fn list_notice_readers(
    State(state): State<NoticeApiState>,
    Path(id): Path<String>,
    RequestQuery(query): RequestQuery<NoticeReaderQuery>,
) -> ApiResult<CursorPage<NoticeReader>> {
    Ok(Json(state.notices.page_readers(&id, query.into()).await?))
}

impl From<NoticeListQuery> for NoticeListFilter {
    fn from(value: NoticeListQuery) -> Self {
        Self {
            page: CursorPageRequest {
                limit: value.limit,
                cursor: value.cursor,
            },
            notice_title: value.notice_title,
            create_by: value.create_by,
            notice_type: value.notice_type,
        }
    }
}

impl From<NoticeReaderQuery> for NoticeReaderFilter {
    fn from(value: NoticeReaderQuery) -> Self {
        Self {
            page: CursorPageRequest {
                limit: value.limit,
                cursor: value.cursor,
            },
            search_value: value.search_value.or(value.user_name),
        }
    }
}

const fn default_cursor_limit() -> u64 {
    kernel::pagination::DEFAULT_CURSOR_LIMIT
}

#[cfg(test)]
mod tests;
