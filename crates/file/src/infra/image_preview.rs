use std::{
    io::{BufReader, Cursor, Seek, SeekFrom},
    sync::{Arc, OnceLock},
};

use async_trait::async_trait;
use bytes::Bytes;
use futures_util::StreamExt;
use image::{ImageFormat, ImageReader};
use tokio::{
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    sync::{OwnedSemaphorePermit, Semaphore},
};

use crate::application::{GeneratedThumbnail, ImagePreviewProcessor, ObjectStream};
use crate::error::keys;
use crate::{FileError, FileResult};

const MAX_DECODED_PIXELS: u64 = 40_000_000;
const MAX_IMAGE_SOURCE_BYTES: u64 = 32 * 1024 * 1024;
const MAX_CONCURRENT_IMAGE_TASKS: usize = 2;
const THUMBNAIL_MAX_EDGE_PIXELS: u32 = 512;
const STREAM_BUFFER_BYTES: usize = 64 * 1024;
const THUMBNAIL_CONTENT_TYPE: &str = "image/png";

static IMAGE_WORK_LIMITER: OnceLock<Arc<Semaphore>> = OnceLock::new();

#[derive(Clone, Copy, Debug, Default)]
pub struct BoundedImagePreviewProcessor;

#[async_trait]
impl ImagePreviewProcessor for BoundedImagePreviewProcessor {
    async fn validate_inline(&self, content_type: &str, body: ObjectStream) -> FileResult<ObjectStream> {
        let _permit = acquire_image_work().await?;
        let file = stage_source(body).await?;
        let content_type = content_type.to_owned();
        let file = tokio::task::spawn_blocking(move || validate_inline_file(file, &content_type))
            .await
            .map_err(|_| infrastructure("join inline image validation"))??;
        Ok(file_stream(file))
    }

    async fn generate_thumbnail(&self, content_type: &str, body: ObjectStream) -> FileResult<GeneratedThumbnail> {
        let _permit = acquire_image_work().await?;
        let file = stage_source(body).await?;
        let content_type = content_type.to_owned();
        tokio::task::spawn_blocking(move || generate_thumbnail(file, &content_type))
            .await
            .map_err(|_| infrastructure("join thumbnail generation"))?
    }
}

async fn acquire_image_work() -> FileResult<OwnedSemaphorePermit> {
    image_work_limiter()
        .acquire_owned()
        .await
        .map_err(|_| infrastructure("acquire image processing capacity"))
}

fn image_work_limiter() -> Arc<Semaphore> {
    IMAGE_WORK_LIMITER.get_or_init(|| new_image_work_limiter(MAX_CONCURRENT_IMAGE_TASKS)).clone()
}

fn new_image_work_limiter(permits: usize) -> Arc<Semaphore> {
    Arc::new(Semaphore::new(permits))
}

async fn stage_source(mut body: ObjectStream) -> FileResult<std::fs::File> {
    let file = tempfile::tempfile().map_err(|_| infrastructure("create image staging file"))?;
    let mut file = tokio::fs::File::from_std(file);
    let mut source_bytes = 0_u64;
    while let Some(chunk) = body.next().await {
        let chunk = chunk?;
        source_bytes = source_bytes
            .checked_add(u64::try_from(chunk.len()).map_err(|_| FileError::InvalidInput(keys::IMAGE_SOURCE_TOO_LARGE))?)
            .ok_or(FileError::InvalidInput(keys::IMAGE_SOURCE_TOO_LARGE))?;
        if source_bytes > MAX_IMAGE_SOURCE_BYTES {
            return Err(FileError::InvalidInput(keys::IMAGE_SOURCE_TOO_LARGE));
        }
        file.write_all(&chunk).await.map_err(|_| infrastructure("write image staging file"))?;
    }
    file.flush().await.map_err(|_| infrastructure("flush image staging file"))?;
    file.seek(SeekFrom::Start(0)).await.map_err(|_| infrastructure("rewind image staging file"))?;
    Ok(file.into_std().await)
}

fn validate_inline_file(mut file: std::fs::File, content_type: &str) -> FileResult<std::fs::File> {
    validate_dimensions(&file, content_type)?;
    file.seek(SeekFrom::Start(0)).map_err(|_| infrastructure("rewind validated image"))?;
    Ok(file)
}

fn generate_thumbnail(file: std::fs::File, content_type: &str) -> FileResult<GeneratedThumbnail> {
    validate_dimensions(&file, content_type)?;
    let image = source_reader(&file, content_type)?.decode().map_err(|_| FileError::Forbidden)?;
    let thumbnail = image.thumbnail(THUMBNAIL_MAX_EDGE_PIXELS, THUMBNAIL_MAX_EDGE_PIXELS);
    let mut output = Cursor::new(Vec::new());
    thumbnail
        .write_to(&mut output, ImageFormat::Png)
        .map_err(|_| infrastructure("encode image thumbnail"))?;
    Ok(GeneratedThumbnail {
        content_type: THUMBNAIL_CONTENT_TYPE,
        bytes: Bytes::from(output.into_inner()),
    })
}

fn validate_dimensions(file: &std::fs::File, content_type: &str) -> FileResult<()> {
    let (width, height) = source_reader(file, content_type)?.into_dimensions().map_err(|_| FileError::Forbidden)?;
    let pixels = u64::from(width).checked_mul(u64::from(height)).ok_or(FileError::Forbidden)?;
    if pixels == 0 || pixels > MAX_DECODED_PIXELS {
        return Err(FileError::Forbidden);
    }
    Ok(())
}

fn source_reader(file: &std::fs::File, content_type: &str) -> FileResult<ImageReader<BufReader<std::fs::File>>> {
    let expected = image_format(content_type).ok_or(FileError::Forbidden)?;
    let mut source = file.try_clone().map_err(|_| infrastructure("clone image staging file"))?;
    source.seek(SeekFrom::Start(0)).map_err(|_| infrastructure("rewind image source"))?;
    let reader = ImageReader::new(BufReader::new(source))
        .with_guessed_format()
        .map_err(|_| infrastructure("detect image format"))?;
    if reader.format() != Some(expected) {
        return Err(FileError::Forbidden);
    }
    Ok(reader)
}

fn image_format(content_type: &str) -> Option<ImageFormat> {
    match content_type.split(';').next()?.trim().to_ascii_lowercase().as_str() {
        "image/png" => Some(ImageFormat::Png),
        "image/jpeg" => Some(ImageFormat::Jpeg),
        "image/webp" => Some(ImageFormat::WebP),
        "image/gif" => Some(ImageFormat::Gif),
        _ => None,
    }
}

fn file_stream(file: std::fs::File) -> ObjectStream {
    let file = tokio::fs::File::from_std(file);
    Box::pin(futures_util::stream::unfold(Some(file), |state| async move {
        let mut file = state?;
        let mut buffer = vec![0_u8; STREAM_BUFFER_BYTES];
        match file.read(&mut buffer).await {
            Ok(0) => None,
            Ok(read) => {
                buffer.truncate(read);
                Some((Ok(Bytes::from(buffer)), Some(file)))
            }
            Err(_) => Some((Err(infrastructure("read validated image")), None)),
        }
    }))
}

fn infrastructure(operation: &'static str) -> FileError {
    FileError::Infrastructure(operation.into())
}

#[cfg(test)]
mod tests;
