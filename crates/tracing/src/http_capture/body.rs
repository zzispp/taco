use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use axum::body::Body;
use bytes::Bytes;
use http_body::{Body as HttpBody, Frame, SizeHint};

#[derive(Clone)]
pub(crate) struct SharedBodyCapture(Arc<Mutex<BodyCaptureState>>);

#[derive(Default)]
struct BodyCaptureState {
    bytes: Vec<u8>,
    truncated: bool,
    stream_error: bool,
    complete: bool,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct BodyCaptureSnapshot {
    pub bytes: Vec<u8>,
    pub truncated: bool,
    pub stream_error: bool,
}

pub(crate) struct BodyCaptureOptions {
    pub(crate) limit: usize,
    pub(crate) on_complete: Option<Box<dyn FnOnce(BodyCaptureSnapshot) + Send>>,
}

impl SharedBodyCapture {
    pub(crate) fn new() -> Self {
        Self(Arc::new(Mutex::new(BodyCaptureState::default())))
    }

    pub(crate) fn snapshot(&self) -> BodyCaptureSnapshot {
        let Ok(state) = self.0.lock() else {
            return BodyCaptureSnapshot {
                stream_error: true,
                ..Default::default()
            };
        };
        BodyCaptureSnapshot {
            bytes: state.bytes.clone(),
            truncated: state.truncated,
            stream_error: state.stream_error || !state.complete,
        }
    }

    fn record_frame(&self, frame: &Frame<Bytes>, limit: usize) {
        let Some(data) = frame.data_ref() else {
            return;
        };
        let Ok(mut state) = self.0.lock() else {
            return;
        };
        let remaining = limit.saturating_sub(state.bytes.len());
        let bytes = data.as_ref();
        let take = remaining.min(bytes.len());
        state.bytes.extend_from_slice(&bytes[..take]);
        state.truncated |= take < bytes.len();
    }

    fn finish(&self, stream_error: bool) {
        let Ok(mut state) = self.0.lock() else {
            return;
        };
        state.stream_error |= stream_error;
        state.complete = !stream_error;
    }
}

pub(crate) fn wrap_body(body: Body, capture: SharedBodyCapture, options: BodyCaptureOptions) -> Body {
    Body::new(CapturingBody {
        inner: Box::pin(body),
        capture,
        limit: options.limit,
        on_complete: options.on_complete,
        completed: false,
    })
}

struct CapturingBody {
    inner: Pin<Box<Body>>,
    capture: SharedBodyCapture,
    limit: usize,
    on_complete: Option<Box<dyn FnOnce(BodyCaptureSnapshot) + Send>>,
    completed: bool,
}

impl CapturingBody {
    fn finish(&mut self, stream_error: bool) {
        if self.completed {
            return;
        }
        self.capture.finish(stream_error);
        self.completed = true;
        if let Some(callback) = self.on_complete.take() {
            callback(self.capture.snapshot());
        }
    }
}

impl Drop for CapturingBody {
    fn drop(&mut self) {
        self.finish(!self.inner.is_end_stream());
    }
}

impl HttpBody for CapturingBody {
    type Data = Bytes;
    type Error = axum::Error;

    fn poll_frame(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let this = self.get_mut();
        match this.inner.as_mut().poll_frame(context) {
            Poll::Ready(Some(Ok(frame))) => {
                this.capture.record_frame(&frame, this.limit);
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
