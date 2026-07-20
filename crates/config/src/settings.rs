use serde::{Deserialize, Serialize};

use crate::{DatabaseScheme, DatabaseSslMode, RedisProtocol, RedisScheme, SettingsError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub jwt: JwtSettings,
    pub user: UserSettings,
    pub http: HttpSettings,
    pub metrics: MetricsSettings,
    pub audit: AuditSettings,
    pub client_info: ClientInfoSettings,
    pub redis: RedisSettings,
    pub scheduler: SchedulerSettings,
    pub uploads: UploadSettings,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DatabaseSettings {
    pub scheme: DatabaseScheme,
    pub ssl_mode: DatabaseSslMode,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JwtSettings {
    pub secret: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserSettings {
    pub online_sessions: OnlineSessionSettings,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OnlineSessionSettings {
    /// Interval between independent expired-session cleanup cycles.
    pub cleanup_interval_ms: u64,
    /// Maximum expired sessions removed by one cleanup transaction.
    pub cleanup_batch_size: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HttpSettings {
    pub request_timeout_ms: u64,
    pub compression_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MetricsSettings {
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuditSettings {
    pub outbox: AuditOutboxSettings,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuditOutboxSettings {
    /// Number of independent consumers that claim durable audit messages.
    pub worker_count: usize,
    /// Maximum number of rows claimed by one consumer transaction.
    pub claim_batch_size: usize,
    /// Delay while the queue has no deliverable record.
    pub poll_interval_ms: u64,
    /// Lease duration after which a crashed consumer's record is claimable again.
    pub lease_duration_ms: u64,
    /// Delay before retrying an enrichment or projection failure.
    pub retry_delay_ms: u64,
    /// Interval for deleting only completed delivery receipts.
    pub cleanup_interval_ms: u64,
    /// Maximum completed receipts removed in one cleanup transaction.
    pub cleanup_batch_size: usize,
    /// Retention period for completed delivery receipts, not final audit logs.
    pub processed_retention_days: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClientInfoSettings {
    pub ip_location: ClientIpLocationSettings,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClientIpLocationSettings {
    pub request_timeout_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SchedulerSettings {
    pub http_client: SchedulerHttpClientSettings,
    pub runtime: SchedulerRuntimeSettings,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SchedulerHttpClientSettings {
    /// Total timeout for one scheduled HTTP request.
    pub request_timeout_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SchedulerRuntimeSettings {
    /// Interval for leader health checks, notification recovery, and retries.
    pub reconcile_interval_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RedisSettings {
    pub scheme: RedisScheme,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database: Option<u16>,
    pub protocol: Option<RedisProtocol>,
    pub key_prefix: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UploadSettings {
    pub avatar_directory: String,
}

pub(crate) fn required_config_value(key: &'static str, value: &str) -> Result<String, SettingsError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(SettingsError::BlankConfigValue(key));
    }
    Ok(trimmed.to_owned())
}
