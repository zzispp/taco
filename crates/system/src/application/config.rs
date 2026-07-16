use constants::system_config::EXPORT_BATCH_CONFIG_KEY;
use kernel::runtime_config::ExportBatchConfig;

use super::{SystemError, SystemResult};

pub fn parse_export_batch_config(value: &str) -> SystemResult<ExportBatchConfig> {
    kernel::runtime_config::parse_export_batch_config(value).map_err(|error| {
        taco_tracing::error_with_fields!("invalid system export runtime config", &error, key = EXPORT_BATCH_CONFIG_KEY);
        SystemError::InvalidInput(kernel::error::LocalizedError::new("errors.system.invalid_export_batch_config"))
    })
}

#[cfg(test)]
mod tests {
    use kernel::runtime_config::{ExportBatchConfig, MAX_EXPORT_BATCH_PAGE_SIZE};

    use super::{SystemError, parse_export_batch_config};

    #[test]
    fn export_batch_config_uses_system_owned_error() {
        assert_eq!(parse_export_batch_config(r#"{"page_size":100}"#).unwrap(), ExportBatchConfig { page_size: 100 });
        let above_maximum = format!(r#"{{"page_size":{}}}"#, MAX_EXPORT_BATCH_PAGE_SIZE + 1);

        for invalid in [r#"{"page_size":0}"#, &above_maximum] {
            let SystemError::InvalidInput(error) = parse_export_batch_config(invalid).unwrap_err() else {
                panic!("invalid export config must use the system invalid-input error");
            };
            assert_eq!(error.key(), "errors.system.invalid_export_batch_config");
        }
    }
}
