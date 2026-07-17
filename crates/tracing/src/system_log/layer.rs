use serde_json::{Map, Value};
use tracing::{Event, Subscriber, field::Visit};
use tracing_subscriber::{Layer, layer::Context};

use kernel::redaction::redact_json;

use super::{SystemLogEmitter, SystemLogEvent, SystemLogEventInput, SystemLogLevel};

const SYSTEM_LOG_ADMISSION_FIELD: &str = "__taco_system_log";

#[derive(Clone)]
pub struct SystemLogLayer {
    emitter: SystemLogEmitter,
}

impl SystemLogLayer {
    pub fn new(emitter: SystemLogEmitter) -> Self {
        Self { emitter }
    }

    pub fn emitter(&self) -> SystemLogEmitter {
        self.emitter.clone()
    }
}

impl<S> Layer<S> for SystemLogLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _context: Context<'_, S>) {
        let metadata = event.metadata();
        let mut visitor = EventVisitor::default();
        event.record(&mut visitor);
        if !visitor.admitted {
            return;
        }
        let level = SystemLogLevel::from_tracing(metadata.level());
        if !self.emitter.enabled(level) {
            return;
        }
        let message = visitor.message.unwrap_or_else(|| metadata.name().into());
        let mut fields = Value::Object(visitor.fields);
        redact_json(&mut fields);
        let Value::Object(fields) = fields else {
            unreachable!("event fields must remain a JSON object");
        };
        self.emitter.emit(SystemLogEvent::new(SystemLogEventInput {
            occurred_at: time::OffsetDateTime::now_utc(),
            level,
            target: metadata.target().into(),
            message,
            fields,
        }));
    }
}

#[derive(Default)]
struct EventVisitor {
    admitted: bool,
    message: Option<String>,
    fields: Map<String, Value>,
}

impl EventVisitor {
    fn record_value(&mut self, field: &tracing::field::Field, value: Value) {
        if field.name() == SYSTEM_LOG_ADMISSION_FIELD {
            self.admitted = value.as_bool().unwrap_or(false);
            return;
        }
        if field.name() == "message" {
            self.message = value.as_str().map(str::to_owned).or_else(|| Some(value.to_string()));
            return;
        }
        if field.name() == "fields_json" {
            self.record_fields_json(value);
            return;
        }
        self.fields.insert(field.name().into(), value);
    }

    fn record_fields_json(&mut self, value: Value) {
        let Some(raw) = value.as_str() else {
            self.fields.insert("fields_json".into(), value);
            return;
        };
        let Ok(Value::Object(fields)) = serde_json::from_str(raw) else {
            self.fields.insert("fields_json".into(), Value::String(raw.into()));
            return;
        };
        self.fields.extend(fields);
    }
}

impl Visit for EventVisitor {
    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.record_value(field, Value::Bool(value));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.record_value(field, Value::Number(value.into()));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.record_value(field, Value::Number(value.into()));
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.record_value(field, serde_json::Number::from_f64(value).map(Value::Number).unwrap_or(Value::Null));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.record_value(field, Value::String(value.into()));
    }

    fn record_error(&mut self, field: &tracing::field::Field, value: &(dyn std::error::Error + 'static)) {
        self.record_value(field, Value::String(value.to_string()));
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.record_value(field, Value::String(format!("{value:?}")));
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use tracing_subscriber::{Registry, layer::SubscriberExt};

    use crate::{SystemLogLevel, SystemLogSink, start_system_log_runtime};

    use super::SystemLogLayer;

    #[tokio::test]
    async fn layer_captures_admitted_events_and_redacts_sensitive_fields() {
        let sink = Arc::new(CollectingSink::default());
        let runtime = start_system_log_runtime(sink.clone(), SystemLogLevel::Trace);
        let subscriber = Registry::default().with(SystemLogLayer::new(runtime.emitter()));

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(target: "test::system_log", __taco_system_log = true, message = "request completed", request_id = "request-1", password = "secret");
            tracing::info!(target: "third_party", message = "must not persist");
        });
        wait_for_events(&sink).await;

        let events = sink.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].message, "request completed");
        assert_eq!(events[0].fields["request_id"], "request-1");
        assert_eq!(events[0].fields["password"], kernel::redaction::REDACTED);
    }

    async fn wait_for_events(sink: &CollectingSink) {
        tokio::time::timeout(std::time::Duration::from_secs(1), async {
            while sink.events().is_empty() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("writer did not persist the captured event");
    }

    #[derive(Default)]
    struct CollectingSink {
        events: Mutex<Vec<crate::SystemLogEvent>>,
    }

    impl CollectingSink {
        fn events(&self) -> Vec<crate::SystemLogEvent> {
            self.events.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl SystemLogSink for CollectingSink {
        async fn insert_batch(&self, events: Vec<crate::SystemLogEvent>) -> Result<(), String> {
            self.events.lock().unwrap().extend(events);
            Ok(())
        }
    }
}
