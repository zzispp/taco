use std::sync::Arc;

use async_trait::async_trait;
use kernel::error::LocalizedError;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::domain::{ExecutionDetail, TaskParamFormSpec};

use super::{FileCleanupPort, HttpTaskClient, SystemCacheRefreshPort, SystemLogCleanupPort};

const TASK_DETAIL_ENCODING_ERROR_KEY: &str = "errors.scheduler.task_detail_encoding_failed";

#[derive(Clone)]
pub struct TaskExecutionContext {
    pub http_client: Arc<dyn HttpTaskClient>,
    pub system_cache: Arc<dyn SystemCacheRefreshPort>,
    pub system_log_cleanup: Arc<dyn SystemLogCleanupPort>,
    pub file_cleanup: Arc<dyn FileCleanupPort>,
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

    fn into_execution_detail(self) -> Result<ExecutionDetail, TaskExecutionFailure>
    where
        Self: Sized,
    {
        if Self::SCHEMA_VERSION <= 0 {
            return Err(task_detail_encoding_failure("task execution detail schema version must be positive"));
        }
        let value = serde_json::to_value(self).map_err(|error| task_detail_encoding_failure(format!("task execution detail serialization failed: {error}")))?;
        let Value::Object(payload) = value else {
            return Err(task_detail_encoding_failure("task execution detail payload must serialize as an object"));
        };
        Ok(ExecutionDetail::new(Self::KIND, Self::SCHEMA_VERSION, payload))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TaskExecutionOutput {
    pub detail: Option<ExecutionDetail>,
}

impl TaskExecutionOutput {
    pub fn with_detail(payload: impl TaskExecutionDetailPayload) -> Result<Self, TaskExecutionFailure> {
        Ok(Self {
            detail: Some(payload.into_execution_detail()?),
        })
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

    pub fn with_detail(mut self, payload: impl TaskExecutionDetailPayload) -> Result<Self, TaskExecutionFailure> {
        let detail = payload.into_execution_detail()?;
        self.detail = Some(Box::new(detail));
        Ok(self)
    }
}

fn task_detail_encoding_failure(diagnostic: impl Into<String>) -> TaskExecutionFailure {
    TaskExecutionFailure::new(LocalizedError::new(TASK_DETAIL_ENCODING_ERROR_KEY), diagnostic)
}

#[derive(Clone, Copy)]
pub struct ParamDefinition {
    pub schema_version: i16,
    pub form: fn() -> TaskParamFormSpec,
    pub default_params: fn() -> Value,
    pub validate: fn(&Value) -> crate::application::SchedulerResult<()>,
    pub validate_persisted: fn(&Value) -> crate::application::SchedulerResult<()>,
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
    pub lifecycle: TaskLifecyclePolicy,
    pub params: ParamDefinition,
    pub factory: fn() -> Arc<dyn ScheduledTask>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskLifecyclePolicy {
    Administrable,
    RequiredPausable,
    RequiredEnabled,
}

impl TaskLifecyclePolicy {
    pub const fn capabilities(self) -> TaskLifecycleCapabilities {
        match self {
            Self::Administrable => TaskLifecycleCapabilities::ADMINISTRABLE,
            Self::RequiredPausable => TaskLifecycleCapabilities::REQUIRED_PAUSABLE,
            Self::RequiredEnabled => TaskLifecycleCapabilities::REQUIRED_ENABLED,
        }
    }

    pub const fn can_disable(self) -> bool {
        self.capabilities().can_disable
    }

    pub const fn can_delete(self) -> bool {
        self.capabilities().can_delete
    }

    pub const fn can_edit_execution_policy(self) -> bool {
        self.capabilities().can_edit_execution_policy
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskLifecycleCapabilities {
    pub can_disable: bool,
    pub can_delete: bool,
    pub can_edit_execution_policy: bool,
}

impl TaskLifecycleCapabilities {
    pub const ADMINISTRABLE: Self = Self {
        can_disable: true,
        can_delete: true,
        can_edit_execution_policy: true,
    };

    pub const REQUIRED_ENABLED: Self = Self {
        can_disable: false,
        can_delete: false,
        can_edit_execution_policy: false,
    };

    pub const REQUIRED_PAUSABLE: Self = Self {
        can_disable: true,
        can_delete: false,
        can_edit_execution_policy: true,
    };
}

pub trait ScheduledTaskMetadata {
    fn descriptor() -> ScheduledTaskDefinition;
}

pub trait TaskParams: Send + Sync + 'static {
    const SCHEMA_VERSION: i16;
    fn form() -> TaskParamFormSpec;
    fn default_params() -> Value;
    fn validate(value: &Value) -> crate::application::SchedulerResult<()>;
    fn validate_persisted(value: &Value) -> crate::application::SchedulerResult<()> {
        Self::validate(value)
    }
    fn render_invoke_target(task_key: &str, value: &Value) -> crate::application::SchedulerResult<String>;
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::{TaskExecutionDetailPayload, TaskExecutionOutput};

    #[test]
    fn serialization_failure_becomes_a_typed_task_failure() {
        let error = TaskExecutionOutput::with_detail(SerializationFailure).unwrap_err();

        assert_eq!(error.public.key(), "errors.scheduler.task_detail_encoding_failed");
        assert!(error.to_string().contains("planned serialization failure"));
    }

    #[test]
    fn non_object_payload_becomes_a_typed_task_failure() {
        let error = TaskExecutionOutput::with_detail(ArrayPayload).unwrap_err();

        assert_eq!(error.public.key(), "errors.scheduler.task_detail_encoding_failed");
        assert_eq!(error.to_string(), "task execution detail payload must serialize as an object");
    }

    struct SerializationFailure;

    impl Serialize for SerializationFailure {
        fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Err(serde::ser::Error::custom("planned serialization failure"))
        }
    }

    impl TaskExecutionDetailPayload for SerializationFailure {
        const KIND: &'static str = "test";
        const SCHEMA_VERSION: i16 = 1;
    }

    struct ArrayPayload;

    impl Serialize for ArrayPayload {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            ["array"].serialize(serializer)
        }
    }

    impl TaskExecutionDetailPayload for ArrayPayload {
        const KIND: &'static str = "test";
        const SCHEMA_VERSION: i16 = 1;
    }
}
