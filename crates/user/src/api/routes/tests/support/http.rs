use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, Response, StatusCode, header},
};
use serde_json::{Value, json};
use tower::ServiceExt;

use super::{SessionTokens, VALID_PASSWORD};

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
    let body = response_json(response).await;

    SessionTokens {
        access_token: body["access_token"].as_str().unwrap().into(),
        refresh_token: body["refresh_token"].as_str().unwrap().into(),
    }
}

pub(crate) fn json_request(method: Method, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

pub(crate) fn json_request_with_accept_language(input: LocalizedJsonRequest<'_>) -> Request<Body> {
    Request::builder()
        .method(input.method)
        .uri(input.uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCEPT_LANGUAGE, input.accept_language)
        .body(Body::from(input.body.to_string()))
        .unwrap()
}

pub(crate) fn authenticated_request(method: Method, uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
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
