use axum::http::{Method, StatusCode};
use serde_json::json;
use tower::ServiceExt;

use super::support::*;

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn sign_in_rejects_missing_captcha_when_enabled() {
    let app = test_router_with_captcha(TestCaptcha::enabled());

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
    let body = json_body(response).await;

    assert_eq!(body["code"], "invalid_input");
    assert_eq!(body["message"], "参数错误");
    assert_eq!(body["details"], "请先完成验证码校验");
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn sign_in_accepts_captcha_token_when_enabled() {
    let app = test_router_with_captcha(TestCaptcha::enabled());

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/auth/sign-in",
            json!({
                "identifier": "alice@example.com",
                "password": VALID_PASSWORD,
                "captcha_token": VALID_CAPTCHA_TOKEN
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
async fn sign_up_rejects_missing_captcha_when_enabled() {
    let app = test_router_with_captcha(TestCaptcha::enabled());

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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["code"], "invalid_input");
    assert_eq!(body["message"], "参数错误");
    assert_eq!(body["details"], "请先完成验证码校验");
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn sign_up_accepts_captcha_token_when_enabled() {
    let app = test_router_with_captcha(TestCaptcha::enabled());

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/auth/sign-up",
            json!({
                "username": "bob",
                "email": "bob@example.com",
                "password": VALID_PASSWORD,
                "captcha_token": VALID_CAPTCHA_TOKEN
            }),
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_eq!(body["user"]["username"], "bob");
    assert_non_empty_string(&body["access_token"]);
}
