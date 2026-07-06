use async_trait::async_trait;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct ExportBatchConfig {
    pub page_size: u64,
}

#[async_trait]
pub trait ExportConfigProvider: Send + Sync + 'static {
    type Error;

    async fn export_batch_config(&self) -> Result<ExportBatchConfig, Self::Error>;
}
