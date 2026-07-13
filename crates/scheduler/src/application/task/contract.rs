use std::sync::Arc;

use async_trait::async_trait;
use kernel::error::LocalizedError;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::domain::{ExecutionDetail, TaskParamFormSpec};

use super::{HttpTaskClient, SystemCacheRefreshPort};

#[derive(Clone)]
pub struct TaskExecutionContext {
    pub http_client: Arc<dyn HttpTaskClient>,
    pub system_cache: Arc<dyn SystemCacheRefreshPort>,
}

#[derive(Clone, Debug)]
pub struct TaskInvocation {
    pub execution_id: String,
    pub job_id: String,
    pub task_key: String,
    pub task_params: Value,
    pub invoke_target: String,
}

impl TaskInvocation {
    pub fn decode_params<T: DeserializeOwned>(&self) -> Result<T, TaskExecutionFailure> {
        serde_json::from_value(self.task_params.clone()).map_err(|error| {
            TaskExecutionFailure::new(
                LocalizedError::new("errors.scheduler.invalid_params"),
                format!("failed to decode task parameters: {error}"),
            )
        })
    }
}

#[async_trait]
pub trait ScheduledTask: Send + Sync + 'static {
    async fn execute(&self, context: TaskExecutionContext, invocation: TaskInvocation) -> Result<TaskExecutionOutput, TaskExecutionFailure>;
}

pub trait TaskExecutionDetailPayload: Serialize {
    const KIND: &'static str;
    const SCHEMA_VERSION: i16;

    fn into_execution_detail(self) -> ExecutionDetail
    where
        Self: Sized,
    {
        assert!(Self::SCHEMA_VERSION > 0, "task execution detail schema version must be positive");
        let value = serde_json::to_value(self).expect("task execution detail payload serialization must succeed");
        let Value::Object(payload) = value else {
            panic!("task execution detail payload must serialize as an object");
        };
        ExecutionDetail::new(Self::KIND, Self::SCHEMA_VERSION, payload)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TaskExecutionOutput {
    pub detail: Option<ExecutionDetail>,
}

impl TaskExecutionOutput {
    pub fn with_detail(payload: impl TaskExecutionDetailPayload) -> Self {
        Self {
            detail: Some(payload.into_execution_detail()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{diagnostic}")]
pub struct TaskExecutionFailure {
    pub public: LocalizedError,
    diagnostic: String,
    pub detail: Option<Box<ExecutionDetail>>,
}

impl TaskExecutionFailure {
    pub fn new(public: LocalizedError, diagnostic: impl Into<String>) -> Self {
        Self {
            public,
            diagnostic: diagnostic.into(),
            detail: None,
        }
    }

    pub fn with_detail(mut self, payload: impl TaskExecutionDetailPayload) -> Self {
        self.detail = Some(Box::new(payload.into_execution_detail()));
        self
    }
}

#[derive(Clone, Copy)]
pub struct ParamDefinition {
    pub schema_version: i16,
    pub form: fn() -> TaskParamFormSpec,
    pub default_params: fn() -> Value,
    pub validate: fn(&Value) -> crate::application::SchedulerResult<()>,
    pub render_invoke_target: fn(&str, &Value) -> crate::application::SchedulerResult<String>,
}

#[derive(Clone, Copy)]
pub struct ScheduledTaskDefinition {
    pub task_key: &'static str,
    pub name_key: &'static str,
    pub group: &'static str,
    pub group_key: &'static str,
    pub description_key: &'static str,
    pub repeatable: bool,
    pub params: ParamDefinition,
    pub factory: fn() -> Arc<dyn ScheduledTask>,
}

pub trait ScheduledTaskMetadata {
    fn descriptor() -> ScheduledTaskDefinition;
}

pub trait TaskParams: Send + Sync + 'static {
    const SCHEMA_VERSION: i16;
    fn form() -> TaskParamFormSpec;
    fn default_params() -> Value;
    fn validate(value: &Value) -> crate::application::SchedulerResult<()>;
    fn render_invoke_target(task_key: &str, value: &Value) -> crate::application::SchedulerResult<String>;
}
