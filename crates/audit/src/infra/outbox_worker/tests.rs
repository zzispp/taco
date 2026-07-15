use std::{collections::BTreeMap, time::Duration};

use async_trait::async_trait;
use audit_contract::{AuditOutboxEvent, AuditStatus, LoginEventType, SecurityAuditEvent};
use client_info::{ClientInfoError, ClientInfoResult, IpLocation, IpLocationResolver};
use time::OffsetDateTime;
use uuid::Uuid;

use super::{AuditOutboxConfig, ClaimedAuditEvent, LOCATION_ENRICHMENT_FAILURES_TOTAL, resolve_location, worker_lease_token};

struct FailingLocationResolver;

#[async_trait]
impl IpLocationResolver for FailingLocationResolver {
    async fn resolve_ip_location(&self, _ip_address: &str) -> ClientInfoResult<IpLocation> {
        Err(ClientInfoError::Provider("unavailable".into()))
    }
}

#[test]
fn config_rejects_zero_operational_values() {
    let config = AuditOutboxConfig {
        worker_count: 0,
        claim_batch_size: 1,
        poll_interval: Duration::from_secs(1),
        lease_duration: Duration::from_secs(1),
        retry_delay: Duration::from_secs(1),
        cleanup_interval: Duration::from_secs(1),
        cleanup_batch_size: 1,
        processed_retention: Duration::from_secs(1),
    };

    assert!(config.validate().is_err());
}

#[tokio::test]
async fn location_resolution_failure_projects_unknown_without_retry_signal() {
    let metrics = hook_tracing::init_metrics(hook_tracing::MetricsConfig { enabled: true })
        .unwrap()
        .expect("enabled metrics must expose a handle");
    let before = counter_value(&metrics.render());
    let event = ClaimedAuditEvent {
        id: "outbox-1".into(),
        lease_token: "lease-1".into(),
        occurred_at: OffsetDateTime::UNIX_EPOCH,
        event: AuditOutboxEvent::Security(SecurityAuditEvent {
            request_id: "request-1".into(),
            route: "/api/auth/sign-in".into(),
            user_id: None,
            username: "alice".into(),
            ip_address: "198.51.100.10".into(),
            browser: "browser".into(),
            os: "os".into(),
            status: AuditStatus::Failure,
            event_type: LoginEventType::LoginFailure,
            message_key: "messages.user.login_failure".into(),
            message_params: BTreeMap::new(),
        }),
    };

    let location = resolve_location(&FailingLocationResolver, &event).await;

    assert_eq!(location, crate::domain::AuditLocation::Unknown);
    assert_eq!(counter_value(&metrics.render()), before + 1);
}

fn counter_value(rendered: &str) -> u64 {
    let prefix = format!("{LOCATION_ENRICHMENT_FAILURES_TOTAL}{{stream=\"security\"}} ");
    rendered
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .map(|value| value.parse().expect("location failure counter must be numeric"))
        .unwrap_or(0)
}

#[test]
fn worker_lease_token_is_a_canonical_uuid() {
    let token = worker_lease_token();

    assert_eq!(Uuid::parse_str(&token).unwrap().to_string(), token);
}

#[test]
fn config_rejects_duration_outside_the_supported_range() {
    let config = AuditOutboxConfig {
        worker_count: 1,
        claim_batch_size: 1,
        poll_interval: Duration::MAX,
        lease_duration: Duration::from_secs(1),
        retry_delay: Duration::from_secs(1),
        cleanup_interval: Duration::from_secs(1),
        cleanup_batch_size: 1,
        processed_retention: Duration::from_secs(1),
    };

    assert!(config.validate().is_err());
}

#[cfg(target_pointer_width = "64")]
#[test]
fn config_rejects_batch_size_outside_postgres_limit_range() {
    let config = AuditOutboxConfig {
        worker_count: 1,
        claim_batch_size: usize::MAX,
        poll_interval: Duration::from_secs(1),
        lease_duration: Duration::from_secs(1),
        retry_delay: Duration::from_secs(1),
        cleanup_interval: Duration::from_secs(1),
        cleanup_batch_size: 1,
        processed_retention: Duration::from_secs(1),
    };

    assert!(config.validate().is_err());
}
