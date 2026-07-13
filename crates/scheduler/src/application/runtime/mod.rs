mod execution_state;
mod execution_task;
mod executor;
mod leader;
mod planner;
mod supervisor;

use std::{sync::Arc, time::Duration};

use tokio::sync::watch;

use super::{
    ChangeListenerFactory, ExecutionLease, LeaderLease, SchedulerRuntimeStore, SchedulerTelemetry,
    task::{TaskCatalog, TaskExecutionContext},
};

pub use supervisor::start_scheduler_runtime;

#[derive(Clone, Copy, Debug)]
pub struct SchedulerRuntimeConfig {
    pub reconcile_interval: Duration,
}

#[derive(Clone)]
pub struct SchedulerRuntimeParts {
    pub store: Arc<dyn SchedulerRuntimeStore>,
    pub catalog: Arc<dyn TaskCatalog>,
    pub task_context: TaskExecutionContext,
    pub leader_lease: Arc<dyn LeaderLease>,
    pub listener_factory: Arc<dyn ChangeListenerFactory>,
    pub execution_lease: Arc<dyn ExecutionLease>,
    pub telemetry: Arc<dyn SchedulerTelemetry>,
    pub executor_epoch: String,
}

#[derive(Clone)]
pub struct SchedulerRuntimeHandle {
    shutdown: watch::Sender<bool>,
}

impl SchedulerRuntimeHandle {
    pub fn shutdown(&self) {
        self.shutdown.send_replace(true);
    }
}
