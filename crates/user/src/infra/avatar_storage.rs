use std::{io::Cursor, path::PathBuf};

use async_trait::async_trait;
use image::{ImageFormat, ImageReader, codecs::png::PngDecoder, codecs::webp::WebPDecoder};
use kernel::error::LocalizedError;
use tokio::fs;
use uuid::Uuid;

use crate::application::{AppError, AppResult, AvatarFile, AvatarStorage};

const PNG: &str = "png";
const JPG: &str = "jpg";
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
        let avatar = tokio::task::spawn_blocking(move || normalize_avatar(file, max_bytes))
            .await
            .map_err(|error| AppError::Infrastructure(error.to_string()))??;
        fs::create_dir_all(&self.directory).await.map_err(storage_error)?;
        let file_name = format!("{}.{}", Uuid::now_v7(), avatar.extension);
        let path = self.directory.join(&file_name);
        fs::write(path, avatar.bytes).await.map_err(storage_error)?;
        Ok(format!("{}/{}", self.url_prefix.trim_end_matches('/'), file_name))
    }
}

struct NormalizedAvatar {
    extension: &'static str,
    bytes: Vec<u8>,
}

fn normalize_avatar(file: AvatarFile, max_bytes: usize) -> AppResult<NormalizedAvatar> {
    validate_size(&file.bytes, max_bytes)?;
    let declared = declared_format(&file)?;
    let actual = actual_format(&file.bytes)?;
    if declared != actual {
        return Err(invalid_image("errors.user.avatar_content_type_mismatch"));
    }

    validate_static(actual, &file.bytes)?;
    let image = ImageReader::with_format(Cursor::new(&file.bytes), actual)
        .decode()
        .map_err(|_| invalid_image("errors.user.avatar_must_be_image"))?;
    let mut bytes = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), actual)
        .map_err(|error| AppError::Infrastructure(error.to_string()))?;
    Ok(NormalizedAvatar {
        extension: extension(actual),
        bytes,
    })
}

fn declared_format(file: &AvatarFile) -> AppResult<ImageFormat> {
    if let Some(content_type) = file.content_type.as_deref() {
        return format_from_content_type(content_type);
    }
    let Some(filename) = file.filename.as_deref() else {
        return Err(AppError::InvalidInput(localized("errors.user.avatar_content_type_required")));
    };
    match filename.rsplit('.').next().map(str::to_ascii_lowercase).as_deref() {
        Some("png") => Ok(ImageFormat::Png),
        Some("jpg") | Some("jpeg") => Ok(ImageFormat::Jpeg),
        Some("webp") => Ok(ImageFormat::WebP),
        _ => Err(AppError::InvalidInput(localized("errors.user.avatar_must_be_image"))),
    }
}

fn format_from_content_type(content_type: &str) -> AppResult<ImageFormat> {
    match content_type.trim().to_ascii_lowercase().as_str() {
        "image/png" => Ok(ImageFormat::Png),
        "image/jpeg" | "image/jpg" => Ok(ImageFormat::Jpeg),
        "image/webp" => Ok(ImageFormat::WebP),
        _ => Err(AppError::InvalidInput(localized_param(
            "errors.user.avatar_unsupported_content_type",
            "content_type",
            content_type.trim(),
        ))),
    }
}

fn actual_format(bytes: &[u8]) -> AppResult<ImageFormat> {
    match image::guess_format(bytes) {
        Ok(format @ (ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP)) => Ok(format),
        _ => Err(invalid_image("errors.user.avatar_must_be_image")),
    }
}

fn validate_static(format: ImageFormat, bytes: &[u8]) -> AppResult<()> {
    let animated = match format {
        ImageFormat::Png => PngDecoder::new(Cursor::new(bytes))
            .and_then(|decoder| decoder.is_apng())
            .map_err(|_| invalid_image("errors.user.avatar_must_be_image"))?,
        ImageFormat::WebP => WebPDecoder::new(Cursor::new(bytes))
            .map_err(|_| invalid_image("errors.user.avatar_must_be_image"))?
            .has_animation(),
        ImageFormat::Jpeg => false,
        _ => return Err(invalid_image("errors.user.avatar_must_be_image")),
    };
    if animated {
        return Err(invalid_image("errors.user.avatar_animation_not_supported"));
    }
    Ok(())
}

fn extension(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Png => PNG,
        ImageFormat::Jpeg => JPG,
        ImageFormat::WebP => WEBP,
        _ => unreachable!("format was validated before extension selection"),
    }
}

fn invalid_image(key: &'static str) -> AppError {
    AppError::InvalidInput(localized(key))
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
mod tests;
