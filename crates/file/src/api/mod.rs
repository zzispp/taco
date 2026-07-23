mod dto;
mod endpoints;
mod error;
mod handlers;

use std::sync::Arc;

use audit_contract::{AuditOutboxRecorder, OperationAuditContext};
use axum::{
    Extension, Router,
    routing::{delete, get, post, put},
};

use crate::application::FileUseCase;
use crate::{FileError, FileResult};

pub use endpoints::endpoint_specs;
pub use error::FileApiError;

#[derive(Clone)]
pub struct FileApiState {
    pub files: Arc<dyn FileUseCase>,
    operation_audit: Arc<dyn AuditOutboxRecorder>,
}

impl FileApiState {
    pub fn new(files: Arc<dyn FileUseCase>, operation_audit: Arc<dyn AuditOutboxRecorder>) -> Self {
        Self { files, operation_audit }
    }

    async fn record_operation(&self, context: Option<Extension<OperationAuditContext>>) -> FileResult<()> {
        let Extension(context) = context.ok_or_else(|| FileError::Infrastructure("operation audit context is missing".into()))?;
        let record = context
            .success_record()
            .map_err(|error| FileError::Infrastructure(error.to_string()))?
            .ok_or_else(|| FileError::Infrastructure("operation audit actor is missing".into()))?;
        self.operation_audit
            .record(record)
            .await
            .map_err(|error| FileError::Infrastructure(error.to_string()))?;
        context.mark_persisted();
        Ok(())
    }
}

pub fn create_router(state: FileApiState) -> Router {
    use endpoints::*;
    use handlers::*;

    Router::new()
        .route(FILES_LIST.api_route_path(), get(list_files))
        .route(FILE_DIRECTORY_TRAIL.api_route_path(), get(file_directory_trail))
        .route(FILES_OVERVIEW.api_route_path(), get(file_overview))
        .route(FILE_SPACES_LIST.api_route_path(), get(list_file_spaces))
        .route(FILE_SPACE_UPDATE.api_route_path(), put(update_file_space))
        .route(FOLDER_CREATE.api_route_path(), post(create_file_folder))
        .route(FILE_GET.api_route_path(), get(get_file).put(update_file))
        .route(FILE_CONTENT.api_route_path(), get(download_file))
        .route(FILE_PREVIEW.api_route_path(), get(preview_file))
        .route(FILE_THUMBNAIL.api_route_path(), get(thumbnail_file))
        .route(FILE_TRASH.api_route_path(), post(trash_file))
        .route(FILE_RESTORE.api_route_path(), post(restore_file))
        .route(FILE_PURGE.api_route_path(), delete(purge_file))
        .route(FILES_TRASH_BATCH.api_route_path(), post(trash_files))
        .route(FILES_RESTORE_BATCH.api_route_path(), post(restore_files))
        .route(FILES_PURGE_BATCH.api_route_path(), post(purge_files))
        .route(PROVIDERS_LIST.api_route_path(), get(list_file_providers))
        .route(UPLOAD_SESSIONS_CREATE.api_route_path(), post(create_upload_session))
        .route(UPLOAD_SESSION_GET.api_route_path(), get(get_upload_session).delete(cancel_upload_session))
        .route(UPLOAD_SESSION_PART.api_route_path(), put(write_upload_part))
        .route(UPLOAD_SESSION_COMPLETE.api_route_path(), post(complete_upload_session))
        .with_state(state)
}
