use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use tower::ServiceExt;

use super::support::*;
use crate::{
    application::{AppResult, AvatarOwner, AvatarStorage, NormalizedAvatar, UserRepository},
    domain::{AvatarFileId, UserId},
    test_support::{MemoryUserRepository, stored_user},
};

const OLD_AVATAR_ID: &str = "018f0000-0000-7000-8000-000000000010";
const NEW_AVATAR_ID: &str = "018f0000-0000-7000-8000-000000000011";
const AVATAR_BOUNDARY: &str = "taco-avatar-boundary";
const PNG_BYTES: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x04,
    0x00, 0x00, 0x00, 0xb5, 0x1c, 0x0c, 0x02, 0x00, 0x00, 0x00, 0x0b, 0x49, 0x44, 0x41, 0x54, 0x78, 0xda, 0x63, 0x64, 0xf8, 0x0f, 0x00, 0x01, 0x05, 0x01, 0x01,
    0x27, 0x18, 0xe3, 0x66, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];

#[tokio::test]
async fn uploaded_avatar_binding_updates_the_profile_reference() {
    let repository = avatar_repository();
    let storage = Arc::new(RecordingAvatarStorage::default());
    let app = test_app_with_avatar_storage(repository.clone(), storage.clone());

    let response = app.router.oneshot(avatar_upload_request()).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["img_url"], "/api/avatars/018f0000-0000-7000-8000-000000000001/2");
    assert_avatar(&repository, NEW_AVATAR_ID, 2).await;
    assert_eq!(storage.trashed_ids(), Vec::<String>::new());
}

#[tokio::test]
async fn failed_uploaded_avatar_binding_trashes_the_new_asset() {
    let repository = avatar_repository();
    repository.fail_audit_with("avatar binding failed");
    let storage = Arc::new(RecordingAvatarStorage::default());
    let app = test_app_with_avatar_storage(repository.clone(), storage.clone());

    let response = app.router.oneshot(avatar_upload_request()).await.unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_avatar(&repository, OLD_AVATAR_ID, 1).await;
    assert_eq!(storage.trashed_ids(), vec![NEW_AVATAR_ID.to_owned()]);
}

async fn assert_avatar(repository: &MemoryUserRepository, expected_id: &str, expected_version: u64) {
    let user = repository
        .find_by_id(UserId("018f0000-0000-7000-8000-000000000001".into()))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user.avatar_file_id.as_ref().map(AvatarFileId::as_str), Some(expected_id));
    assert_eq!(user.avatar_version, expected_version);
}

fn avatar_repository() -> MemoryUserRepository {
    MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123").with_avatar_file_id(OLD_AVATAR_ID))
}

fn avatar_upload_request() -> Request<Body> {
    let mut body =
        format!("--{AVATAR_BOUNDARY}\r\nContent-Disposition: form-data; name=\"avatarfile\"; filename=\"avatar.png\"\r\nContent-Type: image/png\r\n\r\n")
            .into_bytes();
    body.extend_from_slice(PNG_BYTES);
    body.extend_from_slice(format!("\r\n--{AVATAR_BOUNDARY}--\r\n").as_bytes());
    Request::builder()
        .method(Method::POST)
        .uri("/api/account/profile/avatar")
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={AVATAR_BOUNDARY}"))
        .body(Body::from(body))
        .unwrap()
}

#[derive(Clone, Default)]
struct RecordingAvatarStorage {
    trashed: Arc<Mutex<Vec<AvatarFileId>>>,
}

impl RecordingAvatarStorage {
    fn trashed_ids(&self) -> Vec<String> {
        self.trashed.lock().unwrap().iter().map(ToString::to_string).collect()
    }
}

#[async_trait]
impl AvatarStorage for RecordingAvatarStorage {
    async fn store_avatar(&self, _owner: AvatarOwner, _avatar: NormalizedAvatar) -> AppResult<AvatarFileId> {
        Ok(AvatarFileId::new(NEW_AVATAR_ID).unwrap())
    }

    async fn trash_avatar(&self, _owner: AvatarOwner, file_id: AvatarFileId) -> AppResult<()> {
        self.trashed.lock().unwrap().push(file_id);
        Ok(())
    }
}
