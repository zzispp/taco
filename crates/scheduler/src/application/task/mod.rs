mod catalog;
mod contract;
mod ports;
mod validation;

pub use catalog::{StaticTaskCatalog, TaskCatalog};
pub use contract::{
    ParamDefinition, ScheduledTask, ScheduledTaskDefinition, ScheduledTaskMetadata, TaskExecutionContext, TaskExecutionDetailPayload, TaskExecutionFailure,
    TaskExecutionOutput, TaskInvocation, TaskLifecycleCapabilities, TaskLifecyclePolicy, TaskParams,
};
pub use ports::{
    HttpFailureCode, HttpTaskClient, OutboundHttpFailure, OutboundHttpHeader, OutboundHttpRequest, OutboundHttpResponse, OutboundHttpResponseHead,
    SystemCacheRefreshPort, SystemLogCleanupFilter, SystemLogCleanupLevel, SystemLogCleanupPort, SystemLogCleanupResult,
};
pub use validation::{invalid_task_params, validate_param_enum, validate_param_object_keys, validate_param_pattern, validate_required_param_fields};
