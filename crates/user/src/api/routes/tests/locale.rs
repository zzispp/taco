use axum::http::Method;
use serde_json::json;
use tower::ServiceExt;

use super::support::*;

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn sign_in_error_response_uses_requested_english_locale() {
    let app = test_router_with_captcha(TestCaptcha::enabled());

    let response = app
        .oneshot(json_request_with_accept_language(LocalizedJsonRequest {
            method: Method::POST,
            uri: "/api/auth/sign-in",
            body: json!({
                "identifier": "alice@example.com",
                "password": VALID_PASSWORD
            }),
            accept_language: "en-US,en;q=0.9",
        }))
        .await
        .unwrap();
    let body = json_body(response).await;

    assert_eq!(body["code"], "invalid_input");
    assert_eq!(body["message"], "Invalid input");
    assert_eq!(body["details"], "Complete captcha verification first");
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn sign_in_error_response_uses_requested_traditional_chinese_locale() {
    let app = test_router_with_captcha(TestCaptcha::enabled());

    let response = app
        .oneshot(json_request_with_accept_language(LocalizedJsonRequest {
            method: Method::POST,
            uri: "/api/auth/sign-in",
            body: json!({
                "identifier": "alice@example.com",
                "password": VALID_PASSWORD
            }),
            accept_language: "zh-Hant,zh;q=0.9",
        }))
        .await
        .unwrap();
    let body = json_body(response).await;

    assert_eq!(body["code"], "invalid_input");
    assert_eq!(body["message"], "參數錯誤");
    assert_eq!(body["details"], "請先完成驗證碼校驗");
}
