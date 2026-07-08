use axum::{
    body::Body,
    http::{Method, Request, header},
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
    "excel.user.headers.user_name",
    "excel.user.headers.email",
    "excel.user.headers.phone_number",
    "excel.user.headers.sex",
    "excel.user.headers.status",
];

#[cfg_attr(miri, ignore = "Miri isolation blocks rust_xlsxwriter SystemTime usage")]
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

#[cfg_attr(miri, ignore = "Miri isolation blocks rust_xlsxwriter SystemTime usage")]
#[tokio::test]
async fn import_update_success_message_uses_yaml_locale_key() {
    let app = test_router_with_repository(base_repository(), TestConfig::new(true));
    let response = app.oneshot(import_request("en", "alice", true)).await.unwrap();
    let body = response_json(response).await;

    assert_eq!(body["success_count"], json!(1));
    assert_eq!(body["message"], json!("User import succeeded, 1 records. Account alice updated successfully"));
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
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

fn import_request(locale: &str, username: &str, update_support: bool) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri("/api/system/users/import")
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={IMPORT_BOUNDARY}"))
        .header(header::ACCEPT_LANGUAGE, locale)
        .body(Body::from(import_multipart_body(locale, username, update_support)))
        .unwrap()
}

fn import_multipart_body(locale: &str, username: &str, update_support: bool) -> Vec<u8> {
    let file = import_xlsx(Locale::from_header(locale), username);
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{IMPORT_BOUNDARY}\r\n").as_bytes());
    body.extend_from_slice(format!("Content-Disposition: form-data; name=\"{IMPORT_FILE_FIELD}\"; filename=\"{IMPORT_FILENAME}\"\r\n").as_bytes());
    body.extend_from_slice(format!("Content-Type: {IMPORT_XLSX_CONTENT_TYPE}\r\n\r\n").as_bytes());
    body.extend_from_slice(&file);
    body.extend_from_slice(format!("\r\n--{IMPORT_BOUNDARY}\r\n").as_bytes());
    body.extend_from_slice(format!("Content-Disposition: form-data; name=\"{IMPORT_UPDATE_SUPPORT_FIELD}\"\r\n\r\n").as_bytes());
    body.extend_from_slice(update_support.to_string().as_bytes());
    body.extend_from_slice(format!("\r\n--{IMPORT_BOUNDARY}--\r\n").as_bytes());
    body
}

fn import_xlsx(locale: Locale, username: &str) -> Vec<u8> {
    kernel::excel::write_xlsx(
        &translate_message(locale, IMPORT_SHEET_KEY),
        &localized_import_headers(locale),
        &[vec![
            String::new(),
            username.into(),
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
