use sqlx::{PgPool, query, query_scalar};

pub(super) const OUTBOX_CONSTRAINTS: &[&str] = &[
    "chk_audit_outbox_stream",
    "chk_audit_outbox_event_type",
    "chk_audit_outbox_stream_event_type",
    "chk_audit_outbox_payload_version",
    "chk_audit_outbox_attempt_count",
    "chk_audit_outbox_lease",
    "chk_audit_outbox_processed_lease",
];

const OUTBOX_PARTIAL_INDEXES: &[(&str, &str)] = &[
    ("idx_audit_outbox_ready", "processed_at IS NULL"),
    ("idx_audit_outbox_processed", "processed_at IS NOT NULL"),
];

pub(super) async fn assert_outbox_partial_indexes(pool: &PgPool) {
    for (index, predicate) in OUTBOX_PARTIAL_INDEXES {
        let definition: String = query_scalar("SELECT indexdef FROM pg_indexes WHERE tablename='audit_outbox' AND indexname=$1")
            .bind(index)
            .fetch_one(pool)
            .await
            .unwrap();
        assert!(definition.contains(predicate), "missing partial index predicate for {index}: {definition}");
    }
}

pub(super) async fn assert_outbox_rejects_invalid_delivery_state(pool: &PgPool) {
    for fixture in invalid_outbox_fixtures() {
        assert!(outbox_insert(pool, fixture).await.is_err(), "accepted invalid outbox fixture {}", fixture.id);
    }
}

fn invalid_outbox_fixtures() -> [OutboxFixture<'static>; 6] {
    [
        OutboxFixture {
            stream: "invalid",
            ..OutboxFixture::valid("outbox-stream")
        },
        OutboxFixture {
            event_type: "login_success",
            ..OutboxFixture::valid("outbox-event-type")
        },
        OutboxFixture {
            payload_version: 2,
            ..OutboxFixture::valid("outbox-version")
        },
        OutboxFixture {
            attempt_count: -1,
            ..OutboxFixture::valid("outbox-attempts")
        },
        OutboxFixture {
            lease_token: Some("lease"),
            ..OutboxFixture::valid("outbox-lease")
        },
        OutboxFixture {
            lease_token: Some("lease"),
            has_lease_until: true,
            processed: true,
            ..OutboxFixture::valid("outbox-processed-lease")
        },
    ]
}

async fn outbox_insert(pool: &PgPool, fixture: OutboxFixture<'_>) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    query(
        "INSERT INTO audit_outbox (outbox_id,stream,event_type,payload_version,payload,occurred_at,available_at,attempt_count,lease_token,lease_until,processed_at) \
         VALUES ($1,$2,$3,$4,'{}'::jsonb,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP,$5,$6,\
         CASE WHEN $7 THEN CURRENT_TIMESTAMP ELSE NULL END,CASE WHEN $8 THEN CURRENT_TIMESTAMP ELSE NULL END)",
    )
    .bind(fixture.id)
    .bind(fixture.stream)
    .bind(fixture.event_type)
    .bind(fixture.payload_version)
    .bind(fixture.attempt_count)
    .bind(fixture.lease_token)
    .bind(fixture.has_lease_until)
    .bind(fixture.processed)
    .execute(pool)
    .await
}

#[derive(Clone, Copy)]
struct OutboxFixture<'a> {
    id: &'a str,
    stream: &'a str,
    event_type: &'a str,
    payload_version: i16,
    attempt_count: i32,
    lease_token: Option<&'a str>,
    has_lease_until: bool,
    processed: bool,
}

impl<'a> OutboxFixture<'a> {
    const fn valid(id: &'a str) -> Self {
        Self {
            id,
            stream: "operation",
            event_type: "operation",
            payload_version: 1,
            attempt_count: 0,
            lease_token: None,
            has_lease_until: false,
            processed: false,
        }
    }
}
