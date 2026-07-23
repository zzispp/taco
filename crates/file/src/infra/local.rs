use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use bytes::Bytes;
use futures_util::{StreamExt, stream};
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use sysinfo::Disks;
use tokio::{
    fs::{self, File, OpenOptions},
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};
use uuid::Uuid;

use crate::application::{
    BeginUpload, ByteRange, CompleteUpload, FileProvider, ObjectKey, ObjectRead, ObjectStream, ProviderPartReceipt, ProviderPartRef, ProviderUploadRef,
    StoredObject, UploadPart, UploadSession,
};
use crate::domain::{ByteSize, ContentDigest, PartNumber, ProviderCapacity, ProviderKey, StoredObjectId, UploadId};
use crate::{FileError, FileResult};

const FILES_DIRECTORY: &str = "files";
const OBJECTS_DIRECTORY: &str = "objects";
const PARTS_DIRECTORY: &str = "parts";
const DERIVATIVES_DIRECTORY: &str = "derivatives";
const MANIFEST_FILE: &str = "manifest.json";
const OBJECT_DATA_SUFFIX: &str = "data";
const OBJECT_METADATA_SUFFIX: &str = "json";
const STREAM_CHUNK_BYTES: usize = 64 * 1_024;
const MINIMUM_PART_SIZE_BYTES: u64 = 1;

#[derive(Clone)]
pub struct LocalFileProvider {
    paths: Arc<LocalPaths>,
}

#[derive(Debug)]
struct LocalPaths {
    root: PathBuf,
    objects: PathBuf,
    parts: PathBuf,
    derivatives: PathBuf,
}

impl LocalFileProvider {
    pub fn new(data_directory: impl AsRef<Path>) -> FileResult<Self> {
        let paths = LocalPaths::new(data_directory.as_ref())?;
        Ok(Self { paths: Arc::new(paths) })
    }

    async fn manifest(&self, provider_upload_ref: &ProviderUploadRef) -> FileResult<UploadSession> {
        let session_id = local_session_id(provider_upload_ref)?;
        let session: UploadSession = read_json(&self.paths.manifest(session_id), FileError::UploadNotFound, "read upload manifest").await?;
        if session.provider_upload_ref != *provider_upload_ref {
            return Err(FileError::UploadNotFound);
        }
        Ok(session)
    }

    async fn existing_part(&self, session: &UploadSession, number: PartNumber) -> FileResult<Option<ProviderPartReceipt>> {
        let path = self.paths.part(local_session_id(&session.provider_upload_ref)?, number);
        if !try_exists(&path, "inspect upload part").await? {
            return Ok(None);
        }
        let (size, digest) = digest_file(&path, "read upload part").await?;
        Ok(Some(ProviderPartReceipt {
            part_number: number,
            provider_part_ref: ProviderPartRef::new(number.value().to_string())?,
            size,
            digest,
        }))
    }

    async fn complete_new_object(&self, session: &UploadSession, parts: &[ProviderPartReceipt]) -> FileResult<StoredObject> {
        let session_id = local_session_id(&session.provider_upload_ref)?;
        let temporary = self.paths.completing(session_id);
        let result = self.assemble_and_install(&temporary, session, parts).await;
        remove_if_exists(&temporary, "remove completed temporary object").await?;
        let object = result?;
        self.remove_session(session_id).await?;
        Ok(object)
    }

    async fn assemble_and_install(&self, temporary: &Path, session: &UploadSession, parts: &[ProviderPartReceipt]) -> FileResult<StoredObject> {
        let (size, digest) = assemble_parts(temporary, &self.paths, session, parts).await?;
        validate_completed_object(session, size, digest)?;
        let object = stored_object(session, size, digest);
        self.install_object(temporary, &object).await?;
        Ok(object)
    }

    async fn install_object(&self, temporary: &Path, object: &StoredObject) -> FileResult<()> {
        let data_path = self.paths.object_data(&object.key);
        let created = match fs::hard_link(temporary, &data_path).await {
            Ok(()) => true,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                let existing = self.stat(&object.key).await?;
                if existing.size != object.size || existing.digest != object.digest {
                    return Err(FileError::UploadPartConflict);
                }
                false
            }
            Err(_) => return Err(provider_io("install completed object")),
        };
        if let Err(error) = write_json_atomic(&self.paths.object_metadata(&object.key), object, "write object metadata").await {
            if created {
                remove_if_exists(&data_path, "rollback object data").await?;
            }
            return Err(error);
        }
        Ok(())
    }

    async fn remove_session(&self, session_id: UploadId) -> FileResult<()> {
        remove_directory_if_exists(&self.paths.session(session_id), "remove upload session").await
    }
}

#[async_trait]
impl crate::application::FileProvider for LocalFileProvider {
    fn provider_key(&self) -> ProviderKey {
        ProviderKey::local()
    }

    fn minimum_part_size(&self) -> ByteSize {
        ByteSize::from_bytes(MINIMUM_PART_SIZE_BYTES)
    }

    async fn begin_upload(&self, request: BeginUpload) -> FileResult<UploadSession> {
        validate_begin_upload(&request)?;
        let session_id = UploadId::new();
        let session = UploadSession {
            stored_object_id: request.stored_object_id,
            provider_key: self.provider_key(),
            provider_upload_ref: ProviderUploadRef::new(session_id.to_string())?,
            key: ObjectKey::new(request.stored_object_id.to_string())?,
            expected_size: request.expected_size,
            expected_digest: request.expected_digest,
            part_size: request.part_size,
        };
        fs::create_dir(&self.paths.session(session_id))
            .await
            .map_err(|_| provider_io("create upload session"))?;
        write_json_atomic(&self.paths.manifest(session_id), &session, "write upload manifest").await?;
        Ok(session)
    }

    async fn write_part(&self, mut request: UploadPart) -> FileResult<ProviderPartReceipt> {
        let session = self.manifest(&request.provider_upload_ref).await?;
        let expected_size = expected_part_size(&session, request.part_number)?;
        if let Some(existing) = self.existing_part(&session, request.part_number).await? {
            if existing.digest == request.expected_digest && existing.size == expected_size {
                return Ok(existing);
            }
            return Err(FileError::UploadPartConflict);
        }
        let session_id = local_session_id(&session.provider_upload_ref)?;
        let temporary = self.paths.incoming_part(session_id);
        let result = write_stream(&temporary, &mut request.body, expected_size).await;
        let (size, digest) = match result {
            Ok(value) => value,
            Err(error) => {
                remove_if_exists(&temporary, "remove failed upload part").await?;
                return Err(error);
            }
        };
        if let Err(error) = validate_part_content(expected_size, request.expected_digest, size, digest) {
            remove_if_exists(&temporary, "remove invalid upload part").await?;
            return Err(error);
        }
        install_part(&temporary, &self.paths.part(session_id, request.part_number)).await?;
        remove_if_exists(&temporary, "remove temporary upload part").await?;
        let installed = self.existing_part(&session, request.part_number).await?.ok_or(FileError::UploadIncomplete)?;
        if installed.size != expected_size || installed.digest != request.expected_digest {
            return Err(FileError::UploadPartConflict);
        }
        Ok(installed)
    }

    async fn complete_upload(&self, request: CompleteUpload) -> FileResult<StoredObject> {
        let session = self.manifest(&request.provider_upload_ref).await?;
        let data_path = self.paths.object_data(&session.key);
        if try_exists(&data_path, "inspect completed object").await? {
            let object = self.stat(&session.key).await?;
            validate_completed_object(&session, object.size, object.digest.ok_or(FileError::DigestMismatch)?)?;
            self.remove_session(local_session_id(&session.provider_upload_ref)?).await?;
            return Ok(object);
        }
        let parts = validate_receipts(&session, request.parts)?;
        self.complete_new_object(&session, &parts).await
    }

    async fn abort_upload(&self, provider_upload_ref: &ProviderUploadRef) -> FileResult<()> {
        let session_id = local_session_id(provider_upload_ref)?;
        self.remove_session(session_id).await
    }

    async fn read_range(&self, key: &ObjectKey, range: Option<ByteRange>) -> FileResult<ObjectRead> {
        let object = self.stat(key).await?;
        let range = range.map(|value| value.within(object.size)).transpose()?;
        let mut file = File::open(self.paths.object_data(key)).await.map_err(|_| provider_io("open object"))?;
        let (offset, remaining) = range.map_or((0, object.size.bytes()), |value| (value.start(), value.byte_len()));
        file.seek(std::io::SeekFrom::Start(offset)).await.map_err(|_| provider_io("seek object"))?;
        Ok(ObjectRead {
            object,
            range,
            body: object_stream(file, remaining),
        })
    }

    async fn delete(&self, key: &ObjectKey) -> FileResult<()> {
        remove_if_exists(&self.paths.object_data(key), "delete object").await?;
        remove_if_exists(&self.paths.object_metadata(key), "delete object metadata").await
    }

    async fn stat(&self, key: &ObjectKey) -> FileResult<StoredObject> {
        let data_path = self.paths.object_data(key);
        if !try_exists(&data_path, "inspect object").await? {
            return Err(FileError::NotFound);
        }
        let metadata_path = self.paths.object_metadata(key);
        if try_exists(&metadata_path, "inspect object metadata").await? {
            return read_json(&metadata_path, FileError::NotFound, "read object metadata").await;
        }
        let (size, digest) = digest_file(&data_path, "read object").await?;
        Ok(StoredObject {
            id: StoredObjectId::parse(key.as_str())?,
            provider_key: self.provider_key(),
            key: key.clone(),
            size,
            digest: Some(digest),
        })
    }

    async fn capacity(&self) -> FileResult<ProviderCapacity> {
        let root = self.paths.root.clone();
        tokio::task::spawn_blocking(move || disk_capacity(&root))
            .await
            .map_err(|_| FileError::ProviderUnavailable {
                operation: "join disk capacity lookup",
            })?
    }
}

impl LocalPaths {
    fn new(data_directory: &Path) -> FileResult<Self> {
        let root = data_directory.join(FILES_DIRECTORY);
        let paths = Self {
            objects: root.join(OBJECTS_DIRECTORY),
            parts: root.join(PARTS_DIRECTORY),
            derivatives: root.join(DERIVATIVES_DIRECTORY),
            root,
        };
        for path in [&paths.root, &paths.objects, &paths.parts, &paths.derivatives] {
            std::fs::create_dir_all(path).map_err(|_| provider_io("create local provider directory"))?;
        }
        Ok(paths)
    }

    fn session(&self, id: UploadId) -> PathBuf {
        self.parts.join(id.to_string())
    }

    fn manifest(&self, id: UploadId) -> PathBuf {
        self.session(id).join(MANIFEST_FILE)
    }

    fn part(&self, id: UploadId, number: PartNumber) -> PathBuf {
        self.session(id).join(format!("{}.part", number.value()))
    }

    fn incoming_part(&self, id: UploadId) -> PathBuf {
        self.session(id).join(format!(".incoming-{}", Uuid::now_v7()))
    }

    fn completing(&self, id: UploadId) -> PathBuf {
        self.session(id).join(format!(".completing-{}", Uuid::now_v7()))
    }

    fn object_data(&self, key: &ObjectKey) -> PathBuf {
        self.objects.join(key.as_str()).with_extension(OBJECT_DATA_SUFFIX)
    }

    fn object_metadata(&self, key: &ObjectKey) -> PathBuf {
        self.objects.join(key.as_str()).with_extension(OBJECT_METADATA_SUFFIX)
    }
}

#[cfg(test)]
mod tests;

mod support;
use support::*;
