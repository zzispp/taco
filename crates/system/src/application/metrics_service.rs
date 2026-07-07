use async_trait::async_trait;

use crate::domain::ServerDashboard;

use super::{ServerMetricsCollector, ServerMetricsUseCase, SystemResult};

pub struct SystemMetricsService<C> {
    collector: C,
}

impl<C: ServerMetricsCollector> SystemMetricsService<C> {
    pub const fn new(collector: C) -> Self {
        Self { collector }
    }
}

#[async_trait]
impl<C: ServerMetricsCollector> ServerMetricsUseCase for SystemMetricsService<C> {
    async fn dashboard(&self) -> SystemResult<ServerDashboard> {
        self.collector.collect().await
    }
}
