use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use serde_json::json;
use tower::ServiceExt;
use types::http::{Locale, translate_message};

use super::support::*;

const IMPORT_BOUNDARY: &str = "----taco-import-boundary";
const IMPORT_FILENAME: &str = "users.xlsx";
const IMPORT_FILE_FIELD: &str = "file";
const IMPORT_UPDATE_SUPPORT_FIELD: &str = "update_support";
const IMPORT_XLSX_CONTENT_TYPE: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
const IMPORT_SHEET_KEY: &str = "excel.user.import.sheet";
const IMPORT_HEADER_KEYS: &[&str] = &[
    "excel.user.headers.dept_id",
    "excel.user.headers.login_name",
    "excel.user.headers.password",
    "excel.user.headers.user_name",
    "excel.user.headers.email",
    "excel.user.headers.phone_number",
    "excel.user.headers.sex",
    "excel.user.headers.status",
];

#[derive(Clone, Copy)]
struct ImportRequestFixture<'a> {
    locale: &'a str,
    username: &'a str,
    update_support: bool,
    password: &'a str,
}

#[tokio::test]
async fn import_create_success_message_uses_accept_language_locale() {
    let cases = [
        ("zh-CN", "cn_import", "用户导入成功，共1条。账号cn_import导入成功"),
        ("en", "en_import", "User import succeeded, 1 records. Account en_import imported successfully"),
        ("zh-TW", "tw_import", "使用者匯入成功，共1筆。帳號tw_import匯入成功"),
    ];

    for (locale, username, expected) in cases {
        let app = test_router_with_repository(base_repository(), TestConfig::new(true));
        let response = app.oneshot(import_request(locale, username, false)).await.unwrap();
        let body = response_json(response).await;

        assert_eq!(body["success_count"], json!(1));
        assert_eq!(body["message"], json!(expected));
    }
}

#[tokio::test]
async fn import_update_success_message_uses_yaml_locale_key() {
    let repository = base_repository();
    let app = test_router_with_repository(repository.clone(), TestConfig::new(true));
    let response = app.oneshot(import_request("en", "alice", true)).await.unwrap();
    let body = response_json(response).await;

    assert_eq!(body["success_count"], json!(1));
    assert_eq!(body["message"], json!("User import succeeded, 1 records. Account alice updated successfully"));
    assert_eq!(repository.replaced_records()[0].1.password_hash.as_deref(), Some("hashed:secret123"));
}

#[tokio::test]
async fn import_blank_row_password_returns_the_column_error_without_writing() {
    let repository = base_repository();
    let app = test_router_with_repository(repository.clone(), TestConfig::new(true));

    let response = app
        .oneshot(import_request_from(ImportRequestFixture {
            locale: "en",
            username: "blank-password",
            update_support: false,
            password: "",
        }))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;

    assert_eq!(body["code"], json!("invalid_input"));
    assert_eq!(body["details"], json!("Import column Password cannot be blank"));
    assert_eq!(repository.created_records(), Vec::new());
}

#[tokio::test]
async fn import_row_validation_errors_are_localized_without_internal_keys() {
    let cases = [
        ("zh-CN", "用户导入失败：password长度必须在8到128个字符之间"),
        ("en", "User import failed: password must be between 8 and 128 characters"),
        ("zh-TW", "使用者匯入失敗：password長度必須在8到128個字元之間"),
    ];

    for (locale, expected) in cases {
        let repository = base_repository();
        let app = test_router_with_repository(repository.clone(), TestConfig::new(true));
        let response = app
            .oneshot(import_request_from(ImportRequestFixture {
                locale,
                username: "short-password",
                update_support: false,
                password: "short",
            }))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = json_body(response).await;

        assert_eq!(body["code"], json!("invalid_input"));
        assert_eq!(body["details"], json!(expected));
        assert_eq!(repository.created_records(), Vec::new());
    }
}

#[tokio::test]
async fn create_user_rejects_an_empty_password_without_writing() {
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
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(repository.created_records(), Vec::new());
}

#[tokio::test]
async fn create_user_rejects_a_missing_password_without_writing() {
    let repository = base_repository();
    let app = test_router_with_repository(repository.clone(), TestConfig::new(true));

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/system/users",
            json!({
                "username": "charlie",
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["code"], json!("invalid_json"));
    assert_eq!(body["details"], json!("JSON 请求体格式或字段类型无效"));
    assert_eq!(repository.created_records(), Vec::new());
}

#[tokio::test]
async fn replace_user_allows_an_omitted_password_without_rehashing() {
    let repository = base_repository();
    let app = test_router_with_repository(repository.clone(), TestConfig::new(true));

    let response = app
        .oneshot(json_request(
            Method::PUT,
            "/api/system/users/018f0000-0000-7000-8000-000000000001",
            json!({
                "username": "alice",
                "nick_name": "Alice Updated",
                "dept_id": "103",
                "email": "alice@example.com",
                "phonenumber": "15888888888",
                "sex": "2",
                "status": "0",
                "remark": null,
                "role_ids": ["1"],
                "post_ids": ["1"]
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let records = repository.replaced_records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].1.password_hash, None);
}

fn import_request(locale: &str, username: &str, update_support: bool) -> Request<Body> {
    import_request_from(ImportRequestFixture {
        locale,
        username,
        update_support,
        password: VALID_PASSWORD,
    })
}

fn import_request_from(input: ImportRequestFixture<'_>) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri("/api/system/users/import")
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={IMPORT_BOUNDARY}"))
        .header(header::ACCEPT_LANGUAGE, input.locale)
        .body(Body::from(import_multipart_body(input)))
        .unwrap()
}

fn import_multipart_body(input: ImportRequestFixture<'_>) -> Vec<u8> {
    let file = import_xlsx(Locale::from_header(input.locale), input.username, input.password);
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{IMPORT_BOUNDARY}\r\n").as_bytes());
    body.extend_from_slice(format!("Content-Disposition: form-data; name=\"{IMPORT_FILE_FIELD}\"; filename=\"{IMPORT_FILENAME}\"\r\n").as_bytes());
    body.extend_from_slice(format!("Content-Type: {IMPORT_XLSX_CONTENT_TYPE}\r\n\r\n").as_bytes());
    body.extend_from_slice(&file);
    body.extend_from_slice(format!("\r\n--{IMPORT_BOUNDARY}\r\n").as_bytes());
    body.extend_from_slice(format!("Content-Disposition: form-data; name=\"{IMPORT_UPDATE_SUPPORT_FIELD}\"\r\n\r\n").as_bytes());
    body.extend_from_slice(input.update_support.to_string().as_bytes());
    body.extend_from_slice(format!("\r\n--{IMPORT_BOUNDARY}--\r\n").as_bytes());
    body
}

fn import_xlsx(locale: Locale, username: &str, password: &str) -> Vec<u8> {
    kernel::excel::write_xlsx(
        &translate_message(locale, IMPORT_SHEET_KEY),
        &localized_import_headers(locale),
        &[vec![
            String::new(),
            username.into(),
            password.into(),
            username.into(),
            format!("{username}@example.com"),
            String::new(),
            "2".into(),
            "0".into(),
        ]],
    )
    .unwrap()
}

fn localized_import_headers(locale: Locale) -> Vec<String> {
    IMPORT_HEADER_KEYS.iter().map(|key| translate_message(locale, key)).collect()
}
