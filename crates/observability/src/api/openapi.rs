use types::http::ApiErrorResponse;
use utoipa::{IntoResponses, Modify, OpenApi};

use super::dto::{
    BatchIdsRequest, SystemLogCleanupAcceptedResponse, SystemLogCleanupCountResponse, SystemLogCleanupExecutionResponse, SystemLogDetailResponse,
    SystemLogSummaryResponse,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        super::handlers::list_system_logs,
        super::handlers::get_system_log,
        super::handlers::delete_system_log,
        super::handlers::delete_system_logs,
        super::handlers::count_system_logs_for_cleanup,
        super::handlers::clean_system_logs,
        super::handlers::get_system_log_cleanup_execution,
        super::handlers::export_system_logs
    ),
    components(schemas(
        ApiErrorResponse,
        BatchIdsRequest,
        SystemLogCleanupCountResponse,
        SystemLogCleanupAcceptedResponse,
        SystemLogCleanupExecutionResponse,
        SystemLogDetailResponse,
        SystemLogSummaryResponse
    )),
    modifiers(&BearerSecurity),
    tags((name = "observability-system-log", description = "System log management"))
)]
pub struct SystemLogApiDoc;

struct BearerSecurity;

impl Modify for BearerSecurity {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

        openapi.components.get_or_insert_default().add_security_scheme(
            "bearerAuth",
            SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).bearer_format("JWT").build()),
        );
    }
}

#[allow(dead_code)]
#[derive(IntoResponses)]
pub(super) enum SystemLogErrorResponses {
    #[response(status = 401, description = "Authentication is required")]
    Unauthorized(ApiErrorResponse),
    #[response(status = 403, description = "The account lacks the required permission")]
    Forbidden(ApiErrorResponse),
    #[response(status = 503, description = "System log service is unavailable")]
    ServiceUnavailable(ApiErrorResponse),
}
