use kernel::pagination::CursorPageRequest;

use crate::application::{ObjectKey, ProviderUploadRef};
use crate::domain::{ByteSize, ContentDigest, ProviderKey, SpaceId};

use super::{BeginUploadSessionCommand, ExistingObject, FileAccessScope, FileSpaceQuery, ProviderCleanupKind, UpdateSpaceCommand};

pub struct FileSpaceListRequest<'a> {
    pub actor: &'a FileAccessScope,
    pub query: FileSpaceQuery,
    pub page: CursorPageRequest,
    pub default_quota: ByteSize,
}

pub struct ReusableObjectLookup<'a> {
    pub actor: &'a FileAccessScope,
    pub space_id: SpaceId,
    pub digest: ContentDigest,
    pub size: ByteSize,
}

pub struct UpdateSpaceRequest<'a> {
    pub actor: &'a FileAccessScope,
    pub space_id: SpaceId,
    pub command: UpdateSpaceCommand,
    pub default_quota: ByteSize,
}

pub struct ProviderCleanupRecordRequest<'a> {
    pub provider_key: &'a ProviderKey,
    pub kind: ProviderCleanupKind,
    pub object_key: Option<&'a ObjectKey>,
    pub upload_ref: Option<&'a ProviderUploadRef>,
}

pub struct UploadIntentLookup<'a> {
    pub actor: &'a FileAccessScope,
    pub owner_user_id: &'a str,
    pub space_id: SpaceId,
    pub idempotency_key: &'a str,
}

pub struct ReusedUploadCreate<'a> {
    pub actor: &'a FileAccessScope,
    pub command: BeginUploadSessionCommand,
    pub object: ExistingObject,
    pub part_size: ByteSize,
}
