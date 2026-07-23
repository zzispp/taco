use std::io::Cursor;

use image::{ImageFormat, ImageReader, codecs::png::PngDecoder, codecs::webp::WebPDecoder};
use kernel::error::LocalizedError;

use super::{AppError, AppResult, AvatarFile, NormalizedAvatar};

const PNG: &str = "png";
const JPG: &str = "jpg";
const WEBP: &str = "webp";

pub async fn normalize_avatar(file: AvatarFile, max_bytes: usize) -> AppResult<NormalizedAvatar> {
    tokio::task::spawn_blocking(move || normalize_avatar_blocking(file, max_bytes))
        .await
        .map_err(|error| AppError::Infrastructure(error.to_string()))?
}

fn normalize_avatar_blocking(file: AvatarFile, max_bytes: usize) -> AppResult<NormalizedAvatar> {
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
        content_type: content_type(actual),
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

fn content_type(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Png => "image/png",
        ImageFormat::Jpeg => "image/jpeg",
        ImageFormat::WebP => "image/webp",
        _ => unreachable!("format was validated before content type selection"),
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

fn invalid_image(key: &'static str) -> AppError {
    AppError::InvalidInput(localized(key))
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

    const PNG_BYTES: &[u8] = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08,
        0x04, 0x00, 0x00, 0x00, 0xb5, 0x1c, 0x0c, 0x02, 0x00, 0x00, 0x00, 0x0b, 0x49, 0x44, 0x41, 0x54, 0x78, 0xda, 0x63, 0x64, 0xf8, 0x0f, 0x00, 0x01, 0x05,
        0x01, 0x01, 0x27, 0x18, 0xe3, 0x66, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];
    const GIF_BYTES: &[u8] = &[
        0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x01, 0x00, 0x00, 0x02, 0x01, 0x4c, 0x00, 0x3b,
    ];
    const TRAILING_PAYLOAD: &[u8] = b"untrusted-trailing-payload";

    #[tokio::test]
    async fn normalizes_image_avatar_and_discards_trailing_payload() {
        let mut bytes = PNG_BYTES.to_vec();
        bytes.extend_from_slice(TRAILING_PAYLOAD);
        let avatar = normalize_avatar(image_file("image/png", bytes), 1024).await.unwrap();
        assert_eq!(avatar.extension, "png");
        assert_eq!(avatar.content_type, "image/png");
        assert!(!avatar.bytes.ends_with(TRAILING_PAYLOAD));
    }

    #[tokio::test]
    async fn rejects_invalid_avatar_input() {
        for file in [
            image_file("image/png", vec![]),
            image_file("text/plain", vec![1]),
            image_file("image/png", b"not an image".to_vec()),
            image_file("image/jpeg", PNG_BYTES.to_vec()),
            image_file("image/gif", GIF_BYTES.to_vec()),
            image_file("image/png", animated_png()),
        ] {
            assert!(matches!(normalize_avatar(file, 1024).await, Err(AppError::InvalidInput(_))));
        }
    }

    fn image_file(content_type: &str, bytes: Vec<u8>) -> AvatarFile {
        AvatarFile {
            filename: Some("avatar.png".into()),
            content_type: Some(content_type.into()),
            bytes,
        }
    }

    fn animated_png() -> Vec<u8> {
        const IHDR_END: usize = 33;
        let data = [0, 0, 0, 1, 0, 0, 0, 0];
        let mut chunk = Vec::from([0, 0, 0, data.len() as u8]);
        chunk.extend_from_slice(b"acTL");
        chunk.extend_from_slice(&data);
        chunk.extend_from_slice(&png_crc(&chunk[4..]).to_be_bytes());
        let mut png = PNG_BYTES[..IHDR_END].to_vec();
        png.extend_from_slice(&chunk);
        png.extend_from_slice(&PNG_BYTES[IHDR_END..]);
        png
    }

    fn png_crc(bytes: &[u8]) -> u32 {
        bytes.iter().fold(u32::MAX, |crc, byte| {
            (0..8).fold(crc ^ u32::from(*byte), |value, _| {
                (value >> 1) ^ (0xedb8_8320 & (0_u32.wrapping_sub(value & 1)))
            })
        }) ^ u32::MAX
    }
}
