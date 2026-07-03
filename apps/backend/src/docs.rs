use axum::{Json, Router, routing::get};
use serde_json::Value;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};

use crate::openapi::ApiDoc;

pub fn router() -> Router {
    let (_openapi_router, generated) = OpenApiRouter::<()>::with_openapi(ApiDoc::openapi())
        .routes(routes!(crate::system::health))
        .split_for_parts();
    let openapi = generated;

    Router::new()
        .route(
            "/openapi.json",
            get({
                let openapi = openapi.clone();
                move || async move { Json::<Value>(serde_json::to_value(&openapi).expect("openapi should serialize")) }
            }),
        )
        .merge(Scalar::with_url("/docs", openapi))
}
