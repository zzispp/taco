use axum::{Json, extract::State};
use rbac_macros::require_perms;
use types::system_dashboard::ServerDashboard;

use super::{SystemApiError, SystemApiState};

type ApiJson<T> = Json<T>;
type ApiResult<T> = Result<T, SystemApiError>;

#[require_perms("system:dashboard:view")]
pub async fn get_server_dashboard(State(state): State<SystemApiState>) -> ApiResult<ApiJson<ServerDashboard>> {
    Ok(Json(state.metrics.dashboard().await?))
}
