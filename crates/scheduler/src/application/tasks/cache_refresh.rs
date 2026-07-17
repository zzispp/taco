use async_trait::async_trait;
use scheduler_macros::scheduled_task;

use crate::application::task::{ScheduledTask, TaskExecutionContext, TaskExecutionFailure, TaskExecutionOutput, TaskInvocation};

use super::NoTaskParams;

pub const REFRESH_CONFIG_CACHE_TASK_KEY: &str = "system.refreshConfigCache";
pub const REFRESH_DICT_CACHE_TASK_KEY: &str = "system.refreshDictCache";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheRefreshKind {
    Config,
    Dict,
}

impl CacheRefreshKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Config => "config",
            Self::Dict => "dict",
        }
    }
}

pub fn cache_refresh_failure(kind: CacheRefreshKind, diagnostic: impl Into<String>) -> TaskExecutionFailure {
    TaskExecutionFailure::new(
        kernel::error::LocalizedError::new("errors.scheduler.task_cache_refresh_failed").with_param("kind", kind.as_str()),
        diagnostic,
    )
}

#[scheduled_task(
    task_key = REFRESH_CONFIG_CACHE_TASK_KEY,
    name_key = "scheduler.tasks.system.refresh_config_cache.name",
    group = "SYSTEM",
    group_key = "scheduler.task_groups.system",
    description_key = "scheduler.tasks.system.refresh_config_cache.description",
    repeatable = false,
    lifecycle = scheduler::application::task::TaskLifecyclePolicy::Administrable,
    params = NoTaskParams,
)]
#[derive(Default)]
pub struct RefreshConfigCacheTask;

#[async_trait]
impl ScheduledTask for RefreshConfigCacheTask {
    async fn execute(&self, context: TaskExecutionContext, _invocation: TaskInvocation) -> Result<TaskExecutionOutput, TaskExecutionFailure> {
        context.system_cache.refresh_config_cache().await?;
        Ok(TaskExecutionOutput::default())
    }
}

#[scheduled_task(
    task_key = REFRESH_DICT_CACHE_TASK_KEY,
    name_key = "scheduler.tasks.system.refresh_dict_cache.name",
    group = "SYSTEM",
    group_key = "scheduler.task_groups.system",
    description_key = "scheduler.tasks.system.refresh_dict_cache.description",
    repeatable = false,
    lifecycle = scheduler::application::task::TaskLifecyclePolicy::Administrable,
    params = NoTaskParams,
)]
#[derive(Default)]
pub struct RefreshDictCacheTask;

#[async_trait]
impl ScheduledTask for RefreshDictCacheTask {
    async fn execute(&self, context: TaskExecutionContext, _invocation: TaskInvocation) -> Result<TaskExecutionOutput, TaskExecutionFailure> {
        context.system_cache.refresh_dict_cache().await?;
        Ok(TaskExecutionOutput::default())
    }
}

#[cfg(test)]
mod tests {
    use super::{CacheRefreshKind, cache_refresh_failure};

    #[test]
    fn cache_refresh_failure_is_owned_by_scheduler() {
        let failure = cache_refresh_failure(CacheRefreshKind::Config, "database unavailable");

        assert_eq!(failure.public.key(), "errors.scheduler.task_cache_refresh_failed");
        assert_eq!(failure.public.params().len(), 1);
        assert_eq!(failure.public.params()[0].key(), "kind");
        assert_eq!(failure.public.params()[0].value(), "config");
        assert_eq!(failure.to_string(), "database unavailable");
    }
}
