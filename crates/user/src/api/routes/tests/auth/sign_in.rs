use audit_contract::{AuditStatus, LoginEventType};
use axum::{
    Router,
    http::{Method, StatusCode, header},
};
use serde_json::json;
use tower::ServiceExt;

use crate::test_support::{MemoryUserRepository, stored_user};

use super::super::support::*;

#[tokio::test]
async fn sign_in_accepts_email_identifier_and_returns_access_token() {
    let app = test_router();

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/auth/sign-in",
            json!({
                "identifier": "alice@example.com",
                "password": VALID_PASSWORD
            }),
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_eq!(body["user"]["username"], "alice");
    assert_non_empty_string(&body["access_token"]);
    assert!(body.get("refresh_token").is_none());
}

#[tokio::test]
async fn sign_in_sets_strict_http_only_refresh_cookie_without_exposing_refresh_token() {
    let app = test_router();

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/auth/sign-in",
            json!({
                "identifier": "alice",
                "password": VALID_PASSWORD
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response.headers().get(header::SET_COOKIE).unwrap().to_str().unwrap();
    assert!(cookie.starts_with("refresh_token="));
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("Secure"));
    assert!(cookie.contains("SameSite=Strict"));
    assert!(cookie.contains("Path=/api/auth"));
    assert!(cookie.contains("Max-Age=604800"));
    assert!(!cookie.contains("Domain="));
    let body = json_body(response).await;
    assert!(body.get("refresh_token").is_none());
    assert_non_empty_string(&body["access_token"]);
}

#[tokio::test]
async fn sign_in_revokes_session_and_publishes_failure_when_finalization_fails() {
    let app = test_app_with_failed_login_cleanup();

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
    let body = json_body(response).await;
    let events = app.events.events();

    assert_eq!(body["code"], "infrastructure_error");
    assert_eq!(app.sessions.sessions(), Vec::new());
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].status, AuditStatus::Failure);
    assert_eq!(app.events.session_counts(), vec![0]);
}

#[tokio::test]
async fn sign_in_publishes_failed_credentials_event_without_session() {
    let app = test_app();

    let response = app
        .router
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/auth/sign-in",
            json!({"identifier":"alice","password":"bad-password"}),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let events = app.events.events();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, LoginEventType::LoginFailure);
    assert_eq!(events[0].status, AuditStatus::Failure);
    assert_eq!(events[0].username, "alice");
    assert_eq!(events[0].message_key, "errors.user.invalid_credentials");
    assert_eq!(app.events.session_counts(), vec![0]);
}

#[tokio::test]
async fn login_identity_and_account_state_failures_have_identical_public_responses() {
    let expected = sign_in_error(test_router(), "alice", "bad-password").await;
    let unknown = sign_in_error(test_router(), "missing", "bad-password").await;
    let disabled = sign_in_error(
        test_router_with_repository(
            MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123").with_status("1")),
            TestConfig::new(true),
        ),
        "alice",
        VALID_PASSWORD,
    )
    .await;
    let locked_router = test_router();
    for _ in 0..5 {
        let _ = sign_in_error(locked_router.clone(), "alice", "bad-password").await;
    }
    let locked = sign_in_error(locked_router, "alice", VALID_PASSWORD).await;

    assert_eq!(expected.0, StatusCode::UNAUTHORIZED);
    assert_eq!(unknown, expected);
    assert_eq!(disabled, expected);
    assert_eq!(locked, expected);
}

#[tokio::test]
async fn temporary_login_lock_preserves_the_existing_session() {
    let app = test_app();
    let tokens = sign_in(app.router.clone()).await;
    for _ in 0..5 {
        let _ = sign_in_error(app.router.clone(), "alice", "bad-password").await;
    }

    let response = app
        .router
        .oneshot(authenticated_request(Method::GET, "/api/auth/me", &tokens.access_token))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(app.sessions.sessions().len(), 1);
    assert_eq!(app.sessions.sessions()[0].user_name, "alice");
}

async fn sign_in_error(app: Router, identifier: &str, password: &str) -> (StatusCode, serde_json::Value) {
    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/auth/sign-in",
            json!({ "identifier": identifier, "password": password }),
        ))
        .await
        .unwrap();
    let status = response.status();
    (status, json_body(response).await)
}
