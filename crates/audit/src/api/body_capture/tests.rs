use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll},
};

use axum::body::{Body, to_bytes};
use bytes::Bytes;
use http_body::{Body as HttpBody, Frame};

use super::{SharedCapture, wrap_body};

#[test]
fn poisoned_capture_state_returns_a_safe_stream_error_snapshot() {
    let capture = SharedCapture::new();
    let state = capture.0.clone();
    let panic = std::panic::catch_unwind(move || {
        let _guard = state.lock().unwrap();
        panic!("poison capture fixture");
    });

    assert!(panic.is_err());
    let snapshot = capture.snapshot();
    assert!(snapshot.bytes.is_empty());
    assert!(!snapshot.truncated);
    assert!(snapshot.stream_error);
}

#[tokio::test]
async fn dropping_an_already_complete_empty_body_is_not_a_stream_error() {
    let capture = SharedCapture::new();
    let callback_count = Arc::new(AtomicUsize::new(0));
    let callback_counter = callback_count.clone();
    let body = wrap_body(
        Body::empty(),
        capture.clone(),
        Some(Box::new(move |_| {
            callback_counter.fetch_add(1, Ordering::SeqCst);
        })),
    );

    assert!(body.is_end_stream());
    drop(body);

    assert!(!capture.snapshot().stream_error);
    assert_eq!(callback_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn bounded_capture_preserves_the_complete_business_body() {
    let original = vec![b'a'; super::super::sanitize::SNAPSHOT_CAPTURE_MAX_BYTES + 500];
    let capture = SharedCapture::new();
    let callback_count = Arc::new(AtomicUsize::new(0));
    let callback_counter = callback_count.clone();
    let body = wrap_body(
        Body::from(original.clone()),
        capture.clone(),
        Some(Box::new(move |_| {
            callback_counter.fetch_add(1, Ordering::SeqCst);
        })),
    );

    let forwarded = to_bytes(body, usize::MAX).await.unwrap();
    let snapshot = capture.snapshot();

    assert_eq!(forwarded.as_ref(), original);
    assert_eq!(snapshot.bytes.len(), super::super::sanitize::SNAPSHOT_CAPTURE_MAX_BYTES);
    assert!(snapshot.truncated);
    assert!(!snapshot.stream_error);
    assert_eq!(callback_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn capture_does_not_truncate_two_thousand_four_byte_unicode_characters() {
    let original = "\u{1f600}".repeat(2_000).into_bytes();
    let capture = SharedCapture::new();
    let body = wrap_body(Body::from(original.clone()), capture.clone(), None);

    let forwarded = to_bytes(body, usize::MAX).await.unwrap();
    let snapshot = capture.snapshot();

    assert_eq!(forwarded.as_ref(), original);
    assert_eq!(snapshot.bytes, original);
    assert!(!snapshot.truncated);
    assert!(!snapshot.stream_error);
}

#[tokio::test]
async fn body_stream_errors_are_forwarded_and_marked_once() {
    let capture = SharedCapture::new();
    let callback_count = Arc::new(AtomicUsize::new(0));
    let callback_counter = callback_count.clone();
    let body = wrap_body(
        Body::new(FailingBody::default()),
        capture.clone(),
        Some(Box::new(move |_| {
            callback_counter.fetch_add(1, Ordering::SeqCst);
        })),
    );

    assert!(to_bytes(body, usize::MAX).await.is_err());
    assert!(capture.snapshot().stream_error);
    assert_eq!(callback_count.load(Ordering::SeqCst), 1);
}

#[derive(Default)]
struct FailingBody {
    emitted: bool,
}

impl HttpBody for FailingBody {
    type Data = Bytes;
    type Error = std::io::Error;

    fn poll_frame(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        if self.emitted {
            return Poll::Ready(None);
        }
        self.emitted = true;
        Poll::Ready(Some(Err(std::io::Error::other("fixture body failure"))))
    }
}
