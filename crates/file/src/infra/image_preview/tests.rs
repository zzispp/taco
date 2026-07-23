use std::io::Cursor;

use bytes::Bytes;
use futures_util::TryStreamExt;
use image::{DynamicImage, Frame, GenericImageView, ImageFormat, Rgba, RgbaImage, codecs::gif::GifEncoder};

use super::{BoundedImagePreviewProcessor, MAX_CONCURRENT_IMAGE_TASKS, MAX_IMAGE_SOURCE_BYTES, new_image_work_limiter};
use crate::FileError;
use crate::application::{ImagePreviewProcessor, as_object_stream};
use crate::error::keys;

#[tokio::test]
async fn thumbnail_is_a_bounded_png_derivative() {
    let source = encoded_image(1024, 256, ImageFormat::Png);
    let result = BoundedImagePreviewProcessor
        .generate_thumbnail("image/png", as_object_stream(source))
        .await
        .unwrap();
    let image = image::load_from_memory_with_format(&result.bytes, ImageFormat::Png).unwrap();

    assert_eq!(result.content_type, "image/png");
    assert_eq!(image.dimensions(), (512, 128));
}

#[tokio::test]
async fn inline_validation_preserves_accepted_image_bytes() {
    let source = encoded_image(4, 2, ImageFormat::WebP);
    let body = BoundedImagePreviewProcessor
        .validate_inline("image/webp", as_object_stream(source.clone()))
        .await
        .unwrap();
    let actual = body.try_collect::<Vec<_>>().await.unwrap().concat();

    assert_eq!(actual, source.as_ref());
}

#[tokio::test]
async fn animated_gif_thumbnail_uses_the_first_frame() {
    let source = animated_gif();
    let result = BoundedImagePreviewProcessor
        .generate_thumbnail("image/gif", as_object_stream(source))
        .await
        .unwrap();
    let image = image::load_from_memory_with_format(&result.bytes, ImageFormat::Png).unwrap();

    assert_eq!(image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
}

#[tokio::test]
async fn image_dimensions_over_forty_million_pixels_are_rejected() {
    let oversized_gif = Bytes::from_static(b"GIF89a\xff\xff\xff\xff\x00\x00\x00;");
    let error = match BoundedImagePreviewProcessor.validate_inline("image/gif", as_object_stream(oversized_gif)).await {
        Ok(_) => panic!("oversized image was accepted"),
        Err(error) => error,
    };

    assert_eq!(error, FileError::Forbidden);
}

#[tokio::test]
async fn declared_and_detected_image_formats_must_match() {
    let source = encoded_image(4, 2, ImageFormat::Png);
    let error = BoundedImagePreviewProcessor
        .generate_thumbnail("image/jpeg", as_object_stream(source))
        .await
        .unwrap_err();

    assert_eq!(error, FileError::Forbidden);
}

#[tokio::test]
async fn image_source_over_thirty_two_mebibytes_is_rejected_before_decode() {
    let source = Bytes::from(vec![0_u8; MAX_IMAGE_SOURCE_BYTES as usize + 1]);
    let error = match BoundedImagePreviewProcessor.validate_inline("image/png", as_object_stream(source)).await {
        Ok(_) => panic!("oversized image source was accepted"),
        Err(error) => error,
    };

    assert_eq!(error, FileError::InvalidInput(keys::IMAGE_SOURCE_TOO_LARGE));
}

#[test]
fn image_work_limiter_allows_only_the_configured_number_of_jobs() {
    let limiter = new_image_work_limiter(MAX_CONCURRENT_IMAGE_TASKS);
    let first = limiter.clone().try_acquire_owned().unwrap();
    let second = limiter.clone().try_acquire_owned().unwrap();

    assert!(limiter.clone().try_acquire_owned().is_err());
    drop(first);
    assert!(limiter.clone().try_acquire_owned().is_ok());
    drop(second);
}

fn encoded_image(width: u32, height: u32, format: ImageFormat) -> Bytes {
    let image = DynamicImage::new_rgba8(width, height);
    let mut output = Cursor::new(Vec::new());
    image.write_to(&mut output, format).unwrap();
    Bytes::from(output.into_inner())
}

fn animated_gif() -> Bytes {
    let first = RgbaImage::from_pixel(2, 2, Rgba([255, 0, 0, 255]));
    let second = RgbaImage::from_pixel(2, 2, Rgba([0, 0, 255, 255]));
    let mut output = Vec::new();
    GifEncoder::new(&mut output).encode_frames([Frame::new(first), Frame::new(second)]).unwrap();
    Bytes::from(output)
}
