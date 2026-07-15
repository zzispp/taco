use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use serde_json::Value;
use tower::ServiceExt;

use super::support::{json_body, sign_in, test_router};

const LIST_USERS_URI: &str = "/api/system/users?limit=10";

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
            .oneshot(authenticated_query(AuthenticatedQuery {
                method: Method::GET,
                uri: &uri,
                access_token: &tokens.access_token,
                locale,
            }))
            .await
            .unwrap();

        assert_invalid_input(response.status(), json_body(response).await, details);
    }
}

#[tokio::test]
async fn user_list_rejects_reversed_create_time_range() {
    let router = test_router();
    let tokens = sign_in(router.clone()).await;
    let uri = format!("{LIST_USERS_URI}&begin_time=2026-07-08T12:00:00.001Z&end_time=2026-07-08T12:00:00.000Z");
    let response = router
        .oneshot(authenticated_query(AuthenticatedQuery {
            method: Method::GET,
            uri: &uri,
            access_token: &tokens.access_token,
            locale: "zh-CN",
        }))
        .await
        .unwrap();

    assert_invalid_input(response.status(), json_body(response).await, "用户创建时间范围无效，开始时间不能晚于结束时间");
}

#[tokio::test]
async fn user_export_validates_create_time_before_loading_export_config() {
    let router = test_router();
    let tokens = sign_in(router.clone()).await;
    let response = router
        .oneshot(authenticated_query(AuthenticatedQuery {
            method: Method::POST,
            uri: "/api/system/users/export?begin_time=not-a-time",
            access_token: &tokens.access_token,
            locale: "en",
        }))
        .await
        .unwrap();

    assert_invalid_input(
        response.status(),
        json_body(response).await,
        "Invalid user creation time filter. Use YYYY-MM-DD / RFC3339.",
    );
}

#[tokio::test]
async fn user_list_query_deserialization_uses_localized_api_error_shape() {
    let router = test_router();
    let tokens = sign_in(router.clone()).await;
    let response = router
        .oneshot(authenticated_query(AuthenticatedQuery {
            method: Method::GET,
            uri: "/api/system/users?limit=invalid",
            access_token: &tokens.access_token,
            locale: "en",
        }))
        .await
        .unwrap();

    assert_invalid_input(response.status(), json_body(response).await, "Invalid input");
}

#[tokio::test]
async fn user_list_rejects_a_malformed_cursor() {
    let router = test_router();
    let tokens = sign_in(router.clone()).await;
    let response = router
        .oneshot(authenticated_query(AuthenticatedQuery {
            method: Method::GET,
            uri: "/api/system/users?limit=20&cursor=broken",
            access_token: &tokens.access_token,
            locale: "en",
        }))
        .await
        .unwrap();
    let body = json_body(response).await;

    assert_eq!(body["code"], "invalid_cursor");
    assert_eq!(body["message"], "The cursor is invalid or no longer matches this query");
}

#[tokio::test]
async fn public_user_queries_reject_legacy_page_parameters() {
    let router = test_router();
    let tokens = sign_in(router.clone()).await;
    let cases = [
        (Method::GET, "/api/system/users?page=1"),
        (Method::POST, "/api/system/users/export?page_size=20"),
        (Method::GET, "/api/system/online/list?page=1&page_size=20"),
    ];

    for (method, uri) in cases {
        let response = router
            .clone()
            .oneshot(authenticated_query(AuthenticatedQuery {
                method,
                uri,
                access_token: &tokens.access_token,
                locale: "en",
            }))
            .await
            .unwrap();
        let status = response.status();
        let body = json_body(response).await;

        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri}");
        assert_eq!(body["code"], "invalid_input", "{uri}");
    }
}

struct AuthenticatedQuery<'a> {
    method: Method,
    uri: &'a str,
    access_token: &'a str,
    locale: &'a str,
}

fn authenticated_query(input: AuthenticatedQuery<'_>) -> Request<Body> {
    Request::builder()
        .method(input.method)
        .uri(input.uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", input.access_token))
        .header(header::ACCEPT_LANGUAGE, input.locale)
        .body(Body::empty())
        .unwrap()
}

fn assert_invalid_input(status: StatusCode, body: Value, details: &str) {
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "invalid_input");
    assert_eq!(body["details"], details);
}
