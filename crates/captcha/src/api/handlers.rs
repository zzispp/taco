use axum::{Json, extract::State};
use serde_json::Value;
use types::http::RequestJson;

use crate::api::{CaptchaApiError, CaptchaApiState};
use crate::application::CaptchaConfigResponse;

type ApiResult<T> = Result<T, CaptchaApiError>;

pub async fn config(State(state): State<CaptchaApiState>) -> ApiResult<Json<CaptchaConfigResponse>> {
    Ok(Json(state.captcha.config().await?))
}

pub async fn challenge(State(state): State<CaptchaApiState>) -> ApiResult<Json<Value>> {
    Ok(Json(state.captcha.challenge().await?))
}

pub async fn redeem(State(state): State<CaptchaApiState>, RequestJson(payload): RequestJson<Value>) -> ApiResult<Json<Value>> {
    Ok(Json(state.captcha.redeem(payload).await?))
}
