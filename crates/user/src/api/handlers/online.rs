use super::*;

const ONLINE_SESSION_FORBIDDEN: &str = "errors.user.online_session_forbidden";

type ListOnlineSessionsRequest = (State<ApiState>, Extension<CurrentUser>, Extension<DataScopeFilter>, Query<OnlineSessionsQuery>);
type ForceLogoutOnlineSessionRequest = (State<ApiState>, Extension<CurrentUser>, Extension<DataScopeFilter>, Path<String>);

#[require_perms("system:online:list")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn list_online_sessions(request: ListOnlineSessionsRequest) -> ApiResult<ApiJson<OnlineSessionsResponse>> {
    let (State(state), Extension(current_user), Extension(data_scope), Query(query)) = request;
    let sessions = state.tokens.online_sessions(query.into()).await?;
    let sessions = scoped_online_sessions(&state, &current_user, sessions, data_scope).await?;
    Ok(ok(sessions.into()))
}

#[require_perms("system:online:forceLogout")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn force_logout_online_session(request: ForceLogoutOnlineSessionRequest) -> ApiResult<ApiJson<()>> {
    let (State(state), Extension(current_user), Extension(data_scope), Path(token_id)) = request;
    if let Some(session) = state.tokens.online_session(&token_id).await? {
        ensure_online_session_visible(&state, &current_user, session, data_scope).await?;
    }
    state.tokens.force_logout(&token_id).await?;
    Ok(ok(()))
}

async fn scoped_online_sessions(
    state: &ApiState,
    current_user: &CurrentUser,
    sessions: Vec<OnlineSession>,
    data_scope: DataScopeFilter,
) -> ApiResult<Vec<OnlineSession>> {
    if current_user.admin {
        return Ok(sessions);
    }
    state.users.filter_online_sessions_scoped(sessions, data_scope).await.map_err(ApiError)
}

async fn ensure_online_session_visible(state: &ApiState, current_user: &CurrentUser, session: OnlineSession, data_scope: DataScopeFilter) -> ApiResult<()> {
    let sessions = scoped_online_sessions(state, current_user, vec![session], data_scope).await?;
    if sessions.is_empty() {
        return Err(ApiError(AppError::Forbidden(super::support::localized(ONLINE_SESSION_FORBIDDEN))));
    }
    Ok(())
}
