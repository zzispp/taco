use std::{sync::Arc, time::Instant as StdInstant};

use tokio::{
    sync::{mpsc, watch},
    time::{Instant, sleep_until},
};

use super::{SystemLogEvent, SystemLogSink};
use super::{
    emitter::{SYSTEM_LOG_BATCH_SIZE, SYSTEM_LOG_FLUSH_INTERVAL},
    ingestion_state::IngestionState,
};

const SINK_WRITE_DURATION_METRIC: &str = "system_log_sink_write_duration_seconds";
const SINK_WRITE_FAILURE: &str = "failure";
const SINK_WRITE_SUCCESS: &str = "success";
const BATCH_COUNT_OVERFLOW_DROP_REASON: &str = "batch_count_overflow";

pub(super) struct WriterRuntime {
    pub(super) receiver: mpsc::Receiver<SystemLogEvent>,
    pub(super) context: WriterContext,
    pub(super) shutdown: watch::Receiver<bool>,
}

pub(super) struct WriterContext {
    sink: Arc<dyn SystemLogSink>,
    state: Arc<IngestionState>,
}

impl WriterContext {
    pub(super) fn new(sink: Arc<dyn SystemLogSink>, state: Arc<IngestionState>) -> Self {
        Self { sink, state }
    }
}

pub(super) async fn run_writer(mut runtime: WriterRuntime) {
    let mut lifecycle = WriterLifecycle::new(runtime.context.state.clone());
    if run_writer_loop(&mut runtime).await {
        lifecycle.complete();
    }
}

async fn run_writer_loop(runtime: &mut WriterRuntime) -> bool {
    let mut buffer = Vec::with_capacity(SYSTEM_LOG_BATCH_SIZE);
    let flush_deadline = sleep_until(Instant::now() + SYSTEM_LOG_FLUSH_INTERVAL);
    tokio::pin!(flush_deadline);
    loop {
        tokio::select! {
            changed = runtime.shutdown.changed() => {
                if changed.is_err() || *runtime.shutdown.borrow() {
                    runtime.receiver.close();
                    return drain_writer(&mut runtime.receiver, &mut buffer, &runtime.context).await;
                }
            }
            event = runtime.receiver.recv() => match event {
                Some(event) => {
                    runtime.context.state.record_dequeued();
                    if buffer.is_empty() {
                        flush_deadline.as_mut().reset(Instant::now() + SYSTEM_LOG_FLUSH_INTERVAL);
                    }
                    if !push_event(&mut buffer, event, &runtime.context).await {
                        return false;
                    }
                }
                None => {
                    return flush(&mut buffer, &runtime.context).await;
                }
            },
            _ = &mut flush_deadline, if !buffer.is_empty() => {
                if !flush(&mut buffer, &runtime.context).await {
                    return false;
                }
            }
        }
    }
}

async fn drain_writer(receiver: &mut mpsc::Receiver<SystemLogEvent>, buffer: &mut Vec<SystemLogEvent>, context: &WriterContext) -> bool {
    while let Some(event) = receiver.recv().await {
        context.state.record_dequeued();
        if !push_event(buffer, event, context).await {
            return false;
        }
    }
    flush(buffer, context).await
}

async fn push_event(buffer: &mut Vec<SystemLogEvent>, event: SystemLogEvent, context: &WriterContext) -> bool {
    buffer.push(event);
    if buffer.len() >= SYSTEM_LOG_BATCH_SIZE {
        return flush(buffer, context).await;
    }
    true
}

async fn flush(buffer: &mut Vec<SystemLogEvent>, context: &WriterContext) -> bool {
    if buffer.is_empty() {
        return true;
    }
    let events = std::mem::replace(buffer, Vec::with_capacity(SYSTEM_LOG_BATCH_SIZE));
    let event_count = match u64::try_from(events.len()) {
        Ok(count) => count,
        Err(error) => {
            context.state.mark_writer_failure(BATCH_COUNT_OVERFLOW_DROP_REASON);
            crate::__tracing::error!(target: "taco.internal.system_log_writer", %error, "system log batch size exceeds the supported event counter range");
            return false;
        }
    };
    let started_at = StdInstant::now();
    let result = context.sink.insert_batch(events).await;
    match result {
        Ok(()) => {
            record_sink_duration(started_at, SINK_WRITE_SUCCESS);
            context.state.record_persisted(event_count);
        }
        Err(error) => {
            record_sink_duration(started_at, SINK_WRITE_FAILURE);
            log_write_failure(context.state.record_write_failure(event_count, &error));
        }
    }
    true
}

fn record_sink_duration(started_at: StdInstant, outcome: &'static str) {
    metrics::histogram!(SINK_WRITE_DURATION_METRIC, "outcome" => outcome).record(started_at.elapsed().as_secs_f64());
}

fn log_write_failure(reason: &'static str) {
    crate::__tracing::error!(target: "taco.internal.system_log_writer", failure_reason = reason, "system log writer batch failed");
}

struct WriterLifecycle {
    state: Arc<IngestionState>,
    completed: bool,
}

impl WriterLifecycle {
    fn new(state: Arc<IngestionState>) -> Self {
        Self { state, completed: false }
    }

    fn complete(&mut self) {
        self.completed = true;
    }
}

impl Drop for WriterLifecycle {
    fn drop(&mut self) {
        self.state.record_writer_stopped(self.completed);
    }
}
