use std::pin::Pin;

use async_trait::async_trait;
use bytes::Bytes;
use futures_util::Stream;

use super::AppResult;
use crate::domain::AvatarFileId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvatarFile {
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvatarOwner {
    pub user_id: String,
    pub department_id: Option<String>,
}

pub struct NormalizedAvatar {
    pub extension: &'static str,
    pub content_type: &'static str,
    pub bytes: Vec<u8>,
}

pub type AvatarProjectionBody = Pin<Box<dyn Stream<Item = AppResult<Bytes>> + Send + 'static>>;

pub struct AvatarProjection {
    pub content_type: String,
    pub content_length: u64,
    pub body: AvatarProjectionBody,
}

/// Stores normalized avatar assets in the configured asset-management implementation.
#[async_trait]
pub trait AvatarStorage: Send + Sync + 'static {
    async fn store_avatar(&self, owner: AvatarOwner, avatar: NormalizedAvatar) -> AppResult<AvatarFileId>;
    async fn trash_avatar(&self, owner: AvatarOwner, file_id: AvatarFileId) -> AppResult<()>;
}

/// Reads a user's public avatar projection without exposing file-management types to user APIs.
#[async_trait]
pub trait AvatarProjectionStorage: Send + Sync + 'static {
    async fn load_avatar_projection(&self, owner: AvatarOwner, file_id: AvatarFileId) -> AppResult<AvatarProjection>;
}
