use bytes::Bytes;
use futures_util::{StreamExt as _, TryStreamExt, stream};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tokio::sync::Barrier;

use super::*;
use crate::application::{ByteStream, FileProvider, ProviderPartReceipt};
use crate::domain::{ContentDigest, PartNumber, StoredObjectId};

#[tokio::test]
async fn multipart_upload_completes_and_supports_range_reads() {
    let directory = tempfile::tempdir().unwrap();
    let provider = LocalFileProvider::new(directory.path()).unwrap();
    let content = b"abcdefghij";
    let session = begin(&provider, content, 4).await;
    let parts = vec![
        write(&provider, &session, 1, b"abcd").await,
        write(&provider, &session, 2, b"efgh").await,
        write(&provider, &session, 3, b"ij").await,
    ];

    let object = provider
        .complete_upload(CompleteUpload {
            provider_upload_ref: session.provider_upload_ref.clone(),
            parts,
        })
        .await
        .unwrap();
    let read = provider.read_range(&object.key, Some(ByteRange::new(2, 7).unwrap())).await.unwrap();

    assert_eq!(object.size, ByteSize::from_bytes(10));
    assert_eq!(object.digest, Some(ContentDigest::from_bytes(content)));
    assert_eq!(read_bytes(read.body).await, b"cdefg");
    assert_eq!(provider.stat(&object.key).await.unwrap(), object);
}

#[tokio::test]
async fn retrying_a_part_is_idempotent_only_for_the_same_digest() {
    let directory = tempfile::tempdir().unwrap();
    let provider = LocalFileProvider::new(directory.path()).unwrap();
    let session = begin(&provider, b"same", 4).await;
    let first = write(&provider, &session, 1, b"same").await;

    let retry = write(&provider, &session, 1, b"same").await;
    let conflict = write_result(&provider, &session, 1, b"diff").await.unwrap_err();

    assert_eq!(retry, first);
    assert_eq!(first.provider_part_ref.as_str(), "1");
    assert_eq!(conflict, FileError::UploadPartConflict);
}

#[tokio::test]
async fn invalid_part_content_removes_the_incoming_staging_file() {
    let directory = tempfile::tempdir().unwrap();
    let provider = LocalFileProvider::new(directory.path()).unwrap();
    let session = begin(&provider, b"body", 4).await;
    let error = write_result_with_digest(&provider, &session, 1, b"body", ContentDigest::from_bytes(b"wrong"))
        .await
        .unwrap_err();

    assert_eq!(error, FileError::DigestMismatch);
    let entries = std::fs::read_dir(provider.paths.session(local_session_id(&session.provider_upload_ref).unwrap()))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert!(entries.iter().all(|entry| !entry.file_name().to_string_lossy().starts_with(".incoming-")));
}

#[tokio::test]
async fn oversized_part_stops_consuming_after_the_first_overflow_chunk() {
    let directory = tempfile::tempdir().unwrap();
    let provider = LocalFileProvider::new(directory.path()).unwrap();
    let session = begin(&provider, b"body", 4).await;
    let polled = Arc::new(AtomicUsize::new(0));
    let observed = polled.clone();
    let body = Box::pin(
        stream::iter([Ok(Bytes::from_static(b"abc")), Ok(Bytes::from_static(b"def")), Ok(Bytes::from_static(b"ghi"))]).inspect(move |_| {
            observed.fetch_add(1, Ordering::Relaxed);
        }),
    );

    let error = provider
        .write_part(UploadPart {
            provider_upload_ref: session.provider_upload_ref,
            part_number: PartNumber::new(1).unwrap(),
            expected_digest: ContentDigest::from_bytes(b"body"),
            body,
        })
        .await
        .unwrap_err();

    assert_eq!(error, FileError::SizeMismatch);
    assert_eq!(polled.load(Ordering::Relaxed), 2);
}

#[tokio::test]
async fn concurrent_different_part_content_has_one_success_and_one_conflict() {
    let directory = tempfile::tempdir().unwrap();
    let provider = Arc::new(LocalFileProvider::new(directory.path()).unwrap());
    let session = begin(provider.as_ref(), b"same", 4).await;
    let barrier = Arc::new(Barrier::new(2));
    let first = concurrent_write(provider.clone(), session.clone(), barrier.clone(), b"same");
    let second = concurrent_write(provider.clone(), session.clone(), barrier, b"diff");
    let (first, second) = tokio::join!(first, second);

    let conflict = match (first, second) {
        (Ok(_), Err(error)) | (Err(error), Ok(_)) => error,
        (Ok(_), Ok(_)) => panic!("both conflicting part writes succeeded"),
        (Err(first), Err(second)) => panic!("both conflicting part writes failed: {first:?}, {second:?}"),
    };
    assert_eq!(conflict, FileError::UploadPartConflict);
}

#[tokio::test]
async fn completion_rejects_a_wrong_full_content_digest() {
    let directory = tempfile::tempdir().unwrap();
    let provider = LocalFileProvider::new(directory.path()).unwrap();
    let mut session = begin(&provider, b"body", 4).await;
    session.expected_digest = ContentDigest::from_bytes(b"wrong");
    write_json_atomic(
        &provider.paths.manifest(local_session_id(&session.provider_upload_ref).unwrap()),
        &session,
        "test manifest update",
    )
    .await
    .unwrap();
    let part = write(&provider, &session, 1, b"body").await;

    let error = provider
        .complete_upload(CompleteUpload {
            provider_upload_ref: session.provider_upload_ref.clone(),
            parts: vec![part],
        })
        .await
        .unwrap_err();

    assert_eq!(error, FileError::DigestMismatch);
    assert_eq!(provider.stat(&session.key).await.unwrap_err(), FileError::NotFound);
    assert_no_completing_file(&provider, &session);
}

#[tokio::test]
async fn failed_metadata_install_removes_the_new_object_data_link() {
    let directory = tempfile::tempdir().unwrap();
    let provider = LocalFileProvider::new(directory.path()).unwrap();
    let session = begin(&provider, b"body", 4).await;
    let part = write(&provider, &session, 1, b"body").await;
    let metadata_path = provider.paths.object_metadata(&session.key);
    std::fs::create_dir(&metadata_path).unwrap();

    let error = provider
        .complete_upload(CompleteUpload {
            provider_upload_ref: session.provider_upload_ref.clone(),
            parts: vec![part],
        })
        .await
        .unwrap_err();

    assert_eq!(error, provider_io("write object metadata"));
    assert!(!provider.paths.object_data(&session.key).exists());
    assert_no_completing_file(&provider, &session);
}

#[tokio::test]
async fn abort_and_delete_are_idempotent() {
    let directory = tempfile::tempdir().unwrap();
    let provider = LocalFileProvider::new(directory.path()).unwrap();
    let session = begin(&provider, b"body", 4).await;
    provider.abort_upload(&session.provider_upload_ref).await.unwrap();
    provider.abort_upload(&session.provider_upload_ref).await.unwrap();
    assert_eq!(write_result(&provider, &session, 1, b"body").await.unwrap_err(), FileError::UploadNotFound);

    let session = begin(&provider, b"body", 4).await;
    let part = write(&provider, &session, 1, b"body").await;
    let object = provider
        .complete_upload(CompleteUpload {
            provider_upload_ref: session.provider_upload_ref.clone(),
            parts: vec![part],
        })
        .await
        .unwrap();
    provider.delete(&object.key).await.unwrap();
    provider.delete(&object.key).await.unwrap();
    assert_eq!(provider.stat(&object.key).await.unwrap_err(), FileError::NotFound);
}

#[tokio::test]
async fn local_capacity_reports_the_backing_volume() {
    let directory = tempfile::tempdir().unwrap();
    let provider = LocalFileProvider::new(directory.path()).unwrap();

    let capacity = provider.capacity().await.unwrap();

    let ProviderCapacity::Bounded { total_bytes, available_bytes } = capacity else {
        panic!("local provider must report bounded disk capacity");
    };
    assert!(total_bytes.bytes() > 0);
    assert!(available_bytes <= total_bytes);
}

async fn begin(provider: &LocalFileProvider, content: &[u8], part_size: u64) -> UploadSession {
    provider
        .begin_upload(BeginUpload {
            stored_object_id: StoredObjectId::new(),
            expected_size: ByteSize::from_bytes(content.len() as u64),
            expected_digest: ContentDigest::from_bytes(content),
            part_size: ByteSize::from_bytes(part_size),
        })
        .await
        .unwrap()
}

async fn write(provider: &LocalFileProvider, session: &UploadSession, number: u32, bytes: &'static [u8]) -> ProviderPartReceipt {
    write_result(provider, session, number, bytes).await.unwrap()
}

async fn write_result(provider: &LocalFileProvider, session: &UploadSession, number: u32, bytes: &'static [u8]) -> FileResult<ProviderPartReceipt> {
    write_result_with_digest(provider, session, number, bytes, ContentDigest::from_bytes(bytes)).await
}

async fn write_result_with_digest(
    provider: &LocalFileProvider,
    session: &UploadSession,
    number: u32,
    bytes: &'static [u8],
    digest: ContentDigest,
) -> FileResult<ProviderPartReceipt> {
    provider
        .write_part(UploadPart {
            provider_upload_ref: session.provider_upload_ref.clone(),
            part_number: PartNumber::new(number).unwrap(),
            expected_digest: digest,
            body: byte_stream(bytes),
        })
        .await
}

async fn concurrent_write(
    provider: Arc<LocalFileProvider>,
    session: UploadSession,
    barrier: Arc<Barrier>,
    bytes: &'static [u8],
) -> FileResult<ProviderPartReceipt> {
    let body = Box::pin(stream::once(async move {
        barrier.wait().await;
        Ok(Bytes::from_static(bytes))
    }));
    provider
        .write_part(UploadPart {
            provider_upload_ref: session.provider_upload_ref,
            part_number: PartNumber::new(1).unwrap(),
            expected_digest: ContentDigest::from_bytes(bytes),
            body,
        })
        .await
}

fn byte_stream(bytes: &'static [u8]) -> ByteStream {
    Box::pin(stream::once(async move { Ok(Bytes::from_static(bytes)) }))
}

async fn read_bytes(body: crate::application::ObjectStream) -> Vec<u8> {
    body.try_collect::<Vec<_>>().await.unwrap().into_iter().flatten().collect()
}

fn assert_no_completing_file(provider: &LocalFileProvider, session: &UploadSession) {
    let session_id = local_session_id(&session.provider_upload_ref).unwrap();
    let entries = std::fs::read_dir(provider.paths.session(session_id))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert!(entries.iter().all(|entry| !entry.file_name().to_string_lossy().starts_with(".completing-")));
}
