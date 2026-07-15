use axum::http::{Method, StatusCode};
use serde_json::json;
use tower::ServiceExt;

use super::support::*;
use crate::test_support::{MemoryUserRepository, stored_user};

const JULY_8_2026_NOON_UTC_MILLIS: i64 = 1_783_512_000_000;

#[tokio::test]
async fn sign_in_creates_online_session() {
    let app = test_app();

    sign_in(app.router).await;

    let sessions = app.sessions.sessions();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].user_name, "alice");
    assert_eq!(sessions[0].dept_name.as_deref(), Some("部门103"));
    assert_eq!(sessions[0].ipaddr, TEST_PUBLIC_IP);
    assert_eq!(sessions[0].login_location, TEST_LOGIN_LOCATION);
}

#[tokio::test]
async fn repeated_sign_in_keeps_multiple_online_sessions_like_ruoyi() {
    let app = test_app();

    sign_in(app.router.clone()).await;
    sign_in(app.router).await;

    let sessions = app.sessions.sessions();
    assert_eq!(sessions.len(), 2);
    assert!(sessions.iter().all(|session| session.user_name == "alice"));
    assert_ne!(sessions[0].token_id, sessions[1].token_id);
}

#[tokio::test]
async fn online_list_returns_aligned_rows_and_filters_fuzzily() {
    let app = test_app();
    let tokens = sign_in(app.router.clone()).await;

    let response = app
        .router
        .oneshot(authenticated_request(
            Method::GET,
            "/api/system/online/list?ipaddr=8.8&userName=LIC",
            &tokens.access_token,
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert!(body.get("total").is_none());
    assert_eq!(body["items"][0]["userName"], "alice");
    assert_eq!(body["items"][0]["deptName"], "部门103");
    assert_eq!(body["items"][0]["ipaddr"], TEST_PUBLIC_IP);
    assert_eq!(body["items"][0]["loginLocation"], TEST_LOGIN_LOCATION);
    assert_non_empty_string(&body["items"][0]["tokenId"]);
}

#[tokio::test]
async fn online_list_rejects_unmatched_fuzzy_filters() {
    let app = test_app();
    let tokens = sign_in(app.router.clone()).await;

    let response = app
        .router
        .oneshot(authenticated_request(
            Method::GET,
            "/api/system/online/list?ipaddr=127&userName=ali",
            &tokens.access_token,
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_eq!(body["items"], json!([]));
}

#[tokio::test]
async fn online_list_applies_server_side_limit_without_total() {
    let app = test_app();
    let tokens = sign_in(app.router.clone()).await;
    for user in 100..111 {
        let mut session = online_session_for_user(user, &format!("user-{user}"), "103");
        session.login_time = user as i64;
        app.sessions.save_session(session).await;
    }

    let response = app
        .router
        .oneshot(authenticated_request(
            Method::GET,
            "/api/system/online/list?limit=5&userName=user-",
            &tokens.access_token,
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert!(body.get("total").is_none());
    assert_eq!(body["items"].as_array().unwrap().len(), 5);
    assert_eq!(body["items"][0]["userName"], "user-110");
    assert_eq!(body["items"][4]["userName"], "user-106");
}

#[tokio::test]
async fn online_list_filters_detail_fields_and_login_time_range() {
    let app = test_app();
    let tokens = sign_in(app.router.clone()).await;
    app.sessions.save_session(detailed_online_session()).await;

    let response = app
        .router
        .oneshot(authenticated_request(
            Method::GET,
            "/api/system/online/list?loginLocation=zhou&browser=fire&os=nux&begin_time=2026-07-08&end_time=2026-07-08",
            &tokens.access_token,
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_eq!(body["items"].as_array().unwrap().len(), 1);
    assert_eq!(body["items"][0]["userName"], "bob");
    assert_eq!(body["items"][0]["loginLocation"], "Guangzhou");
    assert_eq!(body["items"][0]["browser"], "Firefox");
    assert_eq!(body["items"][0]["os"], "Linux");
    assert_eq!(body["items"][0]["loginTime"], "2026-07-08T12:00:00.000Z");
}

#[tokio::test]
async fn online_list_applies_rfc3339_login_time_boundaries_at_millisecond_precision() {
    let app = test_app();
    let tokens = sign_in(app.router.clone()).await;
    app.sessions.save_session(detailed_online_session()).await;
    let token = &tokens.access_token;
    let included = item_count(
        app.router.clone(),
        "/api/system/online/list?userName=bob&begin_time=2026-07-08T12:00:00.000Z&end_time=2026-07-08T12:00:00.000Z",
        token,
    )
    .await;
    let excluded = item_count(app.router, "/api/system/online/list?userName=bob&begin_time=2026-07-08T12:00:00.001Z", token).await;
    assert_eq!(included, 1);
    assert_eq!(excluded, 0);
}

#[tokio::test]
async fn online_list_rejects_invalid_login_time_filter() {
    let app = test_app();
    let tokens = sign_in(app.router.clone()).await;
    let response = app
        .router
        .clone()
        .oneshot(authenticated_request(
            Method::GET,
            "/api/system/online/list?begin_time=2026-99-99",
            &tokens.access_token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["code"], "invalid_input");
    assert_eq!(body["details"], "登录时间筛选格式无效，请使用YYYY-MM-DD / RFC3339");

    let response = app
        .router
        .oneshot(authenticated_request(
            Method::GET,
            "/api/system/online/list?begin_time=2026-07-08T12:00:00.001Z&end_time=2026-07-08T12:00:00.000Z",
            &tokens.access_token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["code"], "invalid_input");
    assert_eq!(body["details"], "登录时间范围无效，开始时间不能晚于结束时间");
}

#[tokio::test]
async fn online_list_rejects_a_malformed_cursor() {
    let app = test_app();
    let tokens = sign_in(app.router.clone()).await;
    let response = app
        .router
        .oneshot(authenticated_request(
            Method::GET,
            "/api/system/online/list?limit=20&cursor=broken",
            &tokens.access_token,
        ))
        .await
        .unwrap();
    let body = json_body(response).await;

    assert_eq!(body["code"], "invalid_cursor");
}

#[tokio::test]
async fn online_list_applies_self_data_scope() {
    let repository = MemoryUserRepository::with_users(vec![
        stored_user(1, "alice", "hashed:secret123"),
        stored_user(2, "bob", "hashed:secret123").with_dept_id("104"),
    ]);
    let app = test_app_with_scope(repository, self_current_user(1, "alice", "103"), self_data_scope(1, "103"));
    let tokens = sign_in(app.router.clone()).await;
    app.sessions.save_session(online_session_for_user(2, "bob", "104")).await;

    let response = app
        .router
        .oneshot(authenticated_request(Method::GET, "/api/system/online/list", &tokens.access_token))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_eq!(body["items"].as_array().unwrap().len(), 1);
    assert_eq!(body["items"][0]["userName"], "alice");
}

#[tokio::test]
async fn force_logout_rejects_online_session_outside_self_data_scope() {
    let repository = MemoryUserRepository::with_users(vec![
        stored_user(1, "alice", "hashed:secret123"),
        stored_user(2, "bob", "hashed:secret123").with_dept_id("104"),
    ]);
    let app = test_app_with_scope(repository, self_current_user(1, "alice", "103"), self_data_scope(1, "103"));
    let tokens = sign_in(app.router.clone()).await;
    let bob_session = online_session_for_user(2, "bob", "104");
    let token_id = bob_session.token_id.clone();
    app.sessions.save_session(bob_session).await;

    let response = app
        .router
        .oneshot(authenticated_request(
            Method::DELETE,
            &format!("/api/system/online/{token_id}"),
            &tokens.access_token,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(app.sessions.sessions().iter().any(|session| session.token_id == token_id));
}

fn detailed_online_session() -> crate::application::OnlineSession {
    crate::application::OnlineSession {
        token_id: "manual-token-detail".into(),
        user_id: crate::test_support::user_id(2),
        dept_id: Some("104".into()),
        dept_name: Some("部门104".into()),
        user_name: "bob".into(),
        ipaddr: "10.0.0.2".into(),
        login_location: "Guangzhou".into(),
        browser: "Firefox".into(),
        os: "Linux".into(),
        login_time: JULY_8_2026_NOON_UTC_MILLIS,
        expires_at: i64::MAX,
    }
}

fn online_session_for_user(user: u64, username: &str, dept: &str) -> crate::application::OnlineSession {
    crate::application::OnlineSession {
        token_id: format!("manual-token-{user}"),
        user_id: crate::test_support::user_id(user),
        dept_id: Some(dept.into()),
        dept_name: Some(format!("部门{dept}")),
        user_name: username.into(),
        ipaddr: TEST_PUBLIC_IP.into(),
        login_location: TEST_LOGIN_LOCATION.into(),
        browser: "Chrome".into(),
        os: "macOS".into(),
        login_time: 1,
        expires_at: i64::MAX,
    }
}

async fn item_count(router: axum::Router, uri: &str, access_token: &str) -> u64 {
    let request = authenticated_request(Method::GET, uri, access_token);
    response_json(router.oneshot(request).await.unwrap()).await["items"].as_array().unwrap().len() as u64
}
