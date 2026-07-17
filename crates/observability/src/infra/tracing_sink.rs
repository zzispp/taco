use std::sync::Arc;

use async_trait::async_trait;
use taco_tracing::{SystemLogEvent, SystemLogLevel as CapturedSystemLogLevel, SystemLogSink};

use crate::{
    application::SystemLogRepository,
    domain::{NewSystemLog, SystemLogLevel},
};

#[derive(Clone)]
pub struct ObservabilitySystemLogSink {
    repository: Arc<dyn SystemLogRepository>,
}

impl ObservabilitySystemLogSink {
    pub fn new(repository: Arc<dyn SystemLogRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl SystemLogSink for ObservabilitySystemLogSink {
    async fn insert_batch(&self, events: Vec<SystemLogEvent>) -> Result<(), String> {
        let events = events.into_iter().map(map_event).collect::<Vec<_>>();
        self.repository.insert_batch(&events).await.map_err(|error| error.to_string())
    }
}

fn map_event(event: SystemLogEvent) -> NewSystemLog {
    NewSystemLog {
        id: event.id,
        occurred_at: event.occurred_at,
        level: map_level(event.level),
        target: event.target,
        message: event.message,
        fields: event.fields,
    }
}

fn map_level(value: CapturedSystemLogLevel) -> SystemLogLevel {
    match value {
        CapturedSystemLogLevel::Trace => SystemLogLevel::Trace,
        CapturedSystemLogLevel::Debug => SystemLogLevel::Debug,
        CapturedSystemLogLevel::Info => SystemLogLevel::Info,
        CapturedSystemLogLevel::Warn => SystemLogLevel::Warn,
        CapturedSystemLogLevel::Error => SystemLogLevel::Error,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use taco_tracing::{SystemLogEvent, SystemLogEventInput, SystemLogLevel as CapturedSystemLogLevel};

    use crate::domain::SystemLogLevel;

    use super::{map_event, map_level};

    #[test]
    fn captured_events_map_without_losing_timestamp_or_fields() {
        let event = SystemLogEvent::new(SystemLogEventInput {
            occurred_at: time::OffsetDateTime::UNIX_EPOCH,
            level: CapturedSystemLogLevel::Warn,
            target: "http".into(),
            message: "slow request".into(),
            fields: serde_json::Map::from_iter([("duration_ms".into(), json!(700))]),
        });

        let mapped = map_event(event);

        assert_eq!(mapped.level, SystemLogLevel::Warn);
        assert_eq!(mapped.occurred_at, time::OffsetDateTime::UNIX_EPOCH);
        assert_eq!(mapped.fields, json!({"duration_ms": 700}));
        assert_eq!(map_level(CapturedSystemLogLevel::Error), SystemLogLevel::Error);
    }
}
