use audit_contract::{AuditStatus, LoginEventType};
use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use tower::ServiceExt;

use super::super::support::*;

#[tokio::test]
async fn refresh_rejects_untrusted_origin_without_renewing_session() {
    let app = test_app();
    let tokens = sign_in(app.router.clone()).await;
    let expires_at = app.sessions.sessions()[0].expires_at;
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/refresh")
        .header(header::COOKIE, format!("refresh_token={}", tokens.refresh_token))
        .header(header::ORIGIN, "https://attacker.example")
        .body(Body::empty())
        .unwrap();

    let response = app.router.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(app.sessions.sessions()[0].expires_at, expires_at);
    let events = app.events.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, LoginEventType::RefreshFailure);
}

#[tokio::test]
async fn me_returns_user_for_bearer_access_token() {
    let app = test_router();
    let tokens = sign_in(app.clone()).await;

    let response = app
        .oneshot(authenticated_request(Method::GET, "/api/auth/me", &tokens.access_token))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_eq!(body["user"]["email"], "alice@example.com");
}

#[tokio::test]
async fn refresh_renews_cookie_and_me_accepts_new_access_token() {
    let app = test_app();
    let tokens = sign_in(app.router.clone()).await;

    let response = app
        .router
        .clone()
        .oneshot(refresh_cookie_request(Method::POST, "/api/auth/refresh", &tokens.refresh_token))
        .await
        .unwrap();
    let cookie = response.headers().get(header::SET_COOKIE).unwrap().to_str().unwrap();
    assert!(cookie.starts_with("refresh_token="));
    assert!(cookie.contains("HttpOnly"));
    let body = response_json(response).await;

    let access_token = body["access_token"].as_str().unwrap();
    assert!(body.get("refresh_token").is_none());

    let response = app
        .router
        .oneshot(authenticated_request(Method::GET, "/api/auth/me", access_token))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_eq!(body["user"]["username"], "alice");
    let events = app.events.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, LoginEventType::RefreshSuccess);
    assert_eq!(events[0].status, AuditStatus::Success);
}

#[tokio::test]
async fn refresh_rejects_access_token() {
    let app = test_app();
    let tokens = sign_in(app.router.clone()).await;

    let response = app
        .router
        .oneshot(refresh_cookie_request(Method::POST, "/api/auth/refresh", &tokens.access_token))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = json_body(response).await;

    assert_eq!(body["code"], "unauthorized");
    let events = app.events.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, LoginEventType::RefreshFailure);
    assert_eq!(events[0].status, AuditStatus::Failure);
}
