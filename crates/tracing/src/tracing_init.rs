use thiserror::Error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{RuntimeTracingConfig, RuntimeTracingState, SystemLogLayer, runtime_filter::RuntimeLevelFilter};

#[derive(Clone)]
pub struct ReloadableTracing {
    state: RuntimeTracingState,
}

#[derive(Debug, Error)]
pub enum TracingInitError {
    #[error("failed to install global tracing subscriber: {0}")]
    Install(#[from] tracing_subscriber::util::TryInitError),
}

impl ReloadableTracing {
    pub fn reload(&self, config: RuntimeTracingConfig) {
        self.state.reload(config);
    }
}

pub fn init_global_subscriber(state: RuntimeTracingState, system_logs: SystemLogLayer) -> Result<ReloadableTracing, TracingInitError> {
    tracing_subscriber::registry()
        .with(RuntimeLevelFilter::new(state.clone()))
        .with(tracing_subscriber::fmt::layer().with_target(true))
        .with(system_logs)
        .try_init()?;
    Ok(ReloadableTracing { state })
}
