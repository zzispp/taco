use axum::{
    Json,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use kernel::error::LocalizedError;
use types::http::{ApiErrorKind, ApiErrorResponse, current_locale, localized_error_response};

use crate::{application::SetupError, domain::SetupInputError};

const CACHE_CONTROL_VALUE: &str = "no-store";

pub(super) struct SetupApiError(pub(super) SetupError);

impl From<SetupError> for SetupApiError {
    fn from(value: SetupError) -> Self {
        Self(value)
    }
}

impl From<SetupInputError> for SetupApiError {
    fn from(value: SetupInputError) -> Self {
        Self(SetupError::InvalidInput(value))
    }
}

impl IntoResponse for SetupApiError {
    fn into_response(self) -> Response {
        let locale = current_locale();
        let details = localized_details(&self.0);
        let body = localized_error_response(locale, error_kind(&self.0), Some(&details));
        (
            status_code(&self.0),
            [(header::CACHE_CONTROL, HeaderValue::from_static(CACHE_CONTROL_VALUE))],
            Json::<ApiErrorResponse>(body),
        )
            .into_response()
    }
}

fn localized_details(error: &SetupError) -> LocalizedError {
    match error {
        SetupError::InvalidInput(SetupInputError::BlankField(field)) => LocalizedError::new("errors.validation.field_blank").with_param("field", *field),
        SetupError::InvalidInput(SetupInputError::NonPositiveNumber(field)) => {
            LocalizedError::new("errors.installation.field_must_be_positive").with_param("field", *field)
        }
        _ => LocalizedError::new(error.localization_key()),
    }
}

fn error_kind(error: &SetupError) -> ApiErrorKind {
    match error {
        SetupError::InvalidInput(_) | SetupError::InstallationOwnerInvalid => ApiErrorKind::InvalidInput,
        SetupError::AlreadyInstalled | SetupError::InstallationStateAlreadyExists => ApiErrorKind::Conflict,
        _ => ApiErrorKind::Infrastructure,
    }
}

fn status_code(error: &SetupError) -> StatusCode {
    match error {
        SetupError::InvalidInput(_) | SetupError::InstallationOwnerInvalid => StatusCode::BAD_REQUEST,
        SetupError::AlreadyInstalled | SetupError::InstallationStateAlreadyExists => StatusCode::CONFLICT,
        SetupError::PostgresConnectionFailed | SetupError::RedisConnectionFailed => StatusCode::UNPROCESSABLE_ENTITY,
        SetupError::JwtGenerationFailed | SetupError::InvalidGeneratedJwt => StatusCode::INTERNAL_SERVER_ERROR,
        _ => StatusCode::SERVICE_UNAVAILABLE,
    }
}
