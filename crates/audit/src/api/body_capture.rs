use std::{
    pin::Pin,
    sync::{Arc, Mutex, MutexGuard},
    task::{Context, Poll},
};

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, header::CONTENT_TYPE},
};
use bytes::Bytes;
use http_body::{Body as HttpBody, Frame, SizeHint};

#[derive(Clone, Debug)]
pub(crate) struct SharedCapture(Arc<Mutex<CaptureState>>);

#[derive(Clone, Debug, Default)]
struct CaptureState {
    bytes: Vec<u8>,
    truncated: bool,
    stream_error: bool,
    complete: bool,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct CaptureSnapshot {
    pub bytes: Vec<u8>,
    pub truncated: bool,
    pub stream_error: bool,
}

#[derive(Debug, thiserror::Error)]
#[error("audit body capture mutex is poisoned")]
struct CaptureLockError;

pub(crate) struct CaptureTrace<'a> {
    pub request_id: &'a str,
    pub route: &'a str,
    pub phase: &'static str,
}

pub(crate) enum RequestCapture {
    Empty,
    Fixed(String),
    Stream { content_type: Option<String>, capture: SharedCapture },
}

impl RequestCapture {
    pub(crate) fn finish(self, trace: CaptureTrace<'_>) -> String {
        match self {
            Self::Empty => String::new(),
            Self::Fixed(value) => value,
            Self::Stream { content_type, capture } => request_stream_snapshot(content_type.as_deref(), capture, trace),
        }
    }
}

impl SharedCapture {
    pub(crate) fn new() -> Self {
        Self(Arc::new(Mutex::new(CaptureState::default())))
    }

    pub(crate) fn snapshot(&self) -> CaptureSnapshot {
        let Ok(state) = self.lock_state("snapshot") else {
            return CaptureSnapshot {
                bytes: Vec::new(),
                truncated: false,
                stream_error: true,
            };
        };
        CaptureSnapshot {
            bytes: state.bytes.clone(),
            truncated: state.truncated,
            stream_error: state.stream_error || !state.complete,
        }
    }

    fn record_frame(&self, frame: &Frame<Bytes>) {
        let Some(data) = frame.data_ref() else { return };
        let Ok(mut state) = self.lock_state("record_frame") else { return };
        let remaining = super::sanitize::SNAPSHOT_CAPTURE_MAX_BYTES.saturating_sub(state.bytes.len());
        if remaining == 0 {
            state.truncated = true;
            return;
        }
        let bytes = data.as_ref();
        let take = remaining.min(bytes.len());
        state.bytes.extend_from_slice(&bytes[..take]);
        state.truncated |= take < bytes.len();
    }

    fn mark_complete(&self) {
        let Ok(mut state) = self.lock_state("mark_complete") else { return };
        state.complete = true;
    }

    fn mark_stream_error(&self) {
        let Ok(mut state) = self.lock_state("mark_stream_error") else { return };
        state.stream_error = true;
    }

    fn lock_state(&self, operation: &'static str) -> Result<MutexGuard<'_, CaptureState>, CaptureLockError> {
        self.0.lock().map_err(|_| {
            let error = CaptureLockError;
            hook_tracing::error_with_fields!(
                "operation audit body capture state lock failed",
                &error,
                event_type = "operation",
                reason = operation
            );
            error
        })
    }
}

pub(crate) fn wrap_body(body: Body, capture: SharedCapture, on_complete: Option<Box<dyn FnOnce(SharedCapture) + Send>>) -> Body {
    Body::new(CapturingBody {
        inner: Box::pin(body),
        capture,
        on_complete,
        completed: false,
    })
}

pub(crate) fn capture_request(request: Request, save: bool) -> (Request, RequestCapture) {
    if !save {
        return (request, RequestCapture::Empty);
    }
    let content_type = content_type(request.headers());
    if is_multipart(content_type.as_deref()) {
        let snapshot = super::sanitize::request_snapshot(content_type.as_deref(), &[]);
        return (request, RequestCapture::Fixed(snapshot));
    }
    let capture = SharedCapture::new();
    let (parts, body) = request.into_parts();
    let body = wrap_body(body, capture.clone(), None);
    (Request::from_parts(parts, body), RequestCapture::Stream { content_type, capture })
}

pub(crate) fn content_type(headers: &HeaderMap) -> Option<String> {
    headers.get(CONTENT_TYPE).and_then(|value| value.to_str().ok()).map(str::to_owned)
}

pub(crate) fn trace_stream_error(trace: CaptureTrace<'_>, failed: bool) {
    if !failed {
        return;
    }
    let error = std::io::Error::other(format!("operation audit {} body stream did not complete", trace.phase));
    hook_tracing::error_with_fields!(
        "operation audit body capture failed",
        &error,
        request_id = trace.request_id,
        route = trace.route,
        event_type = "operation"
    );
}

fn request_stream_snapshot(content_type: Option<&str>, capture: SharedCapture, trace: CaptureTrace<'_>) -> String {
    let captured = capture.snapshot();
    trace_stream_error(trace, captured.stream_error);
    super::sanitize::request_snapshot_from_capture(super::sanitize::CapturedBody {
        content_type,
        bytes: &captured.bytes,
        truncated: captured.truncated,
        stream_error: captured.stream_error,
    })
}

fn is_multipart(content_type: Option<&str>) -> bool {
    content_type.is_some_and(|value| value.to_ascii_lowercase().starts_with("multipart/"))
}

struct CapturingBody {
    inner: Pin<Box<Body>>,
    capture: SharedCapture,
    on_complete: Option<Box<dyn FnOnce(SharedCapture) + Send>>,
    completed: bool,
}

impl CapturingBody {
    fn finish(&mut self, stream_error: bool) {
        if self.completed {
            return;
        }
        if stream_error {
            self.capture.mark_stream_error();
        } else {
            self.capture.mark_complete();
        }
        self.completed = true;
        if let Some(callback) = self.on_complete.take() {
            callback(self.capture.clone());
        }
    }
}

impl Drop for CapturingBody {
    fn drop(&mut self) {
        let stream_error = !self.inner.is_end_stream();
        self.finish(stream_error);
    }
}

impl HttpBody for CapturingBody {
    type Data = Bytes;
    type Error = axum::Error;

    fn poll_frame(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let this = self.get_mut();
        match this.inner.as_mut().poll_frame(cx) {
            Poll::Ready(Some(Ok(frame))) => {
                this.capture.record_frame(&frame);
                Poll::Ready(Some(Ok(frame)))
            }
            Poll::Ready(Some(Err(error))) => {
                this.finish(true);
                Poll::Ready(Some(Err(error)))
            }
            Poll::Ready(None) => {
                this.finish(false);
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }
}

#[cfg(test)]
mod tests;
