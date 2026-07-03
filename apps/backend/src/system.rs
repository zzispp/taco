use axum::{Json, Router, routing::get};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    status: &'static str,
}

pub fn create_router() -> Router {
    Router::new().route("/health", get(health))
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "system",
    responses((status = OK, description = "Health check", body = HealthResponse))
)]
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}
