use audit_contract::{AuditStatus, LoginEventType};
use axum::http::{Method, StatusCode, header};
use serde_json::json;
use tower::ServiceExt;

use super::super::support::*;

#[tokio::test]
async fn sign_up_accepts_public_payload_and_sets_backend_fields() {
    let app = test_router();

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/auth/sign-up",
            json!({
                "username": "bob",
                "email": "bob@example.com",
                "password": VALID_PASSWORD
            }),
        ))
        .await
        .unwrap();
    let cookie = response.headers().get(header::SET_COOKIE).unwrap().to_str().unwrap();
    assert!(cookie.starts_with("refresh_token="));
    assert!(cookie.contains("HttpOnly"));
    let body = response_json(response).await;

    assert_eq!(body["user"]["role_ids"], json!([]));
    assert_eq!(body["user"]["status"], "0");
    assert_eq!(body["user"]["is_active"], true);
    assert_eq!(body["user"]["auth_source"], "local");
    assert_eq!(body["user"]["email_verified"], false);
    assert_non_empty_string(&body["access_token"]);
}

#[tokio::test]
async fn sign_up_rejection_publishes_register_failure_when_registration_is_disabled() {
    let app = test_app_with_config(TestConfig::new(false));

    let response = app
        .router
        .oneshot(json_request(
            Method::POST,
            "/api/auth/sign-up",
            json!({
                "username": "bob",
                "email": "bob@example.com",
                "password": VALID_PASSWORD
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = json_body(response).await;

    assert_eq!(body["code"], "forbidden");
    let events = app.events.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, LoginEventType::RegisterFailure);
    assert_eq!(events[0].status, AuditStatus::Failure);
}
