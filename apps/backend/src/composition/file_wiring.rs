use std::sync::Arc;

use file::{
    application::{FileCleanupUseCase, FileManagementConfigProvider, FileService, FileServiceDependencies, FileUseCase},
    infra::{BoundedImagePreviewProcessor, LocalFileProvider, StorageFileRepository},
};
use storage::Database;

use crate::BackendResult;

pub(super) struct FileServices {
    pub(super) use_case: Arc<dyn FileUseCase>,
    pub(super) cleanup: Arc<dyn FileCleanupUseCase>,
}

pub(super) fn build_file_services(
    data_directory: &std::path::Path,
    database: Database,
    config: Arc<dyn FileManagementConfigProvider>,
) -> BackendResult<FileServices> {
    let repository = StorageFileRepository::new(database);
    let provider = Arc::new(LocalFileProvider::new(data_directory)?);
    let service = Arc::new(FileService::new(
        repository,
        FileServiceDependencies {
            provider,
            config,
            image_previews: Arc::new(BoundedImagePreviewProcessor),
        },
    ));
    Ok(FileServices {
        use_case: service.clone(),
        cleanup: service,
    })
}
