use super::*;
use crate::error::keys;
use uuid::Uuid;

pub(super) struct PartContentValidation {
    pub(super) expected_size: ByteSize,
    pub(super) expected_digest: ContentDigest,
    pub(super) actual_size: ByteSize,
    pub(super) actual_digest: ContentDigest,
}

pub(super) struct PartAssembly<'a> {
    pub(super) target: &'a Path,
    pub(super) paths: &'a LocalPaths,
    pub(super) session: &'a UploadSession,
    pub(super) parts: &'a [ProviderPartReceipt],
}

struct CopyAndHash<'a> {
    input: &'a mut File,
    output: &'a mut File,
    hasher: &'a mut Sha256,
    size: u64,
}

struct AtomicWrite<'a> {
    temporary: &'a Path,
    target: &'a Path,
    bytes: &'a [u8],
    operation: &'static str,
}

pub(super) fn validate_begin_upload(request: &BeginUpload) -> FileResult<()> {
    if request.expected_size == ByteSize::ZERO {
        return Err(FileError::InvalidInput(keys::EMPTY_FILE));
    }
    if request.part_size == ByteSize::ZERO {
        return Err(FileError::InvalidInput(keys::UPLOAD_PART_SIZE_INVALID));
    }
    Ok(())
}

pub(super) fn expected_part_size(session: &UploadSession, part: PartNumber) -> FileResult<ByteSize> {
    let total = session.expected_size.bytes();
    let part_size = session.part_size.bytes();
    let count = total.div_ceil(part_size);
    let number = u64::from(part.value());
    if number == 0 || number > count {
        return Err(FileError::InvalidPart);
    }
    if number < count {
        return Ok(session.part_size);
    }
    Ok(ByteSize::from_bytes(total - part_size * (count - 1)))
}

pub(super) fn validate_part_content(validation: PartContentValidation) -> FileResult<()> {
    let PartContentValidation {
        expected_size,
        expected_digest,
        actual_size,
        actual_digest,
    } = validation;
    if expected_size != actual_size {
        return Err(FileError::SizeMismatch);
    }
    if expected_digest != actual_digest {
        return Err(FileError::DigestMismatch);
    }
    Ok(())
}

pub(super) fn validate_completed_object(session: &UploadSession, size: ByteSize, digest: ContentDigest) -> FileResult<()> {
    if session.expected_size != size {
        return Err(FileError::SizeMismatch);
    }
    if session.expected_digest != digest {
        return Err(FileError::DigestMismatch);
    }
    Ok(())
}

pub(super) fn validate_receipts(session: &UploadSession, mut parts: Vec<ProviderPartReceipt>) -> FileResult<Vec<ProviderPartReceipt>> {
    parts.sort_by_key(|part| part.part_number);
    let expected_count = session.expected_size.bytes().div_ceil(session.part_size.bytes());
    if parts.len() as u64 != expected_count {
        return Err(FileError::UploadIncomplete);
    }
    for (index, part) in parts.iter().enumerate() {
        let expected_number = u32::try_from(index + 1).map_err(|_| FileError::InvalidPart)?;
        let expected_ref = ProviderPartRef::new(part.part_number.value().to_string())?;
        if part.part_number.value() != expected_number || part.provider_part_ref != expected_ref || part.size != expected_part_size(session, part.part_number)?
        {
            return Err(FileError::InvalidPart);
        }
    }
    Ok(parts)
}

pub(super) async fn write_stream(path: &Path, body: &mut crate::application::ByteStream, expected_size: ByteSize) -> FileResult<(ByteSize, ContentDigest)> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .await
        .map_err(|_| provider_io("create upload part"))?;
    let mut hasher = Sha256::new();
    let mut size = 0_u64;
    while let Some(chunk) = body.next().await {
        let chunk = chunk?;
        size = size.checked_add(chunk.len() as u64).ok_or(FileError::SizeMismatch)?;
        if size > expected_size.bytes() {
            return Err(FileError::SizeMismatch);
        }
        file.write_all(&chunk).await.map_err(|_| provider_io("write upload part"))?;
        hasher.update(&chunk);
    }
    file.sync_all().await.map_err(|_| provider_io("sync upload part"))?;
    Ok((ByteSize::from_bytes(size), ContentDigest::from_digest(hasher.finalize().into())))
}

pub(super) async fn install_part(temporary: &Path, target: &Path) -> FileResult<()> {
    match fs::hard_link(temporary, target).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(_) => Err(provider_io("install upload part")),
    }
}

pub(super) async fn assemble_parts(assembly: PartAssembly<'_>) -> FileResult<(ByteSize, ContentDigest)> {
    let PartAssembly { target, paths, session, parts } = assembly;
    let session_id = local_session_id(&session.provider_upload_ref)?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target)
        .await
        .map_err(|_| provider_io("create completed object"))?;
    let mut hasher = Sha256::new();
    let mut size = 0_u64;
    for receipt in parts {
        let part_path = paths.part(session_id, receipt.part_number);
        let (part_size, part_digest) = digest_file(&part_path, "verify upload part").await?;
        if part_size != receipt.size || part_digest != receipt.digest {
            return Err(FileError::DigestMismatch);
        }
        let mut input = File::open(part_path).await.map_err(|_| FileError::UploadIncomplete)?;
        size = copy_and_hash(CopyAndHash {
            input: &mut input,
            output: &mut output,
            hasher: &mut hasher,
            size,
        })
        .await?;
    }
    output.sync_all().await.map_err(|_| provider_io("sync completed object"))?;
    Ok((ByteSize::from_bytes(size), ContentDigest::from_digest(hasher.finalize().into())))
}

async fn copy_and_hash(copy: CopyAndHash<'_>) -> FileResult<u64> {
    let CopyAndHash {
        input,
        output,
        hasher,
        mut size,
    } = copy;
    let mut buffer = vec![0_u8; STREAM_CHUNK_BYTES];
    loop {
        let read = input.read(&mut buffer).await.map_err(|_| provider_io("read upload part"))?;
        if read == 0 {
            return Ok(size);
        }
        output.write_all(&buffer[..read]).await.map_err(|_| provider_io("assemble completed object"))?;
        hasher.update(&buffer[..read]);
        size = size.checked_add(read as u64).ok_or(FileError::SizeMismatch)?;
    }
}

pub(super) fn stored_object(session: &UploadSession, size: ByteSize, digest: ContentDigest) -> StoredObject {
    StoredObject {
        id: session.stored_object_id,
        provider_key: session.provider_key.clone(),
        key: session.key.clone(),
        size,
        digest: Some(digest),
    }
}

pub(super) fn local_session_id(provider_upload_ref: &ProviderUploadRef) -> FileResult<UploadId> {
    UploadId::parse(provider_upload_ref.as_str())
}

pub(super) async fn digest_file(path: &Path, operation: &'static str) -> FileResult<(ByteSize, ContentDigest)> {
    let mut file = File::open(path).await.map_err(|_| provider_io(operation))?;
    let mut hasher = Sha256::new();
    let mut size = 0_u64;
    let mut buffer = vec![0_u8; STREAM_CHUNK_BYTES];
    loop {
        let read = file.read(&mut buffer).await.map_err(|_| provider_io(operation))?;
        if read == 0 {
            return Ok((ByteSize::from_bytes(size), ContentDigest::from_digest(hasher.finalize().into())));
        }
        hasher.update(&buffer[..read]);
        size = size.checked_add(read as u64).ok_or(FileError::SizeMismatch)?;
    }
}

pub(super) fn object_stream(file: File, remaining: u64) -> ObjectStream {
    Box::pin(stream::unfold((file, remaining), |(mut file, remaining)| async move {
        if remaining == 0 {
            return None;
        }
        let limit = remaining.min(STREAM_CHUNK_BYTES as u64) as usize;
        let mut buffer = vec![0_u8; limit];
        match file.read(&mut buffer).await {
            Ok(0) => Some((Err(provider_io("read object stream")), (file, 0))),
            Ok(read) => {
                buffer.truncate(read);
                Some((Ok(Bytes::from(buffer)), (file, remaining - read as u64)))
            }
            Err(_) => Some((Err(provider_io("read object stream")), (file, 0))),
        }
    }))
}

pub(super) async fn read_json<T: DeserializeOwned>(path: &Path, not_found: FileError, operation: &'static str) -> FileResult<T> {
    let bytes = match fs::read(path).await {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Err(not_found),
        Err(_) => return Err(provider_io(operation)),
    };
    serde_json::from_slice(&bytes).map_err(|_| provider_io(operation))
}

pub(super) async fn write_json_atomic(path: &Path, value: &impl Serialize, operation: &'static str) -> FileResult<()> {
    let bytes = serde_json::to_vec(value).map_err(|_| provider_io(operation))?;
    let temporary = path.with_extension(format!("tmp-{}", Uuid::now_v7()));
    let result = write_atomic_bytes(AtomicWrite {
        temporary: &temporary,
        target: path,
        bytes: &bytes,
        operation,
    })
    .await;
    if result.is_err() {
        remove_if_exists(&temporary, "remove failed metadata write").await?;
    }
    result
}

async fn write_atomic_bytes(write: AtomicWrite<'_>) -> FileResult<()> {
    let AtomicWrite {
        temporary,
        target,
        bytes,
        operation,
    } = write;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temporary)
        .await
        .map_err(|_| provider_io(operation))?;
    file.write_all(bytes).await.map_err(|_| provider_io(operation))?;
    file.sync_all().await.map_err(|_| provider_io(operation))?;
    fs::rename(temporary, target).await.map_err(|_| provider_io(operation))
}

pub(super) async fn try_exists(path: &Path, operation: &'static str) -> FileResult<bool> {
    fs::try_exists(path).await.map_err(|_| provider_io(operation))
}

pub(super) async fn remove_if_exists(path: &Path, operation: &'static str) -> FileResult<()> {
    match fs::remove_file(path).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(provider_io(operation)),
    }
}

pub(super) async fn remove_directory_if_exists(path: &Path, operation: &'static str) -> FileResult<()> {
    match fs::remove_dir_all(path).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(provider_io(operation)),
    }
}

pub(super) const fn provider_io(operation: &'static str) -> FileError {
    FileError::ProviderIo { operation }
}
