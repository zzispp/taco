use std::sync::Arc;

use axum::{
    Extension, Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
    middleware,
};
use constants::system::{ALL_PERMISSION, STATUS_NORMAL};
use rbac::api::CurrentUser;
use serde_json::{Value, json};
use tower::ServiceExt;

use super::{NoticeApiState, create_router};
use crate::notice::{NoticeService, domain::NOTICE_STATUS_CLOSED};

mod support;

use support::TestRepository;

const QUERY_PERMISSION: &str = "system:notice:query";
const ADD_PERMISSION: &str = "system:notice:add";

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn closed_notice_requires_admin_wildcard_or_query_permission() {
    let cases = [
        (false, Vec::new(), StatusCode::NOT_FOUND),
        (false, vec![ADD_PERMISSION], StatusCode::NOT_FOUND),
        (false, vec![QUERY_PERMISSION], StatusCode::OK),
        (false, vec![ALL_PERMISSION], StatusCode::OK),
        (true, Vec::new(), StatusCode::OK),
    ];

    for (admin, permissions, expected) in cases {
        let app = test_router(TestRepository::with_status(NOTICE_STATUS_CLOSED), current_user(admin, &permissions));
        let response = app.oneshot(empty_request(Method::GET, "/system/notices/notice-1")).await.unwrap();
        assert_eq!(response.status(), expected, "admin={admin}, permissions={permissions:?}");
    }
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn normal_notice_is_readable_without_notice_permission() {
    let app = test_router(TestRepository::with_status(STATUS_NORMAL), current_user(false, &[]));
    let response = app.oneshot(empty_request(Method::GET, "/system/notices/notice-1")).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn create_notice_uses_authenticated_username_for_audit() {
    let repository = TestRepository::with_status(STATUS_NORMAL);
    let app = test_router(repository.clone(), current_user(false, &[ADD_PERMISSION]));
    let response = app.oneshot(json_request("zh-CN", valid_payload("# Content"))).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["create_by"], "alice");
    assert_eq!(body["update_by"], Value::Null);
    assert_eq!(repository.operators(), vec!["alice"]);
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn invalid_markdown_returns_localized_bad_request() {
    let cases = [
        ("zh-CN", "公告 Markdown 包含原生 HTML 或不安全链接"),
        ("en", "Notice Markdown contains raw HTML or an unsafe link"),
        ("zh-TW", "公告 Markdown 包含原生 HTML 或不安全連結"),
    ];

    for (locale, expected_details) in cases {
        let app = test_router(TestRepository::with_status(STATUS_NORMAL), current_user(false, &[ADD_PERMISSION]));
        let response = app.oneshot(json_request(locale, valid_payload("<script>alert(1)</script>"))).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response_json(response).await;
        assert_eq!(body["code"], "invalid_input");
        assert_eq!(body["details"], expected_details);
    }
}

fn test_router(repository: TestRepository, user: CurrentUser) -> Router {
    let notices = Arc::new(NoticeService::new(repository));
    create_router(NoticeApiState::new(notices))
        .layer(Extension(user))
        .layer(middleware::from_fn(types::http::locale_middleware))
}

fn current_user(admin: bool, permissions: &[&str]) -> CurrentUser {
    CurrentUser {
        id: "user-1".into(),
        username: "alice".into(),
        role_keys: Vec::new(),
        permissions: permissions.iter().map(|permission| (*permission).into()).collect(),
        dept_id: None,
        admin,
        system: false,
    }
}

fn valid_payload(content: &str) -> Value {
    json!({
        "notice_title": "Notice",
        "notice_type": "1",
        "notice_content": content,
        "status": STATUS_NORMAL,
        "remark": null,
        "create_by": "mallory",
        "update_by": "mallory"
    })
}

fn json_request(locale: &str, payload: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri("/system/notices")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCEPT_LANGUAGE, locale)
        .body(Body::from(payload.to_string()))
        .unwrap()
}

fn empty_request(method: Method, uri: &str) -> Request<Body> {
    Request::builder().method(method).uri(uri).body(Body::empty()).unwrap()
}

async fn response_json(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}
