use async_trait::async_trait;
use scheduler_macros::scheduled_task;

use crate::application::task::{ScheduledTask, TaskExecutionContext, TaskExecutionFailure, TaskExecutionOutput, TaskInvocation};

use super::NoTaskParams;

pub const REFRESH_CONFIG_CACHE_TASK_KEY: &str = "system.refreshConfigCache";
pub const REFRESH_DICT_CACHE_TASK_KEY: &str = "system.refreshDictCache";

#[scheduled_task(
    task_key = REFRESH_CONFIG_CACHE_TASK_KEY,
    name_key = "scheduler.tasks.system.refresh_config_cache.name",
    group = "SYSTEM",
    group_key = "scheduler.task_groups.system",
    description_key = "scheduler.tasks.system.refresh_config_cache.description",
    repeatable = false,
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
