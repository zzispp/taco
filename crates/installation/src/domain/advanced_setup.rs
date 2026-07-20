use serde::Deserialize;

use super::SetupInputError;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct AdvancedSetupOverrides {
    pub http_request_timeout_ms: Option<u64>,
    pub compression_enabled: Option<bool>,
    pub metrics_enabled: Option<bool>,
    pub online_session_cleanup_interval_ms: Option<u64>,
    pub online_session_cleanup_batch_size: Option<usize>,
    pub audit_outbox_worker_count: Option<usize>,
    pub audit_outbox_claim_batch_size: Option<usize>,
    pub audit_outbox_poll_interval_ms: Option<u64>,
    pub audit_outbox_lease_duration_ms: Option<u64>,
    pub audit_outbox_retry_delay_ms: Option<u64>,
    pub audit_outbox_cleanup_interval_ms: Option<u64>,
    pub audit_outbox_cleanup_batch_size: Option<usize>,
    pub audit_outbox_processed_retention_days: Option<u64>,
    pub client_ip_location_timeout_ms: Option<u64>,
    pub scheduler_http_timeout_ms: Option<u64>,
    pub scheduler_reconcile_interval_ms: Option<u64>,
    pub redis_key_prefix: Option<String>,
}

impl AdvancedSetupOverrides {
    pub fn validate(mut self) -> Result<Self, SetupInputError> {
        validate_positive_u64("advanced.http_request_timeout_ms", self.http_request_timeout_ms)?;
        validate_positive_u64("advanced.online_session_cleanup_interval_ms", self.online_session_cleanup_interval_ms)?;
        validate_positive_usize("advanced.online_session_cleanup_batch_size", self.online_session_cleanup_batch_size)?;
        validate_positive_usize("advanced.audit_outbox_worker_count", self.audit_outbox_worker_count)?;
        validate_positive_usize("advanced.audit_outbox_claim_batch_size", self.audit_outbox_claim_batch_size)?;
        validate_positive_u64("advanced.audit_outbox_poll_interval_ms", self.audit_outbox_poll_interval_ms)?;
        validate_positive_u64("advanced.audit_outbox_lease_duration_ms", self.audit_outbox_lease_duration_ms)?;
        validate_positive_u64("advanced.audit_outbox_retry_delay_ms", self.audit_outbox_retry_delay_ms)?;
        validate_positive_u64("advanced.audit_outbox_cleanup_interval_ms", self.audit_outbox_cleanup_interval_ms)?;
        validate_positive_usize("advanced.audit_outbox_cleanup_batch_size", self.audit_outbox_cleanup_batch_size)?;
        validate_positive_u64("advanced.audit_outbox_processed_retention_days", self.audit_outbox_processed_retention_days)?;
        validate_positive_u64("advanced.client_ip_location_timeout_ms", self.client_ip_location_timeout_ms)?;
        validate_positive_u64("advanced.scheduler_http_timeout_ms", self.scheduler_http_timeout_ms)?;
        validate_positive_u64("advanced.scheduler_reconcile_interval_ms", self.scheduler_reconcile_interval_ms)?;
        self.redis_key_prefix = self.redis_key_prefix.map(normalize_key_prefix).transpose()?;
        Ok(self)
    }
}

fn normalize_key_prefix(value: String) -> Result<String, SetupInputError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(SetupInputError::BlankField("advanced.redis_key_prefix"));
    }
    Ok(trimmed.to_owned())
}

fn validate_positive_u64(field: &'static str, value: Option<u64>) -> Result<(), SetupInputError> {
    if value == Some(0) {
        return Err(SetupInputError::NonPositiveNumber(field));
    }
    Ok(())
}

fn validate_positive_usize(field: &'static str, value: Option<usize>) -> Result<(), SetupInputError> {
    if value == Some(0) {
        return Err(SetupInputError::NonPositiveNumber(field));
    }
    Ok(())
}
