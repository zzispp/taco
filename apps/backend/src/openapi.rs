use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::system::health
    ),
    tags(
        (name = "system", description = "System endpoints")
    )
)]
pub struct ApiDoc;
