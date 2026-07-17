use std::sync::Arc;

use arc_swap::ArcSwap;

use crate::RuntimeTracingConfig;

/// Immutable tracing configuration snapshot shared by every runtime sink.
#[derive(Clone)]
pub struct RuntimeTracingState {
    config: Arc<ArcSwap<RuntimeTracingConfig>>,
}

impl RuntimeTracingState {
    pub fn new(config: RuntimeTracingConfig) -> Self {
        Self {
            config: Arc::new(ArcSwap::from_pointee(config)),
        }
    }

    pub fn current(&self) -> Arc<RuntimeTracingConfig> {
        self.config.load_full()
    }

    pub fn reload(&self, config: RuntimeTracingConfig) {
        self.config.store(Arc::new(config));
    }

    pub(crate) fn allows(&self, level: &tracing::Level) -> bool {
        self.current().log_level.allows(level)
    }
}
