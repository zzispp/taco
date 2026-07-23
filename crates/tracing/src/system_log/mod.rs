mod emitter;
#[cfg(test)]
mod emitter_tests;
mod event;
mod ingestion_state;
#[cfg(test)]
mod ingestion_state_tests;
mod layer;
#[cfg(test)]
mod runtime_tests;
mod writer;
#[cfg(test)]
mod writer_exit_tests;

pub use emitter::{
    SYSTEM_LOG_BATCH_SIZE, SYSTEM_LOG_CHANNEL_CAPACITY, SYSTEM_LOG_EVENT_MAX_BYTES, SYSTEM_LOG_FLUSH_INTERVAL, SYSTEM_LOG_SHUTDOWN_DRAIN_TIMEOUT,
    SystemLogEmitter, SystemLogRuntime, SystemLogSink, start_system_log_runtime, start_system_log_runtime_with_state,
};
pub use event::{SystemLogEvent, SystemLogEventInput, SystemLogLevel};
pub use ingestion_state::{SystemLogDeliveryGuarantee, SystemLogIngestionStatus, SystemLogWriteFailure};
pub use layer::SystemLogLayer;
