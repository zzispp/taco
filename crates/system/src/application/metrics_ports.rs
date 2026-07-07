use async_trait::async_trait;

use crate::domain::ServerDashboard;

use super::SystemResult;

#[async_trait]
pub trait ServerMetricsUseCase: Send + Sync + 'static {
    async fn dashboard(&self) -> SystemResult<ServerDashboard>;
}

#[async_trait]
pub trait ServerMetricsCollector: Send + Sync + 'static {
    async fn collect(&self) -> SystemResult<ServerDashboard>;
}
