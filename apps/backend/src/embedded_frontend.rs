use axum::Router;

#[cfg(any(test, feature = "embedded-frontend"))]
use std::borrow::Cow;

#[cfg(feature = "embedded-frontend")]
use axum::http::Uri;
#[cfg(any(test, feature = "embedded-frontend"))]
use axum::{
    body::Body,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header},
    response::Response,
};
#[cfg(feature = "embedded-frontend")]
use rust_embed::RustEmbed;

#[cfg(any(test, feature = "embedded-frontend"))]
const API_ROOT_PATH: &str = "/api";

/// Adds the release-only embedded frontend fallback after all backend routes.
///
/// Call this only after API, health, upload, and other backend routes have
/// been registered. The fallback deliberately leaves `/api` and `/api/*` as
/// backend 404 responses rather than serving exported HTML.
pub fn with_embedded_frontend(app: Router) -> Router {
    #[cfg(feature = "embedded-frontend")]
    {
        app.fallback(serve_embedded_frontend)
    }

    #[cfg(not(feature = "embedded-frontend"))]
    {
        app
    }
}

#[cfg(feature = "embedded-frontend")]
#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../frontend/out"]
struct EmbeddedFrontendAssets;

#[cfg(feature = "embedded-frontend")]
async fn serve_embedded_frontend(uri: Uri) -> Response {
    response_for_path(&EmbeddedFrontendAssets, uri.path())
}

#[cfg(any(test, feature = "embedded-frontend"))]
trait FrontendAssets {
    fn get(&self, path: &str) -> Option<Cow<'static, [u8]>>;
}

#[cfg(feature = "embedded-frontend")]
impl FrontendAssets for EmbeddedFrontendAssets {
    fn get(&self, path: &str) -> Option<Cow<'static, [u8]>> {
        <Self as RustEmbed>::get(path).map(|file| file.data)
    }
}

#[cfg(any(test, feature = "embedded-frontend"))]
fn response_for_path(assets: &impl FrontendAssets, request_path: &str) -> Response {
    if is_api_path(request_path) {
        return empty_response(StatusCode::NOT_FOUND);
    }

    match resolved_asset(assets, request_path) {
        Some(asset) => asset_response(StatusCode::OK, &asset.path, asset.data),
        None => not_found_response(assets),
    }
}

#[cfg(any(test, feature = "embedded-frontend"))]
fn is_api_path(path: &str) -> bool {
    path == API_ROOT_PATH || path.starts_with("/api/")
}

#[cfg(any(test, feature = "embedded-frontend"))]
struct ResolvedAsset {
    path: String,
    data: Cow<'static, [u8]>,
}

#[cfg(any(test, feature = "embedded-frontend"))]
fn resolved_asset(assets: &impl FrontendAssets, request_path: &str) -> Option<ResolvedAsset> {
    for path in candidate_paths(request_path) {
        if let Some(data) = assets.get(&path) {
            return Some(ResolvedAsset { path, data });
        }
    }
    None
}

#[cfg(any(test, feature = "embedded-frontend"))]
fn candidate_paths(request_path: &str) -> Vec<String> {
    let path = request_path.trim_start_matches('/');
    if path.is_empty() {
        return vec!["index.html".into()];
    }

    let directory_path = path.trim_end_matches('/');
    if !valid_relative_path(directory_path) {
        return Vec::new();
    }
    if path.ends_with('/') {
        return vec![format!("{directory_path}/index.html")];
    }

    vec![directory_path.into(), format!("{directory_path}/index.html")]
}

#[cfg(any(test, feature = "embedded-frontend"))]
fn valid_relative_path(path: &str) -> bool {
    !path.is_empty() && path.split('/').all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

#[cfg(any(test, feature = "embedded-frontend"))]
fn not_found_response(assets: &impl FrontendAssets) -> Response {
    const NOT_FOUND_DOCUMENT: &str = "404.html";

    match assets.get(NOT_FOUND_DOCUMENT) {
        Some(data) => asset_response(StatusCode::NOT_FOUND, NOT_FOUND_DOCUMENT, data),
        None => empty_response(StatusCode::NOT_FOUND),
    }
}

#[cfg(any(test, feature = "embedded-frontend"))]
fn asset_response(status: StatusCode, path: &str, data: Cow<'static, [u8]>) -> Response {
    let mut response = Response::new(Body::from(data.into_owned()));
    *response.status_mut() = status;
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, content_type(path));
    headers.insert(header::CACHE_CONTROL, cache_control(path));
    apply_security_headers(headers);
    response
}

#[cfg(any(test, feature = "embedded-frontend"))]
fn empty_response(status: StatusCode) -> Response {
    let mut response = Response::new(Body::empty());
    *response.status_mut() = status;
    response
}

#[cfg(any(test, feature = "embedded-frontend"))]
fn content_type(path: &str) -> HeaderValue {
    match path.rsplit('.').next().unwrap_or_default() {
        "html" => HeaderValue::from_static("text/html; charset=utf-8"),
        "css" => HeaderValue::from_static("text/css; charset=utf-8"),
        "js" | "mjs" => HeaderValue::from_static("application/javascript; charset=utf-8"),
        "json" | "map" => HeaderValue::from_static("application/json; charset=utf-8"),
        "svg" => HeaderValue::from_static("image/svg+xml"),
        "png" => HeaderValue::from_static("image/png"),
        "jpg" | "jpeg" => HeaderValue::from_static("image/jpeg"),
        "gif" => HeaderValue::from_static("image/gif"),
        "webp" => HeaderValue::from_static("image/webp"),
        "avif" => HeaderValue::from_static("image/avif"),
        "ico" => HeaderValue::from_static("image/x-icon"),
        "woff" => HeaderValue::from_static("font/woff"),
        "woff2" => HeaderValue::from_static("font/woff2"),
        "ttf" => HeaderValue::from_static("font/ttf"),
        "otf" => HeaderValue::from_static("font/otf"),
        "wasm" => HeaderValue::from_static("application/wasm"),
        "txt" => HeaderValue::from_static("text/plain; charset=utf-8"),
        "xml" => HeaderValue::from_static("application/xml; charset=utf-8"),
        "webmanifest" => HeaderValue::from_static("application/manifest+json"),
        _ => HeaderValue::from_static("application/octet-stream"),
    }
}

#[cfg(any(test, feature = "embedded-frontend"))]
fn cache_control(path: &str) -> HeaderValue {
    if path.starts_with("_next/static/") {
        return HeaderValue::from_static("public, max-age=31536000, immutable");
    }
    HeaderValue::from_static("no-cache")
}

#[cfg(any(test, feature = "embedded-frontend"))]
fn apply_security_headers(headers: &mut HeaderMap) {
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(
            "default-src 'self'; base-uri 'self'; connect-src 'self'; font-src 'self' data:; form-action 'self'; frame-ancestors 'none'; img-src 'self' blob: data: https:; manifest-src 'self'; media-src 'self' blob:; object-src 'none'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; worker-src 'self' blob:",
        ),
    );
    headers.insert(HeaderName::from_static("x-frame-options"), HeaderValue::from_static("DENY"));
    headers.insert(HeaderName::from_static("referrer-policy"), HeaderValue::from_static("no-referrer"));
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    headers.insert(HeaderName::from_static("x-content-type-options"), HeaderValue::from_static("nosniff"));
}

#[cfg(test)]
mod tests;
