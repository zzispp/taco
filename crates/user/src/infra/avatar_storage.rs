use std::path::PathBuf;

use async_trait::async_trait;
use kernel::error::LocalizedError;
use tokio::fs;
use uuid::Uuid;

use crate::application::{AppError, AppResult, AvatarFile, AvatarStorage};

const PNG: &str = "png";
const JPG: &str = "jpg";
const GIF: &str = "gif";
const WEBP: &str = "webp";

#[derive(Clone)]
pub struct LocalAvatarStorage {
    directory: PathBuf,
    url_prefix: String,
}

impl LocalAvatarStorage {
    pub fn new(directory: impl Into<PathBuf>, url_prefix: impl Into<String>) -> Self {
        Self {
            directory: directory.into(),
            url_prefix: url_prefix.into(),
        }
    }
}

#[async_trait]
impl AvatarStorage for LocalAvatarStorage {
    async fn store_avatar(&self, file: AvatarFile, max_bytes: usize) -> AppResult<String> {
        let extension = image_extension(&file)?;
        validate_size(&file.bytes, max_bytes)?;
        fs::create_dir_all(&self.directory).await.map_err(storage_error)?;
        let file_name = format!("{}.{}", Uuid::now_v7(), extension);
        let path = self.directory.join(&file_name);
        fs::write(path, file.bytes).await.map_err(storage_error)?;
        Ok(format!("{}/{}", self.url_prefix.trim_end_matches('/'), file_name))
    }
}

fn image_extension(file: &AvatarFile) -> AppResult<&'static str> {
    match file.content_type.as_deref().map(str::trim) {
        Some("image/png") => Ok(PNG),
        Some("image/jpeg") | Some("image/jpg") => Ok(JPG),
        Some("image/gif") => Ok(GIF),
        Some("image/webp") => Ok(WEBP),
        Some(value) => Err(AppError::InvalidInput(localized_param(
            "errors.user.avatar_unsupported_content_type",
            "content_type",
            value,
        ))),
        None => extension_from_name(file.filename.as_deref()),
    }
}

fn extension_from_name(filename: Option<&str>) -> AppResult<&'static str> {
    let Some(filename) = filename else {
        return Err(AppError::InvalidInput(localized("errors.user.avatar_content_type_required")));
    };
    match filename.rsplit('.').next().map(str::to_ascii_lowercase).as_deref() {
        Some("png") => Ok(PNG),
        Some("jpg") | Some("jpeg") => Ok(JPG),
        Some("gif") => Ok(GIF),
        Some("webp") => Ok(WEBP),
        _ => Err(AppError::InvalidInput(localized("errors.user.avatar_must_be_image"))),
    }
}

fn validate_size(bytes: &[u8], max_bytes: usize) -> AppResult<()> {
    if bytes.is_empty() {
        return Err(AppError::InvalidInput(localized("errors.user.avatar_empty_file")));
    }
    if bytes.len() > max_bytes {
        return Err(AppError::InvalidInput(localized_param(
            "errors.user.avatar_max_bytes",
            "max_bytes",
            max_bytes.to_string(),
        )));
    }
    Ok(())
}

fn storage_error(error: std::io::Error) -> AppError {
    AppError::Infrastructure(error.to_string())
}

fn localized(key: &'static str) -> LocalizedError {
    LocalizedError::new(key)
}

fn localized_param(key: &'static str, param: &'static str, value: impl Into<String>) -> LocalizedError {
    LocalizedError::new(key).with_param(param, value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn stores_image_avatar_and_returns_url() {
        let dir = tempfile::tempdir().unwrap();
        let storage = LocalAvatarStorage::new(dir.path(), "/uploads/avatars");

        let url = storage.store_avatar(image_file("image/png", vec![1, 2, 3]), 1024).await.unwrap();

        assert!(url.starts_with("/uploads/avatars/"));
        assert!(url.ends_with(".png"));
    }

    #[tokio::test]
    async fn rejects_empty_avatar() {
        let dir = tempfile::tempdir().unwrap();
        let storage = LocalAvatarStorage::new(dir.path(), "/uploads/avatars");

        let result = storage.store_avatar(image_file("image/png", vec![]), 1024).await;

        assert!(matches!(result, Err(AppError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn rejects_non_image_avatar() {
        let dir = tempfile::tempdir().unwrap();
        let storage = LocalAvatarStorage::new(dir.path(), "/uploads/avatars");

        let result = storage.store_avatar(image_file("text/plain", vec![1]), 1024).await;

        assert!(matches!(result, Err(AppError::InvalidInput(_))));
    }

    fn image_file(content_type: &str, bytes: Vec<u8>) -> AvatarFile {
        AvatarFile {
            filename: Some("avatar.png".into()),
            content_type: Some(content_type.into()),
            bytes,
        }
    }
}
