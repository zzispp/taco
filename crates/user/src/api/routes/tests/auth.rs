use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use serde_json::json;
use tower::ServiceExt;

use super::support::*;

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn sign_in_accepts_email_identifier_and_returns_token_pair() {
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
    assert_non_empty_string(&body["refresh_token"]);
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
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
    let body = response_json(response).await;

    assert_eq!(body["user"]["role_ids"], json!(["2"]));
    assert_eq!(body["user"]["status"], "0");
    assert_eq!(body["user"]["is_active"], true);
    assert_eq!(body["user"]["auth_source"], "local");
    assert_eq!(body["user"]["email_verified"], false);
    assert_non_empty_string(&body["access_token"]);
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn sign_up_rejects_when_registration_is_disabled() {
    let app = test_router_with_config(TestConfig::new(false));

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
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = json_body(response).await;

    assert_eq!(body["code"], "forbidden");
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
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

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn refresh_returns_new_token_pair_and_me_accepts_new_access_token() {
    let app = test_router();
    let tokens = sign_in(app.clone()).await;

    let response = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/auth/refresh",
            json!({ "refresh_token": tokens.refresh_token }),
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    let access_token = body["access_token"].as_str().unwrap();
    assert_non_empty_string(&body["refresh_token"]);

    let response = app.oneshot(authenticated_request(Method::GET, "/api/auth/me", access_token)).await.unwrap();
    let body = response_json(response).await;

    assert_eq!(body["user"]["username"], "alice");
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn refresh_rejects_access_token() {
    let app = test_router();
    let tokens = sign_in(app.clone()).await;

    let response = app
        .oneshot(json_request(Method::POST, "/api/auth/refresh", json!({ "refresh_token": tokens.access_token })))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = json_body(response).await;

    assert_eq!(body["code"], "unauthorized");
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn sign_in_rejects_malformed_json_with_uniform_error_shape() {
    let app = test_router();

    let response = app
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
}
