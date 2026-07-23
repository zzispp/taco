use async_trait::async_trait;
use bytes::Bytes;
use futures_util::StreamExt;

use crate::application::{FileContent, ObjectStream};
use crate::{FileResult, domain::ByteSize};

pub const TEXT_PREVIEW_MAX_BYTES: u64 = 1024 * 1024;

#[derive(Debug)]
pub struct GeneratedThumbnail {
    pub content_type: &'static str,
    pub bytes: Bytes,
}

#[async_trait]
pub trait ImagePreviewProcessor: Send + Sync + 'static {
    async fn validate_inline(&self, content_type: &str, body: ObjectStream) -> FileResult<ObjectStream>;
    async fn generate_thumbnail(&self, content_type: &str, body: ObjectStream) -> FileResult<GeneratedThumbnail>;
}

/// Returns whether a stored content type is safe for an authenticated inline
/// response. Scriptable document formats remain download-only.
pub fn supports_inline_preview(content_type: Option<&str>) -> bool {
    let Some(content_type) = normalized_content_type(content_type) else {
        return false;
    };
    matches!(
        content_type.as_str(),
        "image/png"
            | "image/jpeg"
            | "image/webp"
            | "image/gif"
            | "application/pdf"
            | "application/json"
            | "application/xml"
            | "text/plain"
            | "text/csv"
            | "text/markdown"
            | "text/xml"
    ) || content_type.starts_with("audio/")
        || content_type.starts_with("video/")
}

pub fn supports_thumbnail(content_type: Option<&str>) -> bool {
    matches!(
        normalized_content_type(content_type).as_deref(),
        Some("image/png" | "image/jpeg" | "image/webp" | "image/gif")
    )
}

pub fn bounded_text_preview(mut content: FileContent) -> FileContent {
    if !is_text_preview(&content.metadata.content_type) {
        return content;
    }
    content.metadata.content_type = "text/plain; charset=utf-8".into();
    content.metadata.truncated = content.metadata.size.bytes() > TEXT_PREVIEW_MAX_BYTES;
    content.metadata.range = bounded_range(content.metadata.range);
    content.body = limit_stream(content.body, TEXT_PREVIEW_MAX_BYTES);
    content
}

fn is_text_preview(content_type: &str) -> bool {
    let normalized = normalized_content_type(Some(content_type));
    matches!(normalized.as_deref(), Some("application/json" | "application/xml")) || normalized.is_some_and(|value| value.starts_with("text/"))
}

fn bounded_range(range: Option<crate::application::ByteRange>) -> Option<crate::application::ByteRange> {
    let range = range?;
    if range.byte_len() <= TEXT_PREVIEW_MAX_BYTES {
        return Some(range);
    }
    Some(crate::application::ByteRange::new(range.start(), range.start() + TEXT_PREVIEW_MAX_BYTES).expect("bounded range remains non-empty"))
}

fn limit_stream(body: ObjectStream, limit: u64) -> ObjectStream {
    Box::pin(futures_util::stream::unfold((body, limit), |(mut body, remaining)| async move {
        if remaining == 0 {
            return None;
        }
        let next = body.next().await?;
        let next = next.map(|mut bytes| {
            bytes.truncate(bytes.len().min(remaining as usize));
            bytes
        });
        let consumed = next.as_ref().map_or(remaining, |bytes| bytes.len() as u64);
        Some((next, (body, remaining - consumed)))
    }))
}

fn normalized_content_type(content_type: Option<&str>) -> Option<String> {
    let value = content_type?.split(';').next()?.trim().to_ascii_lowercase();
    (!value.is_empty()).then_some(value)
}

pub fn thumbnail_content(name: String, thumbnail: GeneratedThumbnail) -> FileContent {
    let size = ByteSize::from_bytes(thumbnail.bytes.len() as u64);
    FileContent {
        metadata: crate::application::FileContentMetadata {
            name,
            content_type: thumbnail.content_type.into(),
            size,
            range: None,
            truncated: false,
            accept_ranges: false,
        },
        body: crate::application::as_object_stream(thumbnail.bytes),
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use futures_util::TryStreamExt;

    use super::{TEXT_PREVIEW_MAX_BYTES, bounded_text_preview, supports_inline_preview, supports_thumbnail};
    use crate::application::{FileContent, FileContentMetadata, as_object_stream};
    use crate::domain::ByteSize;

    #[test]
    fn preview_policy_accepts_only_bounded_safe_types() {
        for value in ["image/png", "application/pdf", "text/plain", "audio/mpeg", "video/mp4"] {
            assert!(supports_inline_preview(Some(value)), "{value} should be previewable");
        }
        for value in ["image/svg+xml", "text/html", "text/css", "application/javascript"] {
            assert!(!supports_inline_preview(Some(value)), "{value} must remain download-only");
        }
    }

    #[test]
    fn thumbnail_policy_accepts_only_supported_raster_types() {
        for value in ["image/png", "image/jpeg", "image/webp", "image/gif"] {
            assert!(supports_thumbnail(Some(value)), "{value} should support thumbnails");
        }
        assert!(!supports_thumbnail(Some("image/svg+xml")));
        assert!(supports_thumbnail(Some(" Image/JPEG; charset=binary ")));
    }

    #[tokio::test]
    async fn text_preview_is_plain_text_bounded_and_marked_truncated() {
        let bytes = Bytes::from(vec![b'a'; TEXT_PREVIEW_MAX_BYTES as usize + 7]);
        let content = bounded_text_preview(file_content("text/markdown", bytes));
        let metadata = content.metadata;
        let body = content.body.try_collect::<Vec<_>>().await.unwrap().concat();

        assert_eq!(metadata.content_type, "text/plain; charset=utf-8");
        assert!(metadata.truncated);
        assert_eq!(body.len(), TEXT_PREVIEW_MAX_BYTES as usize);
    }

    fn file_content(content_type: &str, bytes: Bytes) -> FileContent {
        FileContent {
            metadata: FileContentMetadata {
                name: "notes.md".into(),
                content_type: content_type.into(),
                size: ByteSize::from_bytes(bytes.len() as u64),
                range: None,
                truncated: false,
                accept_ranges: true,
            },
            body: as_object_stream(bytes),
        }
    }
}
