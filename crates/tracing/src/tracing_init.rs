use std::sync::OnceLock;

use thiserror::Error;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

static GLOBAL_SUBSCRIBER_INITIALIZED: OnceLock<()> = OnceLock::new();

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TracingConfig {
    pub log_level: String,
    pub file_logging_enabled: bool,
    pub file_directory: String,
    pub file_prefix: String,
}

#[derive(Debug, Error)]
pub enum TracingInitError {
    #[error("invalid tracing filter: {0}")]
    InvalidFilter(#[from] tracing_subscriber::filter::ParseError),
    #[error("{0} cannot be blank when file logging is enabled")]
    BlankFileSetting(&'static str),
}

pub fn init_global_subscriber(config: TracingConfig) -> Result<Option<WorkerGuard>, TracingInitError> {
    let filter = EnvFilter::try_new(config.log_level.clone())?;
    let mut guard = None;

    GLOBAL_SUBSCRIBER_INITIALIZED.get_or_init(|| {
        let stdout_layer = tracing_subscriber::fmt::layer().with_target(false);
        if config.file_logging_enabled {
            validate_file_settings(&config).expect("file logging settings should already be validated");
            let file_appender = tracing_appender::rolling::daily(&config.file_directory, &config.file_prefix);
            let (non_blocking, worker_guard) = tracing_appender::non_blocking(file_appender);
            let file_layer = tracing_subscriber::fmt::layer().with_target(false).with_ansi(false).with_writer(non_blocking);
            guard = Some(worker_guard);
            tracing_subscriber::registry().with(filter).with(stdout_layer).with(file_layer).init();
            return;
        }

        tracing_subscriber::registry().with(filter).with(stdout_layer).init();
    });

    Ok(guard)
}

fn validate_file_settings(config: &TracingConfig) -> Result<(), TracingInitError> {
    if config.file_directory.trim().is_empty() {
        return Err(TracingInitError::BlankFileSetting("tracing.file.directory"));
    }
    if config.file_prefix.trim().is_empty() {
        return Err(TracingInitError::BlankFileSetting("tracing.file.prefix"));
    }
    Ok(())
}
