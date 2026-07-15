use std::{marker::PhantomData, net::SocketAddr};

use audit_contract::LoginEventType;
use axum::{
    extract::{ConnectInfo, FromRequest, OriginalUri, Request},
    response::{IntoResponse, Response},
};
use kernel::error::LocalizedError;
use serde::de::DeserializeOwned;
use types::http::{RequestJson, RequestJsonRejection};

use crate::{
    api::{
        ApiState,
        dto::{SignInPayload, SignUpPayload},
        error::ApiError,
    },
    application::AppError,
};

use super::auth_events::{AuthEventPublisher, AuthenticationEventContext};

const REQUEST_ID_HEADER: &str = "x-request-id";
const INVALID_JSON_ERROR_KEY: &str = "errors.common.invalid_json";

pub(crate) type SignInJson = AuthenticationRequestJson<SignInPayload, LoginFailure>;
pub(crate) type SignUpJson = AuthenticationRequestJson<SignUpPayload, RegisterFailure>;

pub(crate) struct AuthenticationRequestJson<T, K>(pub T, pub(crate) PhantomData<K>);

pub(crate) enum AuthenticationRequestJsonRejection {
    Json(RequestJsonRejection),
    Audit(ApiError),
}

impl IntoResponse for AuthenticationRequestJsonRejection {
    fn into_response(self) -> Response {
        match self {
            Self::Json(error) => error.into_response(),
            Self::Audit(error) => error.into_response(),
        }
    }
}

pub(crate) trait AuthenticationFailureKind {
    const EVENT_TYPE: LoginEventType;
}

pub(crate) struct LoginFailure;
pub(crate) struct RegisterFailure;

impl AuthenticationFailureKind for LoginFailure {
    const EVENT_TYPE: LoginEventType = LoginEventType::LoginFailure;
}

impl AuthenticationFailureKind for RegisterFailure {
    const EVENT_TYPE: LoginEventType = LoginEventType::RegisterFailure;
}

impl<T, K> FromRequest<ApiState> for AuthenticationRequestJson<T, K>
where
    T: DeserializeOwned + Send,
    K: AuthenticationFailureKind + Send,
{
    type Rejection = AuthenticationRequestJsonRejection;

    async fn from_request(request: Request, state: &ApiState) -> Result<Self, Self::Rejection> {
        let context = rejection_context(&request)?;
        match RequestJson::<T>::from_request(request, state).await {
            Ok(RequestJson(value)) => Ok(Self(value, PhantomData)),
            Err(error) => {
                record_rejection(state, context, K::EVENT_TYPE).await?;
                Err(AuthenticationRequestJsonRejection::Json(error))
            }
        }
    }
}

fn rejection_context(request: &Request) -> Result<AuthenticationEventContext, AuthenticationRequestJsonRejection> {
    let peer = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|value| value.0)
        .ok_or_else(missing_peer_error)?;
    let route = request
        .extensions()
        .get::<OriginalUri>()
        .map(|value| value.0.path().to_owned())
        .unwrap_or_else(|| request.uri().path().to_owned());
    let client = client_info::ClientInfo::from_headers(request.headers(), peer);
    Ok(AuthenticationEventContext::from_client(&client, request_id(request), route))
}

async fn record_rejection(state: &ApiState, context: AuthenticationEventContext, event_type: LoginEventType) -> Result<(), AuthenticationRequestJsonRejection> {
    let error = AppError::InvalidInput(LocalizedError::new(INVALID_JSON_ERROR_KEY));
    AuthEventPublisher::new(state.security_audit.as_ref(), context)
        .authentication_failure(event_type, String::new(), &error)
        .await
        .map_err(|error| AuthenticationRequestJsonRejection::Audit(ApiError(error)))
}

fn request_id(request: &Request) -> String {
    request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned()
}

fn missing_peer_error() -> AuthenticationRequestJsonRejection {
    AuthenticationRequestJsonRejection::Audit(ApiError(AppError::Infrastructure(
        "authentication audit requires peer connection information".into(),
    )))
}
