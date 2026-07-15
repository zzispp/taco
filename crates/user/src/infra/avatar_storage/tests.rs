use super::*;

const PNG_BYTES: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x04,
    0x00, 0x00, 0x00, 0xb5, 0x1c, 0x0c, 0x02, 0x00, 0x00, 0x00, 0x0b, 0x49, 0x44, 0x41, 0x54, 0x78, 0xda, 0x63, 0x64, 0xf8, 0x0f, 0x00, 0x01, 0x05, 0x01, 0x01,
    0x27, 0x18, 0xe3, 0x66, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];
const GIF_BYTES: &[u8] = &[
    0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00,
    0x01, 0x00, 0x00, 0x02, 0x01, 0x4c, 0x00, 0x3b,
];
const TRAILING_PAYLOAD: &[u8] = b"untrusted-trailing-payload";

#[tokio::test]
async fn stores_image_avatar_and_returns_url() {
    let dir = tempfile::tempdir().unwrap();
    let storage = LocalAvatarStorage::new(dir.path(), "/uploads/avatars");
    let mut bytes = PNG_BYTES.to_vec();
    bytes.extend_from_slice(TRAILING_PAYLOAD);

    let url = storage.store_avatar(image_file("image/png", bytes), 1024).await.unwrap();
    let stored = fs::read(dir.path().join(url.rsplit('/').next().unwrap())).await.unwrap();

    assert!(url.starts_with("/uploads/avatars/"));
    assert!(url.ends_with(".png"));
    assert!(!stored.ends_with(TRAILING_PAYLOAD));
}

#[tokio::test]
async fn rejects_empty_avatar() {
    assert_invalid(image_file("image/png", vec![])).await;
}

#[tokio::test]
async fn rejects_non_image_avatar() {
    assert_invalid(image_file("text/plain", vec![1])).await;
}

#[tokio::test]
async fn rejects_spoofed_png_content_type() {
    assert_invalid(image_file("image/png", b"not an image".to_vec())).await;
}

#[tokio::test]
async fn rejects_mismatched_declared_and_actual_format() {
    assert_invalid(image_file("image/jpeg", PNG_BYTES.to_vec())).await;
}

#[tokio::test]
async fn rejects_gif_avatar() {
    assert_invalid(image_file("image/gif", GIF_BYTES.to_vec())).await;
}

#[tokio::test]
async fn rejects_animated_png_avatar() {
    assert_invalid(image_file("image/png", animated_png())).await;
}

async fn assert_invalid(file: AvatarFile) {
    let dir = tempfile::tempdir().unwrap();
    let storage = LocalAvatarStorage::new(dir.path(), "/uploads/avatars");

    let result = storage.store_avatar(file, 1024).await;

    assert!(matches!(result, Err(AppError::InvalidInput(_))));
    assert!(fs::read_dir(dir.path()).await.unwrap().next_entry().await.unwrap().is_none());
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
