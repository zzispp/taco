use std::sync::Arc;

use tokio::{
    sync::{mpsc, watch},
    time::interval,
};

use super::{SystemLogEvent, SystemLogSink};
use super::{
    emitter::{SYSTEM_LOG_BATCH_SIZE, SYSTEM_LOG_FLUSH_INTERVAL},
    ingestion_state::IngestionState,
};

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
    let mut buffer = Vec::with_capacity(SYSTEM_LOG_BATCH_SIZE);
    let mut ticker = interval(SYSTEM_LOG_FLUSH_INTERVAL);
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
                Some(event) => push_event(&mut buffer, event, &runtime.context).await,
                None => {
                    flush(&mut buffer, &runtime.context).await;
                    return;
                }
            },
            _ = ticker.tick(), if !buffer.is_empty() => flush(&mut buffer, &runtime.context).await,
        }
    }
}

async fn drain_writer(receiver: &mut mpsc::Receiver<SystemLogEvent>, buffer: &mut Vec<SystemLogEvent>, context: &WriterContext) {
    while let Some(event) = receiver.recv().await {
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
    match context.sink.insert_batch(events).await {
        Ok(()) => context.state.record_persisted(event_count),
        Err(error) => log_write_failure(context.state.record_write_failure(event_count, &error)),
    }
}

fn log_write_failure(reason: &'static str) {
    crate::__tracing::error!(target: "taco.internal.system_log_writer", failure_reason = reason, "system log writer batch failed");
}
