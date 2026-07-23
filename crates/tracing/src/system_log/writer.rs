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
    run_writer_loop(&mut runtime).await;
    lifecycle.complete();
}

async fn run_writer_loop(runtime: &mut WriterRuntime) {
    let mut buffer = Vec::with_capacity(SYSTEM_LOG_BATCH_SIZE);
    let flush_deadline = sleep_until(Instant::now() + SYSTEM_LOG_FLUSH_INTERVAL);
    tokio::pin!(flush_deadline);
    loop {
        tokio::select! {
            changed = runtime.shutdown.changed() => {
                if changed.is_err() || *runtime.shutdown.borrow() {
                    runtime.receiver.close();
                    drain_writer(&mut runtime.receiver, &mut buffer, &runtime.context).await;
                    return;
                }
            }
            event = runtime.receiver.recv() => match event {
                Some(event) => {
                    runtime.context.state.record_dequeued();
                    if buffer.is_empty() {
                        flush_deadline.as_mut().reset(Instant::now() + SYSTEM_LOG_FLUSH_INTERVAL);
                    }
                    push_event(&mut buffer, event, &runtime.context).await;
                }
                None => {
                    flush(&mut buffer, &runtime.context).await;
                    return;
                }
            },
            _ = &mut flush_deadline, if !buffer.is_empty() => flush(&mut buffer, &runtime.context).await,
        }
    }
}

async fn drain_writer(receiver: &mut mpsc::Receiver<SystemLogEvent>, buffer: &mut Vec<SystemLogEvent>, context: &WriterContext) {
    while let Some(event) = receiver.recv().await {
        context.state.record_dequeued();
        push_event(buffer, event, context).await;
    }
    flush(buffer, context).await;
}

async fn push_event(buffer: &mut Vec<SystemLogEvent>, event: SystemLogEvent, context: &WriterContext) {
    buffer.push(event);
    if buffer.len() >= SYSTEM_LOG_BATCH_SIZE {
        flush(buffer, context).await;
    }
}

async fn flush(buffer: &mut Vec<SystemLogEvent>, context: &WriterContext) {
    if buffer.is_empty() {
        return;
    }
    let events = std::mem::replace(buffer, Vec::with_capacity(SYSTEM_LOG_BATCH_SIZE));
    let event_count = u64::try_from(events.len()).expect("system log batch length must fit in u64");
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
