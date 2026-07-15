use crate::application::{SystemError, SystemExportKind, SystemExportRequest, SystemExportSink, SystemRepository, SystemResult};

use super::validation::{localized, sanitize_config_filter, sanitize_dict_data_filter, sanitize_dict_type_filter, sanitize_post_filter};

pub(super) async fn export<R: SystemRepository>(repository: &R, request: SystemExportRequest, sink: &mut dyn SystemExportSink) -> SystemResult<()> {
    if request.batch_size == 0 {
        return Err(SystemError::InvalidInput(localized("errors.common.invalid_input")));
    }
    let kind = match request.kind {
        SystemExportKind::Posts(filter) => SystemExportKind::Posts(sanitize_post_filter(filter)),
        SystemExportKind::DictTypes(filter) => SystemExportKind::DictTypes(sanitize_dict_type_filter(filter)),
        SystemExportKind::DictData(filter) => SystemExportKind::DictData(sanitize_dict_data_filter(filter)),
        SystemExportKind::Configs(filter) => SystemExportKind::Configs(sanitize_config_filter(filter)),
    };
    repository
        .export(
            SystemExportRequest {
                kind,
                batch_size: request.batch_size,
            },
            sink,
        )
        .await
}
