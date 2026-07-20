use std::{borrow::Cow, collections::HashMap};

#[cfg(feature = "embedded-frontend")]
use axum::{Router, body::Body, http::Request, routing::get};
use axum::{
    body::to_bytes,
    http::{StatusCode, header},
};
#[cfg(feature = "embedded-frontend")]
use tower::ServiceExt;

#[cfg(feature = "embedded-frontend")]
use super::with_embedded_frontend;
use super::{FrontendAssets, response_for_path};

struct FixtureAssets {
    files: HashMap<&'static str, &'static [u8]>,
}

impl FixtureAssets {
    fn new(files: &[(&'static str, &'static [u8])]) -> Self {
        Self {
            files: files.iter().copied().collect(),
        }
    }
}

impl FrontendAssets for FixtureAssets {
    fn get(&self, path: &str) -> Option<Cow<'static, [u8]>> {
        self.files.get(path).map(|data| Cow::Borrowed(*data))
    }
}

#[tokio::test]
async fn serves_root_and_exported_route_directory_documents() {
    let assets = FixtureAssets::new(&[("index.html", b"home"), ("dashboard/index.html", b"dashboard"), ("404.html", b"not found")]);

    let root = response_for_path(&assets, "/");
    let route = response_for_path(&assets, "/dashboard");
    let trailing_route = response_for_path(&assets, "/dashboard/");

    assert_eq!(root.status(), StatusCode::OK);
    assert_eq!(to_bytes(root.into_body(), usize::MAX).await.unwrap().as_ref(), b"home");
    assert_eq!(to_bytes(route.into_body(), usize::MAX).await.unwrap().as_ref(), b"dashboard");
    assert_eq!(to_bytes(trailing_route.into_body(), usize::MAX).await.unwrap().as_ref(), b"dashboard");
}

#[tokio::test]
async fn unknown_frontend_paths_return_exported_not_found_document() {
    let assets = FixtureAssets::new(&[("index.html", b"home"), ("404.html", b"not found")]);

    let response = response_for_path(&assets, "/missing");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "text/html; charset=utf-8");
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-cache");
    assert_security_headers(response.headers());
    assert_eq!(to_bytes(response.into_body(), usize::MAX).await.unwrap().as_ref(), b"not found");
}

#[tokio::test]
async fn api_paths_never_receive_frontend_documents() {
    let assets = FixtureAssets::new(&[("index.html", b"home"), ("404.html", b"not found")]);

    let api_root = response_for_path(&assets, "/api");
    let api_path = response_for_path(&assets, "/api/missing");

    assert_eq!(api_root.status(), StatusCode::NOT_FOUND);
    assert_eq!(api_path.status(), StatusCode::NOT_FOUND);
    assert!(!api_root.headers().contains_key(header::CONTENT_TYPE));
    assert_eq!(to_bytes(api_path.into_body(), usize::MAX).await.unwrap().as_ref(), b"");
}

#[cfg(feature = "embedded-frontend")]
#[tokio::test]
async fn fallback_preserves_api_route_ownership() {
    let app = with_embedded_frontend(Router::new().route("/api/known", get(|| async { "known" })));

    let known = app.clone().oneshot(Request::get("/api/known").body(Body::empty()).unwrap()).await.unwrap();
    let missing = app.oneshot(Request::get("/api/missing").body(Body::empty()).unwrap()).await.unwrap();

    assert_eq!(known.status(), StatusCode::OK);
    assert_eq!(to_bytes(known.into_body(), usize::MAX).await.unwrap().as_ref(), b"known");
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    assert!(!missing.headers().contains_key(header::CONTENT_TYPE));
}

#[tokio::test]
async fn next_static_assets_are_immutable_while_public_assets_are_revalidated() {
    let assets = FixtureAssets::new(&[("_next/static/chunks/main.js", b"script"), ("favicon.ico", b"icon"), ("404.html", b"not found")]);

    let next_asset = response_for_path(&assets, "/_next/static/chunks/main.js");
    let public_asset = response_for_path(&assets, "/favicon.ico");

    assert_eq!(next_asset.status(), StatusCode::OK);
    assert_eq!(next_asset.headers()[header::CACHE_CONTROL], "public, max-age=31536000, immutable");
    assert_eq!(next_asset.headers()[header::CONTENT_TYPE], "application/javascript; charset=utf-8");
    assert_eq!(public_asset.headers()[header::CACHE_CONTROL], "no-cache");
    assert_eq!(public_asset.headers()[header::CONTENT_TYPE], "image/x-icon");
}

fn assert_security_headers(headers: &axum::http::HeaderMap) {
    assert_eq!(headers["x-frame-options"], "DENY");
    assert_eq!(headers["referrer-policy"], "no-referrer");
    assert_eq!(headers["permissions-policy"], "camera=(), microphone=(), geolocation=()");
    assert_eq!(headers["x-content-type-options"], "nosniff");
    assert_eq!(
        headers["content-security-policy"],
        "default-src 'self'; base-uri 'self'; connect-src 'self'; font-src 'self' data:; form-action 'self'; frame-ancestors 'none'; img-src 'self' blob: data: https:; manifest-src 'self'; media-src 'self' blob:; object-src 'none'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; worker-src 'self' blob:",
    );
}
