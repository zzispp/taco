use audit_contract::AuditStatus;
use axum::http::{Method, StatusCode};
use tower::ServiceExt;

use super::support::*;

#[tokio::test]
async fn force_logout_invalidates_access_and_refresh_tokens() {
    let app = test_app();
    let tokens = sign_in(app.router.clone()).await;
    let token_id = app.sessions.sessions()[0].token_id.clone();

    let response = app
        .router
        .clone()
        .oneshot(authenticated_request(
            Method::DELETE,
            &format!("/api/system/online/{token_id}"),
            &tokens.access_token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let operation_events = app.operation_events.events();
    assert_eq!(operation_events.len(), 1);
    assert_eq!(operation_events[0].status, AuditStatus::Success);

    let response = app
        .router
        .clone()
        .oneshot(authenticated_request(Method::GET, "/api/auth/me", &tokens.access_token))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let response = app
        .router
        .oneshot(refresh_cookie_request(Method::POST, "/api/auth/refresh", &tokens.refresh_token))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn force_logout_reports_a_durable_audit_failure_after_invalidating_the_session() {
    let app = test_app();
    let tokens = sign_in(app.router.clone()).await;
    let token_id = app.sessions.sessions()[0].token_id.clone();
    app.operation_events.fail_with("outbox unavailable");

    let response = app
        .router
        .clone()
        .oneshot(authenticated_request(
            Method::DELETE,
            &format!("/api/system/online/{token_id}"),
            &tokens.access_token,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(app.sessions.sessions().iter().all(|session| session.token_id != token_id));
    assert_eq!(app.operation_events.events(), Vec::new());
}
