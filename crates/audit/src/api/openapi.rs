use types::http::ApiErrorResponse;
use utoipa::{IntoResponses, Modify, OpenApi};

use super::dto::{BatchIdsRequest, LoginLogResponse, OperationLogDetailResponse, OperationLogSummaryResponse};

#[derive(OpenApi)]
#[openapi(
    paths(
        super::handlers::operation::list_operation_logs,
        super::handlers::operation::get_operation_log,
        super::handlers::operation::delete_operation_log,
        super::handlers::operation::delete_operation_logs,
        super::handlers::operation::clear_operation_logs,
        super::handlers::operation::export_operation_logs,
        super::handlers::login::list_login_logs,
        super::handlers::login::delete_login_log,
        super::handlers::login::delete_login_logs,
        super::handlers::login::clear_login_logs,
        super::handlers::login::export_login_logs,
        super::handlers::login::unlock_login
    ),
    components(schemas(
        ApiErrorResponse,
        BatchIdsRequest,
        LoginLogResponse,
        OperationLogDetailResponse,
        OperationLogSummaryResponse
    )),
    modifiers(&BearerSecurity),
    tags(
        (name = "audit-operation", description = "Operation audit log management"),
        (name = "audit-login", description = "Authentication audit log management")
    )
)]
/// OpenAPI contract for audit management routes, intended to be nested under `/api`.
pub struct AuditApiDoc;

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
pub(super) enum AuditErrorResponses {
    #[response(status = 401, description = "Authentication is required")]
    Unauthorized(ApiErrorResponse),
    #[response(status = 403, description = "The account lacks the required permission")]
    Forbidden(ApiErrorResponse),
    #[response(status = 503, description = "Audit service is unavailable")]
    ServiceUnavailable(ApiErrorResponse),
}
