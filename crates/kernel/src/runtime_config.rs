use async_trait::async_trait;
use serde::Deserialize;
use std::{error::Error, fmt};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExportBatchConfig {
    pub page_size: u64,
}

#[derive(Debug)]
pub enum RuntimeConfigError {
    InvalidJson(serde_json::Error),
    NonPositiveExportPageSize,
}

impl fmt::Display for RuntimeConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson(_) => formatter.write_str("export batch config must be valid JSON"),
            Self::NonPositiveExportPageSize => formatter.write_str("export batch config page_size must be greater than 0"),
        }
    }
}

impl Error for RuntimeConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidJson(error) => Some(error),
            Self::NonPositiveExportPageSize => None,
        }
    }
}

/// Parses and validates the shared runtime export batching parameter.
pub fn parse_export_batch_config(value: &str) -> Result<ExportBatchConfig, RuntimeConfigError> {
    let config = serde_json::from_str(value).map_err(RuntimeConfigError::InvalidJson)?;
    validate_export_batch_config(config)
}

fn validate_export_batch_config(config: ExportBatchConfig) -> Result<ExportBatchConfig, RuntimeConfigError> {
    if config.page_size == 0 {
        return Err(RuntimeConfigError::NonPositiveExportPageSize);
    }
    Ok(config)
}

#[async_trait]
pub trait ExportConfigProvider: Send + Sync + 'static {
    type Error;

    async fn export_batch_config(&self) -> Result<ExportBatchConfig, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::{ExportBatchConfig, RuntimeConfigError, parse_export_batch_config};

    #[test]
    fn parses_and_validates_export_batch_config() {
        assert_eq!(parse_export_batch_config(r#"{"page_size":100}"#).unwrap(), ExportBatchConfig { page_size: 100 });
        assert!(matches!(
            parse_export_batch_config(r#"{"page_size":0}"#),
            Err(RuntimeConfigError::NonPositiveExportPageSize)
        ));
        assert!(matches!(
            parse_export_batch_config(r#"{"page_size":100,"unexpected":true}"#),
            Err(RuntimeConfigError::InvalidJson(_))
        ));
    }
}
