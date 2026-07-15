mod fixtures;
mod postgres;
mod postgres_outbox;

use audit_contract::{AUDIT_OUTBOX_PAYLOAD_VERSION, AuditOutboxEvent, AuditStream, LoginEventType};

use super::{EncodedEvent, decode_event};
use fixtures::security_event;

#[test]
fn outbox_decoder_accepts_all_stable_login_event_types() {
    for event_type in LoginEventType::ALL {
        let expected = AuditOutboxEvent::Security(security_event(event_type));
        let payload = serde_json::to_value(&expected).unwrap();

        assert_eq!(
            decode_event(EncodedEvent {
                stream: AuditStream::Security.code(),
                event_type: event_type.code(),
                version: AUDIT_OUTBOX_PAYLOAD_VERSION,
                payload,
            })
            .unwrap(),
            expected
        );
    }
}
