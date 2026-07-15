use audit_contract::{AuditStatus, LoginEventType};
use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use tower::ServiceExt;

use super::super::support::*;

#[tokio::test]
async fn sign_in_rejects_malformed_json_with_uniform_error_shape() {
    let app = test_app();

    let response = app
        .router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/auth/sign-in")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"identifier":"alice","password":"secret""#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;

    assert_eq!(body["code"], "invalid_json");
    assert_eq!(body["message"], "JSON 请求体无效");
    assert_eq!(body["details"], "JSON 请求体格式或字段类型无效");
    let events = app.events.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, LoginEventType::LoginFailure);
    assert_eq!(events[0].status, AuditStatus::Failure);
    assert!(events[0].username.is_empty());
}
