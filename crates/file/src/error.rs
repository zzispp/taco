use crate::domain::{ByteSize, UploadState};

pub type FileResult<T> = Result<T, FileError>;

/// Stable localization keys for validation failures raised by the file context.
///
/// Keeping these keys in one bounded-context module prevents transport-facing
/// code from carrying user-facing sentences through domain and repository
/// layers. The API layer resolves the selected key for the request locale.
pub(crate) mod keys {
    pub(crate) const ACTIVE_BUSINESS_REFERENCES: &str = "errors.file.active_business_references";
    pub(crate) const ACTIVE_UPLOAD_TARGET: &str = "errors.file.active_upload_target";
    pub(crate) const AVATAR_REFERENCE: &str = "errors.file.avatar_reference";
    pub(crate) const AVAILABLE_CAPACITY_EXCEEDS_TOTAL: &str = "errors.file.available_capacity_exceeds_total";
    pub(crate) const CLEANUP_BATCH_SIZE_INVALID: &str = "errors.file.cleanup_batch_size_invalid";
    pub(crate) const CLEANUP_BATCH_SIZE_TOO_LARGE: &str = "errors.file.cleanup_batch_size_too_large";
    pub(crate) const CONFIG_DEFAULT_QUOTA_INVALID: &str = "errors.file.config_default_quota_invalid";
    pub(crate) const CONFIG_INVALID_JSON: &str = "errors.file.config_invalid_json";
    pub(crate) const CONFIG_MAX_FILE_BYTES_INVALID: &str = "errors.file.config_max_file_bytes_invalid";
    pub(crate) const CONFIG_UPLOAD_PART_BYTES_INVALID: &str = "errors.file.config_upload_part_bytes_invalid";
    pub(crate) const CONFIG_UPLOAD_SESSION_INACTIVITY_INVALID: &str = "errors.file.config_upload_session_inactivity_invalid";
    pub(crate) const CONTENT_TYPE_INVALID: &str = "errors.file.content_type_invalid";
    pub(crate) const CURSOR_LIMIT_INVALID: &str = "errors.file.cursor_limit_invalid";
    pub(crate) const CURSOR_LIMIT_TOO_LARGE: &str = "errors.file.cursor_limit_too_large";
    pub(crate) const CURSOR_MALFORMED: &str = "errors.file.cursor_malformed";
    pub(crate) const CURSOR_QUERY_MISMATCH: &str = "errors.file.cursor_query_mismatch";
    pub(crate) const DECLARED_DIGEST_REQUIRED: &str = "errors.file.declared_digest_required";
    pub(crate) const DIGEST_FORMAT_INVALID: &str = "errors.file.digest_format_invalid";
    pub(crate) const DIGEST_LENGTH_INVALID: &str = "errors.file.digest_length_invalid";
    pub(crate) const EMPTY_FILE: &str = "errors.file.empty_file";
    pub(crate) const ENTRY_NAME_FORBIDDEN_PATH: &str = "errors.file.entry_name_forbidden_path";
    pub(crate) const ENTRY_NAME_INVALID: &str = "errors.file.entry_name_invalid";
    pub(crate) const ENTRY_TYPE_INVALID: &str = "errors.file.entry_type_invalid";
    pub(crate) const FILE_IDS_REQUIRED: &str = "errors.file.file_ids_required";
    pub(crate) const FILE_IDS_TOO_MANY: &str = "errors.file.file_ids_too_many";
    pub(crate) const FILE_SIZE_EXCEEDED: &str = "errors.file.file_size_exceeded";
    pub(crate) const FOLDER_DOWNLOAD_FORBIDDEN: &str = "errors.file.folder_download_forbidden";
    pub(crate) const IDEMPOTENCY_KEY_INVALID_UTF8: &str = "errors.file.idempotency_key_invalid_utf8";
    pub(crate) const IDEMPOTENCY_KEY_INVALID: &str = "errors.file.idempotency_key_invalid";
    pub(crate) const IDEMPOTENCY_KEY_REQUIRED: &str = "errors.file.idempotency_key_required";
    pub(crate) const IDEMPOTENCY_KEY_TOO_LONG: &str = "errors.file.idempotency_key_too_long";
    pub(crate) const IDENTIFIER_INVALID: &str = "errors.file.identifier_invalid";
    pub(crate) const IMAGE_SOURCE_TOO_LARGE: &str = "errors.file.image_source_too_large";
    pub(crate) const OBJECT_KEY_FORBIDDEN_PATH: &str = "errors.file.object_key_forbidden_path";
    pub(crate) const OBJECT_KEY_INVALID: &str = "errors.file.object_key_invalid";
    pub(crate) const PARENT_FOLDER_INVALID: &str = "errors.file.parent_folder_invalid";
    pub(crate) const PART_DIGEST_HEADER_REQUIRED: &str = "errors.file.part_digest_header_required";
    pub(crate) const PROVIDER_CLEANUP_PAYLOAD_INVALID: &str = "errors.file.provider_cleanup_payload_invalid";
    pub(crate) const PROVIDER_KEY_INVALID: &str = "errors.file.provider_key_invalid";
    pub(crate) const PROVIDER_OBJECT_MISMATCH: &str = "errors.file.provider_object_mismatch";
    pub(crate) const PROVIDER_PART_REF_INVALID: &str = "errors.file.provider_part_ref_invalid";
    pub(crate) const PROVIDER_UPLOAD_REF_INVALID: &str = "errors.file.provider_upload_ref_invalid";
    pub(crate) const PURGE_REQUIRES_TRASHED: &str = "errors.file.purge_requires_trashed";
    pub(crate) const QUOTA_RELEASE_EXCEEDS_USAGE: &str = "errors.file.quota_release_exceeds_usage";
    pub(crate) const QUOTA_TOO_LARGE: &str = "errors.file.quota_too_large";
    pub(crate) const RANGE_HEADER_INVALID: &str = "errors.file.range_header_invalid";
    pub(crate) const RESPONSE_HEADER_INVALID: &str = "errors.file.response_header_invalid";
    pub(crate) const RETENTION_DAYS_TOO_LARGE: &str = "errors.file.retention_days_too_large";
    pub(crate) const SORT_FIELD_INVALID: &str = "errors.file.sort_field_invalid";
    pub(crate) const SORT_ORDER_INVALID: &str = "errors.file.sort_order_invalid";
    pub(crate) const SPACE_IDENTIFIER_REQUIRED: &str = "errors.file.space_identifier_required";
    pub(crate) const SYSTEM_FOLDER_IMMUTABLE: &str = "errors.file.system_folder_immutable";
    pub(crate) const TAG_REQUIRED: &str = "errors.file.tag_required";
    pub(crate) const TAG_INVALID: &str = "errors.file.tag_invalid";
    pub(crate) const TAG_TOO_LONG: &str = "errors.file.tag_too_long";
    pub(crate) const TIME_FILTER_INVALID: &str = "errors.file.time_filter_invalid";
    pub(crate) const UPLOAD_PART_SIZE_INVALID: &str = "errors.file.upload_part_size_invalid";
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum FileError {
    #[error("invalid file input: {0}")]
    InvalidInput(&'static str),
    #[error("file name conflicts with an existing entry")]
    NameConflict,
    #[error("file resource was not found")]
    NotFound,
    #[error("file operation is forbidden")]
    Forbidden,
    #[error("upload session was not found")]
    UploadNotFound,
    #[error("upload intent has already reached a terminal state")]
    UploadIntentTerminal,
    #[error("the completed upload result is no longer available")]
    UploadResultUnavailable,
    #[error("upload completion is already in progress")]
    UploadCompletionInProgress,
    #[error("upload cannot transition from {from:?} to {to:?}")]
    InvalidUploadTransition { from: UploadState, to: UploadState },
    #[error("upload is missing one or more parts")]
    UploadIncomplete,
    #[error("upload part is invalid")]
    InvalidPart,
    #[error("upload part number already contains different content")]
    UploadPartConflict,
    #[error("uploaded content digest does not match the expected digest")]
    DigestMismatch,
    #[error("uploaded content size does not match the expected size")]
    SizeMismatch,
    #[error("requested range is not satisfiable")]
    RangeNotSatisfiable,
    #[error("storage capacity is exhausted: requested {requested} bytes, available {available} bytes")]
    CapacityExceeded { requested: ByteSize, available: ByteSize },
    #[error("file quota is exhausted: requested {requested} bytes, remaining {remaining} bytes")]
    QuotaExceeded { requested: ByteSize, remaining: ByteSize },
    #[error("storage provider operation is unavailable: {operation}")]
    ProviderUnavailable { operation: &'static str },
    #[error("storage provider I/O operation failed: {operation}")]
    ProviderIo { operation: &'static str },
    #[error("file management infrastructure failed: {0}")]
    Infrastructure(String),
}
