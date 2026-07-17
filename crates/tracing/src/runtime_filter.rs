use tracing::{Metadata, Subscriber, subscriber::Interest};
use tracing_subscriber::{Layer, layer::Context};

use crate::RuntimeTracingState;

pub(crate) struct RuntimeLevelFilter {
    state: RuntimeTracingState,
}

impl RuntimeLevelFilter {
    pub(crate) fn new(state: RuntimeTracingState) -> Self {
        Self { state }
    }
}

impl<S> Layer<S> for RuntimeLevelFilter
where
    S: Subscriber,
{
    fn register_callsite(&self, _: &'static Metadata<'static>) -> Interest {
        Interest::sometimes()
    }

    fn enabled(&self, metadata: &Metadata<'_>, _: Context<'_, S>) -> bool {
        self.state.allows(metadata.level())
    }
}
