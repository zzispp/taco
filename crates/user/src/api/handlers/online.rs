use audit_contract::OperationAuditContext;

use super::*;
use axum::extract::Path;
use rbac_macros::require_perms;

use crate::api::online_session_filter::online_session_page_request;

const ONLINE_SESSION_FORBIDDEN: &str = "errors.user.online_session_forbidden";

type ListOnlineSessionsRequest = (
    State<ApiState>,
    Extension<CurrentUser>,
    Extension<DataScopeFilter>,
    RequestQuery<OnlineSessionsQuery>,
);
type ForceLogoutOnlineSessionRequest = (
    State<ApiState>,
    Extension<CurrentUser>,
    Extension<DataScopeFilter>,
    Option<Extension<OperationAuditContext>>,
    Path<String>,
);

struct OnlineScopeGuard<'a> {
    state: &'a ApiState,
    current_user: &'a CurrentUser,
    data_scope: DataScopeFilter,
}

#[require_perms("system:online:list")]
pub async fn list_online_sessions(request: ListOnlineSessionsRequest) -> ApiResult<ApiJson<OnlineSessionsResponse>> {
    let (State(state), Extension(current_user), Extension(data_scope), RequestQuery(query)) = request;
    let scope = (!current_user.admin).then_some(data_scope);
    let sessions = state.tokens.online_sessions(online_session_page_request(query, scope)?).await?;
    Ok(ok(crate::api::dto::online_sessions_response(sessions)?))
}

#[require_perms("system:online:forceLogout")]
pub async fn force_logout_online_session(request: ForceLogoutOnlineSessionRequest) -> ApiResult<ApiJson<()>> {
    let (State(state), Extension(current_user), Extension(data_scope), audit_context, Path(token_id)) = request;
    let guard = OnlineScopeGuard::new(&state, &current_user, data_scope);
    if let Some(session) = state.tokens.online_session(&token_id).await? {
        guard.ensure_visible(session).await?;
    }
    let audit = super::support::successful_operation_audit(audit_context)?;
    state.tokens.force_logout(&token_id).await?;
    state
        .operation_audit
        .record(audit.record())
        .await
        .map_err(|error| ApiError(AppError::Infrastructure(format!("operation audit outbox recording failed: {error}"))))?;
    audit.mark_persisted();
    Ok(ok(()))
}

impl<'a> OnlineScopeGuard<'a> {
    fn new(state: &'a ApiState, current_user: &'a CurrentUser, data_scope: DataScopeFilter) -> Self {
        Self {
            state,
            current_user,
            data_scope,
        }
    }

    async fn filter(&self, sessions: Vec<OnlineSession>) -> ApiResult<Vec<OnlineSession>> {
        if self.current_user.admin {
            return Ok(sessions);
        }
        self.state
            .users
            .filter_online_sessions_scoped(sessions, self.data_scope.clone())
            .await
            .map_err(ApiError)
    }

    async fn ensure_visible(&self, session: OnlineSession) -> ApiResult<()> {
        let sessions = self.filter(vec![session]).await?;
        if sessions.is_empty() {
            return Err(ApiError(AppError::Forbidden(super::support::localized(ONLINE_SESSION_FORBIDDEN))));
        }
        Ok(())
    }
}
