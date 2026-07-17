use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use async_trait::async_trait;
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
    time::timeout,
};

use crate::RuntimeTracingState;

use super::{SystemLogEvent, SystemLogLevel};
use super::{
    ingestion_state::{IngestionState, SystemLogIngestionStatus},
    writer::{WriterContext, WriterRuntime, run_writer},
};

pub const SYSTEM_LOG_CHANNEL_CAPACITY: usize = 512;
pub const SYSTEM_LOG_BATCH_SIZE: usize = 100;
pub const SYSTEM_LOG_FLUSH_INTERVAL: Duration = Duration::from_millis(100);
pub const SYSTEM_LOG_EVENT_MAX_BYTES: usize = 128 * 1024;
pub const SYSTEM_LOG_SHUTDOWN_DRAIN_TIMEOUT: Duration = Duration::from_secs(5);

/// Persists batches without blocking business event producers.
#[async_trait]
pub trait SystemLogSink: Send + Sync + 'static {
    async fn insert_batch(&self, events: Vec<SystemLogEvent>) -> Result<(), String>;
}

#[derive(Clone)]
pub struct SystemLogEmitter {
    sender: mpsc::Sender<SystemLogEvent>,
    level: SystemLogLevelSource,
    state: Arc<IngestionState>,
}

#[derive(Clone)]
enum SystemLogLevelSource {
    Fixed(Arc<std::sync::atomic::AtomicU8>),
    Runtime(RuntimeTracingState),
}

impl SystemLogEmitter {
    pub fn enabled(&self, level: SystemLogLevel) -> bool {
        level.priority() >= self.minimum_level()
    }

    pub fn emit(&self, event: SystemLogEvent) {
        let size = match event.serialized_size() {
            Ok(size) => size,
            Err(_) => return self.state.record_drop("serialization_failed", 1),
        };
        if size > SYSTEM_LOG_EVENT_MAX_BYTES {
            return self.state.record_drop("oversize", 1);
        }
        // Reserve before publishing so a fast writer cannot complete this event
        // before it is included in pending accounting.
        self.state.record_accepted();
        if let Err(error) = self.sender.try_send(event) {
            self.state.record_send_failure(match error {
                mpsc::error::TrySendError::Full(_) => "queue_full",
                mpsc::error::TrySendError::Closed(_) => "queue_closed",
            });
        }
    }

    fn minimum_level(&self) -> u8 {
        match &self.level {
            SystemLogLevelSource::Fixed(level) => level.load(std::sync::atomic::Ordering::Relaxed),
            SystemLogLevelSource::Runtime(state) => state.current().log_level.priority(),
        }
    }
}

pub struct SystemLogRuntime {
    emitter: SystemLogEmitter,
    shutdown: watch::Sender<bool>,
    writer: Mutex<Option<JoinHandle<()>>>,
}

impl SystemLogRuntime {
    pub fn emitter(&self) -> SystemLogEmitter {
        self.emitter.clone()
    }

    pub fn status(&self) -> SystemLogIngestionStatus {
        self.emitter.state.status()
    }

    pub async fn shutdown(&self) {
        self.request_shutdown();
        let Some(mut writer) = self.writer.lock().unwrap().take() else {
            return;
        };
        match timeout(SYSTEM_LOG_SHUTDOWN_DRAIN_TIMEOUT, &mut writer).await {
            Ok(Ok(())) => {}
            Ok(Err(_)) => self.emitter.state.discard_all_pending("writer_cancelled"),
            Err(_) => {
                writer.abort();
                let _ = writer.await;
                self.emitter.state.discard_all_pending("shutdown_timeout");
            }
        }
    }

    fn request_shutdown(&self) {
        self.shutdown.send_replace(true);
    }
}

impl Drop for SystemLogRuntime {
    fn drop(&mut self) {
        self.request_shutdown();
    }
}

pub fn start_system_log_runtime(sink: Arc<dyn SystemLogSink>, min_level: SystemLogLevel) -> SystemLogRuntime {
    let (sender, receiver) = mpsc::channel(SYSTEM_LOG_CHANNEL_CAPACITY);
    let emitter = SystemLogEmitter {
        sender,
        level: SystemLogLevelSource::Fixed(Arc::new(std::sync::atomic::AtomicU8::new(min_level.priority()))),
        state: Arc::new(IngestionState::default()),
    };
    runtime(emitter, receiver, sink)
}

pub fn start_system_log_runtime_with_state(sink: Arc<dyn SystemLogSink>, state: RuntimeTracingState) -> SystemLogRuntime {
    let (sender, receiver) = mpsc::channel(SYSTEM_LOG_CHANNEL_CAPACITY);
    let emitter = SystemLogEmitter {
        sender,
        level: SystemLogLevelSource::Runtime(state),
        state: Arc::new(IngestionState::default()),
    };
    runtime(emitter, receiver, sink)
}

fn runtime(emitter: SystemLogEmitter, receiver: mpsc::Receiver<SystemLogEvent>, sink: Arc<dyn SystemLogSink>) -> SystemLogRuntime {
    let (shutdown, shutdown_receiver) = watch::channel(false);
    let writer = tokio::spawn(run_writer(WriterRuntime {
        receiver,
        context: WriterContext::new(sink, emitter.state.clone()),
        shutdown: shutdown_receiver,
    }));
    SystemLogRuntime {
        emitter,
        shutdown,
        writer: Mutex::new(Some(writer)),
    }
}
