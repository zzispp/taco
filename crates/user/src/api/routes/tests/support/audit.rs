use std::{net::Ipv4Addr, sync::Arc};

use audit_contract::{ActorSnapshot, BusinessType, OperationAuditContext, OperationAuditSeed, OperationRequestSnapshot, OperatorType};
use axum::{extract::Request, middleware::Next, response::Response};

pub(super) async fn operation_context_middleware(mut request: Request, next: Next) -> Response {
    let context = OperationAuditContext::new(
        OperationAuditSeed {
            id: "test-operation-audit".into(),
            occurred_at: time::OffsetDateTime::UNIX_EPOCH,
            title_key: "audit.module.user".into(),
            business_type: BusinessType::Update,
            handler: "user::test".into(),
            request_method: request.method().as_str().into(),
            operator_type: OperatorType::Manage,
            operation_url: request.uri().path().into(),
            operation_ip: Ipv4Addr::LOCALHOST.to_string(),
            request_id: "test-request".into(),
        },
        Arc::new(TestOperationRequestSnapshot),
    );
    context
        .set_actor(ActorSnapshot {
            user_id: Some("1".into()),
            username: "admin".into(),
            department_id: Some("103".into()),
            department_name: "研发部门".into(),
        })
        .unwrap();
    request.extensions_mut().insert(context);
    next.run(request).await
}

struct TestOperationRequestSnapshot;

impl OperationRequestSnapshot for TestOperationRequestSnapshot {
    fn request_params(&self) -> String {
        "{}".into()
    }
}
