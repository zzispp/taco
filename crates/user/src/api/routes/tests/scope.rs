use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use serde_json::{Value, json};
use tower::ServiceExt;

use super::support::*;
use crate::test_support::{MemoryUserRepository, stored_user};

const BOB_ID: &str = "018f0000-0000-7000-8000-000000000002";

#[tokio::test]
async fn self_scope_rejects_out_of_scope_user_object_operations() {
    let app = self_scope_app().router;

    for request in out_of_scope_requests() {
        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}

#[tokio::test]
async fn admin_permission_does_not_bypass_self_data_scope_user_object() {
    let repository = user_scope_repository();
    let app = test_app_with_scope(repository.clone(), admin_current_user(), self_data_scope(1, "103"));

    let response = app
        .router
        .oneshot(empty_request(Method::DELETE, "/api/system/users/018f0000-0000-7000-8000-000000000002"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(repository.deleted_records(), Vec::new());
}

fn self_scope_app() -> TestApp {
    test_app_with_scope(user_scope_repository(), self_current_user(1, "alice", "103"), self_data_scope(1, "103"))
}

fn user_scope_repository() -> MemoryUserRepository {
    MemoryUserRepository::with_users(vec![
        stored_user(1, "alice", "hashed:secret123"),
        stored_user(2, "bob", "hashed:secret123").with_dept_id("104"),
    ])
}

fn out_of_scope_requests() -> Vec<Request<Body>> {
    vec![
        empty_request(Method::GET, "/api/system/users/018f0000-0000-7000-8000-000000000002"),
        json_request(Method::PUT, "/api/system/users/018f0000-0000-7000-8000-000000000002", user_payload("bob_edit")),
        empty_request(Method::DELETE, "/api/system/users/018f0000-0000-7000-8000-000000000002"),
        json_request(Method::DELETE, "/api/system/users/batch", json!({ "ids": [BOB_ID] })),
        json_request(
            Method::PUT,
            "/api/system/users/018f0000-0000-7000-8000-000000000002/password",
            json!({ "password": "Secret123!" }),
        ),
        json_request(
            Method::PUT,
            "/api/system/users/018f0000-0000-7000-8000-000000000002/status",
            json!({ "status": "1" }),
        ),
        empty_request(Method::GET, "/api/system/users/018f0000-0000-7000-8000-000000000002/roles"),
        json_request(
            Method::PUT,
            "/api/system/users/018f0000-0000-7000-8000-000000000002/roles",
            json!({ "role_ids": ["1"] }),
        ),
    ]
}

fn empty_request(method: Method, uri: &str) -> Request<Body> {
    Request::builder().method(method).uri(uri).body(Body::empty()).unwrap()
}

fn user_payload(username: &str) -> Value {
    json!({
        "username": username,
        "password": null,
        "nick_name": username,
        "dept_id": "104",
        "email": format!("{username}@example.com"),
        "phonenumber": null,
        "sex": "2",
        "status": "0",
        "remark": null,
        "role_ids": ["1"],
        "post_ids": []
    })
}
