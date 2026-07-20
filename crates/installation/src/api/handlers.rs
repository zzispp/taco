use axum::{
    Json,
    extract::State,
    http::{HeaderValue, header},
    response::IntoResponse,
};
use types::http::RequestJson;

use super::{
    dto::{
        ConnectionTestResponse, InstallationPayload, InstallationResponse, InstallationStatusResponse, PostgresConnectionPayload, RedisConnectionPayload,
        SetupDefaultsResponse,
    },
    error::SetupApiError,
    state::{InstallationApiState, SetupApiState},
};

const CACHE_CONTROL_VALUE: &str = "no-store";

pub(super) async fn get_status(State(state): State<InstallationApiState>) -> impl IntoResponse {
    status_response(state.status)
}

pub(super) async fn get_setup_status(State(state): State<SetupApiState>) -> impl IntoResponse {
    status_response(state.status)
}

pub(super) async fn get_setup_defaults() -> impl IntoResponse {
    no_store(Json(SetupDefaultsResponse::from_profile(configuration::InstallationProfile::default())))
}

pub(super) async fn test_postgres(
    State(state): State<SetupApiState>,
    RequestJson(payload): RequestJson<PostgresConnectionPayload>,
) -> Result<impl IntoResponse, SetupApiError> {
    state.setup.test_postgres(payload.try_into()?).await?;
    Ok(no_store(Json(ConnectionTestResponse::valid())))
}

pub(super) async fn test_redis(
    State(state): State<SetupApiState>,
    RequestJson(payload): RequestJson<RedisConnectionPayload>,
) -> Result<impl IntoResponse, SetupApiError> {
    state.setup.test_redis(payload.try_into()?).await?;
    Ok(no_store(Json(ConnectionTestResponse::valid())))
}

pub(super) async fn install(
    State(state): State<SetupApiState>,
    RequestJson(payload): RequestJson<InstallationPayload>,
) -> Result<impl IntoResponse, SetupApiError> {
    state.setup.install(payload.try_into()?).await?;
    Ok(no_store(Json(InstallationResponse::completed())))
}

fn status_response(status: crate::application::InstallationStatus) -> impl IntoResponse {
    (
        [(header::CACHE_CONTROL, HeaderValue::from_static(CACHE_CONTROL_VALUE))],
        Json(InstallationStatusResponse::from(status)),
    )
}

fn no_store<T>(body: Json<T>) -> ([(axum::http::header::HeaderName, HeaderValue); 1], Json<T>) {
    ([(header::CACHE_CONTROL, HeaderValue::from_static(CACHE_CONTROL_VALUE))], body)
}
