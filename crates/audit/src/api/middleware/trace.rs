use axum::http::{HeaderMap, Uri};

use crate::api::sanitize::normalized_request_id;

pub(super) fn request_id(headers: &HeaderMap) -> String {
    normalized_request_id(&header(headers, "x-request-id"))
}

pub(super) fn trace_missing_peer(uri: &Uri, headers: &HeaderMap) {
    let error = std::io::Error::other("ConnectInfo<SocketAddr> missing from request extensions");
    hook_tracing::error_with_fields!(
        "operation audit client peer missing",
        &error,
        request_id = request_id(headers),
        route = uri.path(),
        event_type = "operation",
        reason = "missing_peer"
    );
}

fn header(headers: &HeaderMap, name: &'static str) -> String {
    headers.get(name).and_then(|value| value.to_str().ok()).unwrap_or_default().into()
}
