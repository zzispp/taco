use audit_contract::{AuditStatus, LoginEventType};
use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use tower::ServiceExt;

use super::support::{refresh_cookie_request, sign_in, test_app};

#[tokio::test]
async fn logout_deletes_session_and_publishes_success() {
    let app = test_app();
    let tokens = sign_in(app.router.clone()).await;

    let response = app
        .router
        .clone()
        .oneshot(refresh_cookie_request(Method::POST, "/api/auth/logout", &tokens.refresh_token))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let cleared_cookie = response.headers().get(header::SET_COOKIE).unwrap().to_str().unwrap();
    assert!(cleared_cookie.starts_with("refresh_token="));
    assert!(cleared_cookie.contains("Max-Age=0"));
    assert!(cleared_cookie.contains("HttpOnly"));
    assert!(cleared_cookie.contains("SameSite=Strict"));
    assert!(app.sessions.sessions().is_empty());
    let events = app.events.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, LoginEventType::LogoutSuccess);
    assert_eq!(events[0].status, AuditStatus::Success);
    assert_eq!(events[0].username, "alice");
    assert_eq!(app.persisted_security_events().len(), 1);
}

#[tokio::test]
async fn logout_rejects_untrusted_origin_without_deleting_session() {
    let app = test_app();
    let tokens = sign_in(app.router.clone()).await;
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/logout")
        .header(header::COOKIE, format!("refresh_token={}", tokens.refresh_token))
        .header(header::ORIGIN, "https://attacker.example")
        .body(Body::empty())
        .unwrap();

    let response = app.router.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(app.sessions.sessions().len(), 1);
    let events = app.events.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, LoginEventType::LogoutFailure);
}

#[tokio::test]
async fn logout_with_an_invalid_token_publishes_failure() {
    let app = test_app();

    let response = app
        .router
        .clone()
        .oneshot(refresh_cookie_request(Method::POST, "/api/auth/logout", "invalid-token"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let events = app.events.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, LoginEventType::LogoutFailure);
    assert_eq!(events[0].status, AuditStatus::Failure);
    assert!(events[0].username.is_empty());
}

#[tokio::test]
async fn logout_failure_returns_an_explicit_error_when_its_security_record_cannot_be_persisted() {
    let app = test_app();
    app.events.fail_with("outbox unavailable");

    let response = app
        .router
        .clone()
        .oneshot(refresh_cookie_request(Method::POST, "/api/auth/logout", "invalid-token"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(app.events.events().is_empty());
}
