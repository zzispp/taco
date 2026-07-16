use std::net::SocketAddr;

use crate::{
    api::dto::{AuthSessionResponse, TokenPairResponse},
    application::AppError,
    domain::User,
};
use axum::{
    extract::{ConnectInfo, OriginalUri, State},
    http::{HeaderMap, HeaderValue, header},
};

use super::{
    ApiResult, ApiState, CookieApiJson, TokenPair,
    auth_events::{AuthEventPublisher, AuthenticationEventContext},
    auth_json::{AuthenticationRequestJson, SignInJson, SignUpJson},
    support::{issue_tokens_for_user, new_sign_up_user, reject_disabled_registration, verify_account_captcha},
};

type SignUpRequest = (State<ApiState>, ConnectInfo<SocketAddr>, OriginalUri, HeaderMap, SignUpJson);
type SignInRequest = (State<ApiState>, ConnectInfo<SocketAddr>, OriginalUri, HeaderMap, SignInJson);
type RefreshRequest = (State<ApiState>, ConnectInfo<SocketAddr>, OriginalUri, HeaderMap);
type LogoutRequest = (State<ApiState>, ConnectInfo<SocketAddr>, OriginalUri, HeaderMap);
const REQUEST_ID_HEADER: &str = "x-request-id";

struct AuthTokenRequest<'a> {
    state: &'a ApiState,
    client: &'a client_info::ClientInfo,
    user: &'a User,
}

pub async fn sign_up(
    (State(state), ConnectInfo(peer), original_uri, headers, AuthenticationRequestJson(payload, _)): SignUpRequest,
) -> ApiResult<CookieApiJson<AuthSessionResponse>> {
    let client = client_info::ClientInfo::from_headers(&headers, peer);
    let events = AuthEventPublisher::new(state.security_audit.as_ref(), event_context(&client, &headers, original_uri.0.path().into()));
    let username = payload.username.trim().to_owned();
    if let Err(error) = prepare_registration(&state, payload.captcha_token.as_deref()).await {
        return register_failure(&events, username, error).await;
    }
    let registration_audit = events.register_success_record(username.clone());
    let user = match state.users.sign_up_with_audit(new_sign_up_user(payload), registration_audit).await {
        Ok(user) => user,
        Err(error) => return register_failure(&events, username, error).await,
    };
    let tokens = issue_auth_tokens(AuthTokenRequest {
        state: &state,
        client: &client,
        user: &user,
    })
    .await?;
    auth_session_response(&state, user, tokens)
}

pub async fn sign_in(
    (State(state), ConnectInfo(peer), original_uri, headers, AuthenticationRequestJson(payload, _)): SignInRequest,
) -> ApiResult<CookieApiJson<AuthSessionResponse>> {
    let client = client_info::ClientInfo::from_headers(&headers, peer);
    let route = original_uri.0.path().to_owned();
    let events = AuthEventPublisher::new(state.security_audit.as_ref(), event_context(&client, &headers, route.clone()));
    let username = payload.identifier.trim().to_owned();
    if let Err(error) = verify_account_captcha(&state, payload.captcha_token.as_deref()).await {
        events.login_failure(username, &error).await?;
        return Err(error.into());
    }
    let login = match state.users.sign_in(payload.into()).await {
        Ok(login) => login,
        Err(error) => return login_failure(&events, username, error).await,
    };
    let canonical_username = login.user().username.clone();
    let tokens = match issue_auth_tokens(AuthTokenRequest {
        state: &state,
        client: &client,
        user: login.user(),
    })
    .await
    {
        Ok(tokens) => tokens,
        Err(error) => return login_failure(&events, canonical_username, error).await,
    };
    let login_audit = events.login_success_record(login.user());
    let user = match state.users.complete_sign_in_with_audit(login, client.ipaddr(), login_audit.clone()).await {
        Ok(user) => user,
        Err(error) => {
            if let Err(revoke_error) = state.tokens.logout_access(&tokens.access_token).await {
                taco_tracing::error_with_fields!(
                    "failed to revoke incomplete login session",
                    &revoke_error,
                    request_id = request_id(&headers),
                    route = route
                );
            }
            return login_failure(&events, canonical_username, error).await;
        }
    };
    auth_session_response(&state, user, tokens)
}

pub async fn logout((State(state), ConnectInfo(peer), original_uri, headers): LogoutRequest) -> ApiResult<CookieApiJson<()>> {
    let client = client_info::ClientInfo::from_headers(&headers, peer);
    let events = AuthEventPublisher::new(state.security_audit.as_ref(), event_context(&client, &headers, original_uri.0.path().into()));
    if let Err(error) = state.auth_http.require_trusted_origin(&headers) {
        events.logout_failure(String::new(), &error).await?;
        return Err(error.into());
    }
    let refresh_token = match state.auth_http.refresh_token(&headers) {
        Ok(token) => token,
        Err(error) => {
            events.logout_failure(String::new(), &error).await?;
            return Err(error.into());
        }
    };
    let session = match state.tokens.logout_refresh(refresh_token).await {
        Ok(session) => session,
        Err(error) => {
            events.logout_failure(String::new(), &error).await?;
            return Err(error.into());
        }
    };
    events.logout_success(session.user_name, session.user_id).await?;
    cookie_response((), state.auth_http.cleared_cookie()?)
}

pub async fn refresh((State(state), ConnectInfo(peer), original_uri, headers): RefreshRequest) -> ApiResult<CookieApiJson<TokenPairResponse>> {
    let client = client_info::ClientInfo::from_headers(&headers, peer);
    let events = AuthEventPublisher::new(state.security_audit.as_ref(), event_context(&client, &headers, original_uri.0.path().into()));
    if let Err(error) = state.auth_http.require_trusted_origin(&headers) {
        events.refresh_failure(String::new(), &error).await?;
        return Err(error.into());
    }
    let refresh_token = match state.auth_http.refresh_token(&headers) {
        Ok(token) => token,
        Err(error) => {
            events.refresh_failure(String::new(), &error).await?;
            return Err(error.into());
        }
    };
    let (user_id, tokens) = match state.tokens.refresh(refresh_token).await {
        Ok(result) => result,
        Err(error) => {
            events.refresh_failure(String::new(), &error).await?;
            return Err(error.into());
        }
    };
    let user = match state.users.authenticated_user(user_id).await {
        Ok(user) => user,
        Err(error) => {
            events.refresh_failure(String::new(), &error).await?;
            return Err(error.into());
        }
    };
    events.refresh_success(&user).await?;
    let cookie = state.auth_http.issued_cookie(&tokens.refresh_token, tokens.refresh_token_ttl_seconds)?;
    cookie_response(tokens.into(), cookie)
}

fn auth_session_response(state: &ApiState, user: User, tokens: TokenPair) -> ApiResult<CookieApiJson<AuthSessionResponse>> {
    let cookie = state.auth_http.issued_cookie(&tokens.refresh_token, tokens.refresh_token_ttl_seconds)?;
    cookie_response(AuthSessionResponse::new(user.into(), tokens), cookie)
}

fn cookie_response<T>(value: T, cookie: HeaderValue) -> ApiResult<CookieApiJson<T>> {
    let mut headers = HeaderMap::new();
    headers.insert(header::SET_COOKIE, cookie);
    Ok((headers, axum::Json(value)))
}

fn event_context(client: &client_info::ClientInfo, headers: &HeaderMap, route: String) -> AuthenticationEventContext {
    AuthenticationEventContext::from_client(client, request_id(headers), route)
}

fn request_id(headers: &HeaderMap) -> String {
    headers
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned()
}

async fn prepare_registration(state: &ApiState, captcha_token: Option<&str>) -> Result<(), AppError> {
    reject_disabled_registration(state).await?;
    verify_account_captcha(state, captcha_token).await
}

async fn issue_auth_tokens(input: AuthTokenRequest<'_>) -> Result<TokenPair, AppError> {
    issue_tokens_for_user(input.state, input.client, input.user).await
}

async fn login_failure<T>(events: &AuthEventPublisher<'_>, username: String, error: AppError) -> ApiResult<T> {
    events.login_failure(username, &error).await?;
    Err(public_login_error(error).into())
}

fn public_login_error(error: AppError) -> AppError {
    match error {
        AppError::Unauthorized | AppError::AccountDisabled | AppError::AccountLocked { .. } => AppError::Unauthorized,
        error => error,
    }
}

async fn register_failure<T>(events: &AuthEventPublisher<'_>, username: String, error: AppError) -> ApiResult<T> {
    events.register_failure(username, &error).await?;
    Err(error.into())
}
