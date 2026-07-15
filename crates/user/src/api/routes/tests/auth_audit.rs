use audit_contract::{AuditStatus, LoginEventType};
use axum::http::{Method, StatusCode};
use serde_json::json;
use tower::ServiceExt;

use super::support::*;

#[tokio::test]
async fn sign_in_persists_its_security_event_with_login_metadata() {
    let app = test_app();

    let tokens = sign_in(app.router.clone()).await;
    let events = app.persisted_security_events();

    assert!(!tokens.access_token.is_empty());
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, LoginEventType::LoginSuccess);
    assert_eq!(events[0].status, AuditStatus::Success);
    assert_eq!(events[0].username, "alice");
    assert_eq!(events[0].ip_address, TEST_PUBLIC_IP);
    assert_eq!(events[0].request_id, "019f5a5c-0823-7c22-acf1-add778ee83bf");
    assert_eq!(events[0].route, "/api/auth/sign-in");
    assert_eq!(events[0].message_key, "messages.user.login_success");
    assert_eq!(app.repository.login_records().len(), 1);
    assert_eq!(app.sessions.sessions().len(), 1);
    assert_eq!(app.events.events(), Vec::new());
}

#[tokio::test]
async fn sign_in_exposes_an_atomic_security_audit_persistence_failure() {
    let app = test_app();
    app.repository.fail_audit_with("outbox unavailable");

    let response = app
        .router
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/auth/sign-in",
            json!({"identifier":"alice","password":VALID_PASSWORD}),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(json_body(response).await["code"], "infrastructure_error");
    assert_eq!(app.repository.login_records(), Vec::new());
    assert_eq!(app.persisted_security_events(), Vec::new());
    let events = app.events.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, LoginEventType::LoginFailure);
    assert_eq!(events[0].status, AuditStatus::Failure);
}

#[tokio::test]
async fn sign_up_persists_its_security_event_with_the_created_user() {
    let app = test_app();

    let response = app
        .router
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/auth/sign-up",
            json!({"username":"bob","email":"bob@example.com","password":VALID_PASSWORD}),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let events = app.persisted_security_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, LoginEventType::RegisterSuccess);
    assert_eq!(events[0].status, AuditStatus::Success);
    assert_eq!(events[0].username, "bob");
    assert_eq!(events[0].user_id.as_deref(), Some("018f0000-0000-7000-8000-000000000001"));
    assert_eq!(app.sessions.sessions().len(), 1);
    assert_eq!(app.events.events(), Vec::new());
}
