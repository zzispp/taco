use std::sync::Arc;

use axum::routing::{delete, get, put};
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
};
use kernel::pagination::{Page, PageRequest};
use rbac::api::CurrentUser;
use rbac_macros::require_perms;
use serde::Deserialize;
use types::http::{RequestJson, RequestQuery};
use types::system::BatchIdsInput;

use crate::api::SystemApiError;

use super::{Notice, NoticeInput, NoticeListFilter, NoticeReader, NoticeReaderFilter, NoticeSummary, NoticeTopResponse, NoticeUseCase};

const NOTICE_QUERY_PERMISSION: &str = "system:notice:query";

#[derive(Clone)]
pub struct NoticeApiState {
    pub notices: Arc<dyn NoticeUseCase>,
}

impl NoticeApiState {
    pub fn new(notices: Arc<dyn NoticeUseCase>) -> Self {
        Self { notices }
    }
}

pub fn create_router(state: NoticeApiState) -> Router {
    Router::new()
        .route("/system/notices", get(list_notices).post(create_notice))
        .route("/system/notices/top", get(top_notices))
        .route("/system/notices/read-all", put(mark_all_notices_read))
        .route("/system/notices/batch", delete(delete_notices))
        .route("/system/notices/{id}/read", put(mark_notice_read))
        .route("/system/notices/{id}/readers", get(list_notice_readers))
        .route("/system/notices/{id}", get(get_notice).put(replace_notice).delete(delete_notice))
        .with_state(state)
}

type ApiResult<T> = Result<Json<T>, SystemApiError>;

#[derive(Debug, Deserialize)]
pub struct NoticeListQuery {
    pub page: u64,
    pub page_size: u64,
    pub notice_title: Option<String>,
    pub create_by: Option<String>,
    pub notice_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NoticeReaderQuery {
    pub page: u64,
    pub page_size: u64,
    pub search_value: Option<String>,
    pub user_name: Option<String>,
}

#[require_perms("system:notice:list")]
pub async fn list_notices(State(state): State<NoticeApiState>, RequestQuery(query): RequestQuery<NoticeListQuery>) -> ApiResult<Page<NoticeSummary>> {
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
pub async fn create_notice(
    State(state): State<NoticeApiState>,
    Extension(current_user): Extension<CurrentUser>,
    RequestJson(payload): RequestJson<NoticeInput>,
) -> ApiResult<Notice> {
    Ok(Json(state.notices.create_notice(payload, current_user.username).await?))
}

#[require_perms("system:notice:edit")]
pub async fn replace_notice(
    State(state): State<NoticeApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<NoticeInput>,
) -> ApiResult<Notice> {
    Ok(Json(state.notices.replace_notice(&id, payload, current_user.username).await?))
}

#[require_perms("system:notice:remove")]
pub async fn delete_notice(State(state): State<NoticeApiState>, Path(id): Path<String>) -> ApiResult<()> {
    state.notices.delete_notice(&id).await?;
    Ok(Json(()))
}

#[require_perms("system:notice:remove")]
pub async fn delete_notices(State(state): State<NoticeApiState>, RequestJson(payload): RequestJson<BatchIdsInput>) -> ApiResult<()> {
    state.notices.delete_notices(payload.ids).await?;
    Ok(Json(()))
}

pub async fn top_notices(State(state): State<NoticeApiState>, Extension(current_user): Extension<CurrentUser>) -> ApiResult<NoticeTopResponse> {
    Ok(Json(state.notices.top_notices(&current_user.id).await?))
}

pub async fn mark_notice_read(State(state): State<NoticeApiState>, Extension(current_user): Extension<CurrentUser>, Path(id): Path<String>) -> ApiResult<()> {
    state.notices.mark_read(&id, &current_user.id).await?;
    Ok(Json(()))
}

pub async fn mark_all_notices_read(State(state): State<NoticeApiState>, Extension(current_user): Extension<CurrentUser>) -> ApiResult<()> {
    state.notices.mark_all_read(&current_user.id).await?;
    Ok(Json(()))
}

#[require_perms("system:notice:list")]
pub async fn list_notice_readers(
    State(state): State<NoticeApiState>,
    Path(id): Path<String>,
    RequestQuery(query): RequestQuery<NoticeReaderQuery>,
) -> ApiResult<Page<NoticeReader>> {
    Ok(Json(state.notices.page_readers(&id, query.into()).await?))
}

impl From<NoticeListQuery> for NoticeListFilter {
    fn from(value: NoticeListQuery) -> Self {
        Self {
            page: PageRequest {
                page: value.page,
                page_size: value.page_size,
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
            page: PageRequest {
                page: value.page,
                page_size: value.page_size,
            },
            search_value: value.search_value.or(value.user_name),
        }
    }
}

#[cfg(test)]
mod tests;
