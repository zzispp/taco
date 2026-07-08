use axum::http::{Method, StatusCode};
use serde_json::json;
use tower::ServiceExt;

use super::support::*;
use crate::test_support::{MemoryUserRepository, stored_user};

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
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

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn online_list_returns_aligned_rows_and_filters_exactly() {
    let app = test_app();
    let tokens = sign_in(app.router.clone()).await;

    let response = app
        .router
        .oneshot(authenticated_request(
            Method::GET,
            "/api/system/online/list?ipaddr=8.8.8.8&userName=alice",
            &tokens.access_token,
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_eq!(body["total"], 1);
    assert_eq!(body["rows"][0]["userName"], "alice");
    assert_eq!(body["rows"][0]["deptName"], "部门103");
    assert_eq!(body["rows"][0]["ipaddr"], TEST_PUBLIC_IP);
    assert_eq!(body["rows"][0]["loginLocation"], TEST_LOGIN_LOCATION);
    assert_non_empty_string(&body["rows"][0]["tokenId"]);
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn online_list_filters_username_and_ip_as_exact_matches() {
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

    assert_eq!(body["total"], 0);
    assert_eq!(body["rows"], json!([]));
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn force_logout_invalidates_access_and_refresh_tokens() {
    let app = test_app();
    let tokens = sign_in(app.router.clone()).await;
    let token_id = app.sessions.sessions()[0].token_id.clone();

    let response = app
        .router
        .clone()
        .oneshot(authenticated_request(
            Method::DELETE,
            &format!("/api/system/online/{token_id}"),
            &tokens.access_token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .router
        .clone()
        .oneshot(authenticated_request(Method::GET, "/api/auth/me", &tokens.access_token))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let response = app
        .router
        .oneshot(json_request(
            Method::POST,
            "/api/auth/refresh",
            json!({ "refresh_token": tokens.refresh_token }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn logout_deletes_current_online_session() {
    let app = test_app();
    let tokens = sign_in(app.router.clone()).await;

    let response = app
        .router
        .oneshot(authenticated_request(Method::POST, "/api/auth/logout", &tokens.access_token))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(app.sessions.sessions().is_empty());
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
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

    assert_eq!(body["total"], 1);
    assert_eq!(body["rows"][0]["userName"], "alice");
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
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

fn online_session_for_user(user: u64, username: &str, dept: &str) -> crate::application::OnlineSession {
    crate::application::OnlineSession {
        token_id: format!("manual-token-{user}"),
        user_id: crate::test_support::user_id(user),
        dept_name: Some(format!("部门{dept}")),
        user_name: username.into(),
        ipaddr: TEST_PUBLIC_IP.into(),
        login_location: TEST_LOGIN_LOCATION.into(),
        browser: "Chrome".into(),
        os: "macOS".into(),
        login_time: 1,
    }
}
