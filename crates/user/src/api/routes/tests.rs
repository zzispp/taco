use std::sync::Arc;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, Response, StatusCode, header},
};
use serde_json::{Value, json};
use tower::ServiceExt;

use super::create_router;
use crate::{
    api::{ApiState, TokenService, TokenSettings},
    application::UserService,
    test_support::{MemoryUserRepository, TestPasswordHasher, VALID_PASSWORD, stored_user},
};

const TEST_SECRET: &str = "test-secret-with-enough-entropy";
const ACCESS_TTL_SECONDS: u64 = 900;
const REFRESH_TTL_SECONDS: u64 = 604800;

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

    assert_eq!(body["user"]["role"], "user");
    assert_eq!(body["user"]["is_active"], true);
    assert_eq!(body["user"]["auth_source"], "local");
    assert_eq!(body["user"]["email_verified"], false);
    assert_non_empty_string(&body["access_token"]);
}

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
    assert_eq!(body["message"], "invalid JSON payload");
    assert!(body["details"].as_str().is_some());
}

struct SessionTokens {
    access_token: String,
    refresh_token: String,
}

fn test_router() -> Router {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let users = UserService::new(repository, TestPasswordHasher);
    Router::new().nest("/api", create_router(ApiState::new(Arc::new(users), token_service())))
}

fn token_service() -> TokenService {
    TokenService::new(TokenSettings {
        secret: TEST_SECRET.into(),
        access_token_ttl_seconds: ACCESS_TTL_SECONDS,
        refresh_token_ttl_seconds: REFRESH_TTL_SECONDS,
    })
}

async fn sign_in(app: Router) -> SessionTokens {
    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/auth/sign-in",
            json!({
                "identifier": "alice",
                "password": VALID_PASSWORD
            }),
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    SessionTokens {
        access_token: body["access_token"].as_str().unwrap().into(),
        refresh_token: body["refresh_token"].as_str().unwrap().into(),
    }
}

fn json_request(method: Method, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn authenticated_request(method: Method, uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

async fn response_json(response: Response<Body>) -> Value {
    assert_eq!(response.status(), StatusCode::OK);
    json_body(response).await
}

async fn json_body(response: Response<Body>) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

fn assert_non_empty_string(value: &Value) {
    assert!(!value.as_str().unwrap().is_empty());
}
