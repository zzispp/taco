use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, Response, StatusCode, header},
};
use serde_json::{Value, json};
use tower::ServiceExt;

use super::{SessionTokens, TEST_PUBLIC_IP, VALID_PASSWORD};

const TEST_REQUEST_ID: &str = "019f5a5c-0823-7c22-acf1-add778ee83bf";
const TEST_ORIGIN: &str = "http://localhost:8082";

pub(crate) struct LocalizedJsonRequest<'a> {
    pub(crate) method: Method,
    pub(crate) uri: &'a str,
    pub(crate) body: Value,
    pub(crate) accept_language: &'a str,
}

pub(crate) async fn sign_in(app: Router) -> SessionTokens {
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
    let refresh_token = response
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(refresh_token_from_set_cookie)
        .unwrap()
        .to_owned();
    let body = response_json(response).await;

    SessionTokens {
        access_token: body["access_token"].as_str().unwrap().into(),
        refresh_token,
    }
}

pub(crate) fn refresh_cookie_request(method: Method, uri: &str, refresh_token: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::COOKIE, format!("refresh_token={refresh_token}"))
        .header(header::ORIGIN, TEST_ORIGIN)
        .header("x-forwarded-for", TEST_PUBLIC_IP)
        .header("x-request-id", TEST_REQUEST_ID)
        .body(Body::empty())
        .unwrap()
}

pub(crate) fn json_request(method: Method, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-forwarded-for", TEST_PUBLIC_IP)
        .header("x-request-id", TEST_REQUEST_ID)
        .body(Body::from(body.to_string()))
        .unwrap()
}

pub(crate) fn json_request_with_accept_language(input: LocalizedJsonRequest<'_>) -> Request<Body> {
    Request::builder()
        .method(input.method)
        .uri(input.uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCEPT_LANGUAGE, input.accept_language)
        .header("x-forwarded-for", TEST_PUBLIC_IP)
        .header("x-request-id", TEST_REQUEST_ID)
        .body(Body::from(input.body.to_string()))
        .unwrap()
}

pub(crate) fn authenticated_request(method: Method, uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header("x-forwarded-for", TEST_PUBLIC_IP)
        .header("x-request-id", TEST_REQUEST_ID)
        .body(Body::empty())
        .unwrap()
}

pub(crate) async fn response_json(response: Response<Body>) -> Value {
    assert_eq!(response.status(), StatusCode::OK);
    json_body(response).await
}

pub(crate) async fn json_body(response: Response<Body>) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

pub(crate) fn assert_non_empty_string(value: &Value) {
    assert!(!value.as_str().unwrap().is_empty());
}

fn refresh_token_from_set_cookie(value: &str) -> Option<&str> {
    value.split(';').next()?.strip_prefix("refresh_token=")
}
