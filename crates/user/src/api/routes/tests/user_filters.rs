use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use serde_json::Value;
use tower::ServiceExt;

use super::support::{json_body, sign_in, test_router};

const LIST_USERS_URI: &str = "/api/system/users?page=1&page_size=10";

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn user_list_rejects_invalid_create_time_in_all_locales() {
    let cases = [
        ("zh-CN", "用户创建时间筛选格式无效，请使用YYYY-MM-DD / RFC3339"),
        ("en", "Invalid user creation time filter. Use YYYY-MM-DD / RFC3339."),
        ("zh-TW", "使用者建立時間篩選格式無效，請使用YYYY-MM-DD / RFC3339"),
    ];

    for (locale, details) in cases {
        let router = test_router();
        let tokens = sign_in(router.clone()).await;
        let uri = format!("{LIST_USERS_URI}&begin_time=not-a-time");
        let response = router
            .oneshot(authenticated_query(Method::GET, &uri, &tokens.access_token, locale))
            .await
            .unwrap();

        assert_invalid_input(response.status(), json_body(response).await, details);
    }
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn user_list_rejects_reversed_create_time_range() {
    let router = test_router();
    let tokens = sign_in(router.clone()).await;
    let uri = format!("{LIST_USERS_URI}&begin_time=2026-07-08T12:00:00.001Z&end_time=2026-07-08T12:00:00.000Z");
    let response = router
        .oneshot(authenticated_query(Method::GET, &uri, &tokens.access_token, "zh-CN"))
        .await
        .unwrap();

    assert_invalid_input(response.status(), json_body(response).await, "用户创建时间范围无效，开始时间不能晚于结束时间");
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn user_export_validates_create_time_before_loading_export_config() {
    let router = test_router();
    let tokens = sign_in(router.clone()).await;
    let response = router
        .oneshot(authenticated_query(
            Method::POST,
            "/api/system/users/export?begin_time=not-a-time",
            &tokens.access_token,
            "en",
        ))
        .await
        .unwrap();

    assert_invalid_input(
        response.status(),
        json_body(response).await,
        "Invalid user creation time filter. Use YYYY-MM-DD / RFC3339.",
    );
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn user_list_query_deserialization_uses_localized_api_error_shape() {
    let router = test_router();
    let tokens = sign_in(router.clone()).await;
    let response = router
        .oneshot(authenticated_query(
            Method::GET,
            "/api/system/users?page=invalid&page_size=10",
            &tokens.access_token,
            "en",
        ))
        .await
        .unwrap();

    assert_invalid_input(response.status(), json_body(response).await, "Invalid input");
}

fn authenticated_query(method: Method, uri: &str, access_token: &str, locale: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {access_token}"))
        .header(header::ACCEPT_LANGUAGE, locale)
        .body(Body::empty())
        .unwrap()
}

fn assert_invalid_input(status: StatusCode, body: Value, details: &str) {
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "invalid_input");
    assert_eq!(body["details"], details);
}
