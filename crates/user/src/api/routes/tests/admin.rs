use axum::http::Method;
use serde_json::json;
use tower::ServiceExt;

use super::support::*;

#[tokio::test]
async fn create_user_uses_default_password_when_payload_password_is_empty() {
    let repository = base_repository();
    let app = test_router_with_repository(repository.clone(), TestConfig::new(true));

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/system/users",
            json!({
                "username": "charlie",
                "password": "",
                "nick_name": "Charlie",
                "dept_id": null,
                "email": "charlie@example.com",
                "phonenumber": null,
                "sex": "2",
                "status": "0",
                "remark": null,
                "role_ids": ["2"],
                "post_ids": []
            }),
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_eq!(body["username"], "charlie");
    assert_eq!(repository.created_records()[0].password_hash.as_deref(), Some("hashed:12345678"));
}
