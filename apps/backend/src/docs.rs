use axum::{Json, Router, http::StatusCode, routing::get};
use serde::Serialize;
use serde_json::Value;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::openapi::ApiDoc;

pub fn router() -> Router {
    let openapi = ApiDoc::openapi();

    Router::new()
        .route(
            "/openapi.json",
            get({
                let openapi = openapi.clone();
                move || {
                    let openapi = openapi.clone();
                    async move { openapi_json(&openapi) }
                }
            }),
        )
        .merge(Scalar::with_url("/docs", openapi))
}

fn openapi_json(openapi: &impl Serialize) -> Result<Json<Value>, StatusCode> {
    serde_json::to_value(openapi).map(Json).map_err(|error| {
        taco_tracing::error_with_fields!("OpenAPI serialization failed", &error, component = "docs");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}
