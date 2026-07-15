use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;

use super::support::*;

const USER_ID: &str = "018f0000-0000-7000-8000-000000000001";

#[tokio::test]
async fn create_user_accepts_an_explicit_empty_role_list() {
    let repository = base_repository();
    let app = test_router_with_repository(repository.clone(), TestConfig::new(true));

    let response = app
        .oneshot(json_request(Method::POST, "/api/system/users", user_payload("charlie", true, true)))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(repository.created_records()[0].role_ids, Vec::<String>::new());
}

#[tokio::test]
async fn replace_user_accepts_an_explicit_empty_role_list() {
    let repository = base_repository();
    let app = test_router_with_repository(repository.clone(), TestConfig::new(true));

    let response = app
        .oneshot(json_request(
            Method::PUT,
            &format!("/api/system/users/{USER_ID}"),
            user_payload("alice", false, true),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(repository.replaced_records()[0].1.role_ids, Vec::<String>::new());
}

#[tokio::test]
async fn independent_role_assignment_accepts_an_explicit_empty_list() {
    let app = test_app();
    let response = app
        .router
        .oneshot(json_request(
            Method::PUT,
            &format!("/api/system/users/{USER_ID}/roles"),
            json!({ "role_ids": [] }),
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_eq!(body["role_ids"], json!([]));
}

#[tokio::test]
async fn role_lists_must_be_explicit_on_all_three_write_routes() {
    for (method, uri, payload) in missing_role_requests() {
        let response = test_router_with_repository(base_repository(), TestConfig::new(true))
            .oneshot(json_request(method, &uri, payload))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{uri}");
        let body = json_body(response).await;
        assert_eq!(body["code"], json!("invalid_json"), "{uri}");
    }
}

fn missing_role_requests() -> [(Method, String, Value); 3] {
    [
        (Method::POST, "/api/system/users".into(), user_payload("charlie", true, false)),
        (Method::PUT, format!("/api/system/users/{USER_ID}"), user_payload("alice", false, false)),
        (Method::PUT, format!("/api/system/users/{USER_ID}/roles"), json!({})),
    ]
}

fn user_payload(username: &str, include_password: bool, include_roles: bool) -> Value {
    let mut payload = json!({
        "username": username,
        "nick_name": username,
        "dept_id": null,
        "email": format!("{username}@example.com"),
        "phonenumber": null,
        "sex": "2",
        "status": "0",
        "remark": null,
        "post_ids": []
    });
    if include_password {
        payload["password"] = json!(VALID_PASSWORD);
    }
    if include_roles {
        payload["role_ids"] = json!([]);
    }
    payload
}
