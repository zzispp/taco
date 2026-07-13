use super::*;
use axum::extract::Query;

const ONLINE_SESSION_FORBIDDEN: &str = "errors.user.online_session_forbidden";

type ListOnlineSessionsRequest = (State<ApiState>, Extension<CurrentUser>, Extension<DataScopeFilter>, Query<OnlineSessionsQuery>);
type ForceLogoutOnlineSessionRequest = (State<ApiState>, Extension<CurrentUser>, Extension<DataScopeFilter>, Path<String>);

struct OnlineScopeGuard<'a> {
    state: &'a ApiState,
    current_user: &'a CurrentUser,
    data_scope: DataScopeFilter,
}

#[require_perms("system:online:list")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn list_online_sessions(request: ListOnlineSessionsRequest) -> ApiResult<ApiJson<OnlineSessionsResponse>> {
    let (State(state), Extension(current_user), Extension(data_scope), Query(query)) = request;
    let sessions = state.tokens.online_sessions(query.into()).await?;
    let sessions = OnlineScopeGuard::new(&state, &current_user, data_scope).filter(sessions).await?;
    Ok(ok(sessions.into()))
}

#[require_perms("system:online:forceLogout")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn force_logout_online_session(request: ForceLogoutOnlineSessionRequest) -> ApiResult<ApiJson<()>> {
    let (State(state), Extension(current_user), Extension(data_scope), Path(token_id)) = request;
    let guard = OnlineScopeGuard::new(&state, &current_user, data_scope);
    if let Some(session) = state.tokens.online_session(&token_id).await? {
        guard.ensure_visible(session).await?;
    }
    state.tokens.force_logout(&token_id).await?;
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
