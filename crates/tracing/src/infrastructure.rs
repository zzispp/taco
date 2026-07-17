use std::{future::Future, time::Duration};

use crate::{DurationMs, RuntimeTracingState};

/// Dependency categories with independently reloadable slow-operation thresholds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InfrastructureDependency {
    Postgres,
    Redis,
    OutboundHttp,
}

impl InfrastructureDependency {
    const fn code(self) -> &'static str {
        match self {
            Self::Postgres => "postgres",
            Self::Redis => "redis",
            Self::OutboundHttp => "outbound_http",
        }
    }
}

/// Emits only failed or threshold-exceeding dependency operations using safe metadata.
#[derive(Clone)]
pub struct InfrastructureObserver {
    state: RuntimeTracingState,
}

impl InfrastructureObserver {
    pub fn new(state: RuntimeTracingState) -> Self {
        Self { state }
    }

    pub async fn observe<T, E, F>(&self, dependency: InfrastructureDependency, operation: &'static str, future: F) -> Result<T, E>
    where
        F: Future<Output = Result<T, E>>,
    {
        let started = std::time::Instant::now();
        let result = future.await;
        self.record(dependency, operation, started.elapsed(), result.is_ok());
        result
    }

    pub fn record(&self, dependency: InfrastructureDependency, operation: &'static str, elapsed: Duration, succeeded: bool) {
        let threshold = threshold(&self.state, dependency);
        if !succeeded {
            return crate::error_with_fields!(
                "infrastructure operation failed",
                dependency = dependency.code(),
                operation = operation,
                duration = DurationMs(elapsed)
            );
        }
        if elapsed >= threshold {
            crate::warn_with_fields!(
                "slow infrastructure operation",
                dependency = dependency.code(),
                operation = operation,
                duration = DurationMs(elapsed),
                threshold = DurationMs(threshold)
            );
        }
    }
}

fn threshold(state: &RuntimeTracingState, dependency: InfrastructureDependency) -> Duration {
    let thresholds = &state.current().slow_operation_ms;
    let milliseconds = match dependency {
        InfrastructureDependency::Postgres => thresholds.postgres,
        InfrastructureDependency::Redis => thresholds.redis,
        InfrastructureDependency::OutboundHttp => thresholds.outbound_http,
    };
    Duration::from_millis(milliseconds)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{HttpLogCaptureConfig, RuntimeTracingConfig, RuntimeTracingState, SlowOperationThresholds, TracingLevel};

    use super::{InfrastructureDependency, threshold};

    #[test]
    fn thresholds_follow_the_current_runtime_snapshot() {
        let state = RuntimeTracingState::new(config(500));
        assert_eq!(threshold(&state, InfrastructureDependency::Postgres), Duration::from_millis(500));

        state.reload(config(25));

        assert_eq!(threshold(&state, InfrastructureDependency::Postgres), Duration::from_millis(25));
    }

    fn config(postgres_threshold: u64) -> RuntimeTracingConfig {
        RuntimeTracingConfig {
            log_level: TracingLevel::Info,
            http: HttpLogCaptureConfig {
                access_enabled: true,
                capture_request_body: false,
                capture_response_body: false,
                capture_query_parameters: false,
                capture_request_headers: false,
                max_body_capture_bytes: 0,
            },
            slow_operation_ms: SlowOperationThresholds {
                postgres: postgres_threshold,
                redis: 100,
                outbound_http: 1_000,
            },
        }
    }
}
