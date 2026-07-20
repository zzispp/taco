use serde::{Deserialize, Serialize};

use crate::{
    application::InstallationStatus,
    application::{SetupInstallationInput, SetupInstallationInputParts},
    domain::InstallationState,
    domain::{
        AdvancedSetupOverrides, InitialAdministrator, InitialAdministratorInput, PostgresConnection, PostgresConnectionInput, RedisConnection,
        RedisConnectionInput, SetupInputError,
    },
};

#[derive(Serialize)]
pub(super) struct SetupDefaultsResponse {
    postgres: ConnectionDefaultsResponse,
    redis: ConnectionDefaultsResponse,
    advanced: AdvancedDefaultsResponse,
}

impl SetupDefaultsResponse {
    pub(super) fn from_profile(profile: configuration::InstallationProfile) -> Self {
        Self {
            postgres: ConnectionDefaultsResponse {
                port: profile.database.port,
                use_tls: !matches!(profile.database.ssl_mode, configuration::DatabaseSslMode::Disable),
            },
            redis: ConnectionDefaultsResponse {
                port: profile.redis.port,
                use_tls: matches!(profile.redis.scheme, configuration::RedisScheme::Rediss),
            },
            advanced: AdvancedDefaultsResponse::from_profile(profile),
        }
    }
}

#[derive(Serialize)]
struct ConnectionDefaultsResponse {
    port: u16,
    use_tls: bool,
}

#[derive(Serialize)]
struct AdvancedDefaultsResponse {
    http_request_timeout_ms: u64,
    compression_enabled: bool,
    metrics_enabled: bool,
    online_session_cleanup_interval_ms: u64,
    online_session_cleanup_batch_size: usize,
    audit_outbox_worker_count: usize,
    audit_outbox_claim_batch_size: usize,
    audit_outbox_poll_interval_ms: u64,
    audit_outbox_lease_duration_ms: u64,
    audit_outbox_retry_delay_ms: u64,
    audit_outbox_cleanup_interval_ms: u64,
    audit_outbox_cleanup_batch_size: usize,
    audit_outbox_processed_retention_days: u64,
    client_ip_location_timeout_ms: u64,
    scheduler_http_timeout_ms: u64,
    scheduler_reconcile_interval_ms: u64,
    redis_key_prefix: String,
}

impl AdvancedDefaultsResponse {
    fn from_profile(profile: configuration::InstallationProfile) -> Self {
        Self {
            http_request_timeout_ms: profile.http.request_timeout_ms,
            compression_enabled: profile.http.compression_enabled,
            metrics_enabled: profile.metrics.enabled,
            online_session_cleanup_interval_ms: profile.user.online_sessions.cleanup_interval_ms,
            online_session_cleanup_batch_size: profile.user.online_sessions.cleanup_batch_size,
            audit_outbox_worker_count: profile.audit.outbox.worker_count,
            audit_outbox_claim_batch_size: profile.audit.outbox.claim_batch_size,
            audit_outbox_poll_interval_ms: profile.audit.outbox.poll_interval_ms,
            audit_outbox_lease_duration_ms: profile.audit.outbox.lease_duration_ms,
            audit_outbox_retry_delay_ms: profile.audit.outbox.retry_delay_ms,
            audit_outbox_cleanup_interval_ms: profile.audit.outbox.cleanup_interval_ms,
            audit_outbox_cleanup_batch_size: profile.audit.outbox.cleanup_batch_size,
            audit_outbox_processed_retention_days: profile.audit.outbox.processed_retention_days,
            client_ip_location_timeout_ms: profile.client_info.ip_location.request_timeout_ms,
            scheduler_http_timeout_ms: profile.scheduler.http_client.request_timeout_ms,
            scheduler_reconcile_interval_ms: profile.scheduler.runtime.reconcile_interval_ms,
            redis_key_prefix: profile.redis.key_prefix,
        }
    }
}

#[derive(Serialize)]
pub(super) struct InstallationStatusResponse {
    state: &'static str,
}

impl From<InstallationStatus> for InstallationStatusResponse {
    fn from(status: InstallationStatus) -> Self {
        let state = match status.state() {
            InstallationState::Setup => "setup",
            InstallationState::Installed => "installed",
        };
        Self { state }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresConnectionPayload {
    host: String,
    #[serde(default = "default_postgres_port")]
    port: u16,
    username: String,
    password: String,
    database: String,
    #[serde(default = "default_postgres_use_tls")]
    use_tls: bool,
}

impl TryFrom<PostgresConnectionPayload> for PostgresConnection {
    type Error = SetupInputError;

    fn try_from(value: PostgresConnectionPayload) -> Result<Self, Self::Error> {
        Self::new(PostgresConnectionInput {
            host: value.host,
            port: value.port,
            username: value.username,
            password: value.password,
            database: value.database,
            use_tls: value.use_tls,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RedisConnectionPayload {
    host: String,
    #[serde(default = "default_redis_port")]
    port: u16,
    username: Option<String>,
    password: Option<String>,
    database: Option<u16>,
    #[serde(default = "default_redis_use_tls")]
    use_tls: bool,
}

impl TryFrom<RedisConnectionPayload> for RedisConnection {
    type Error = SetupInputError;

    fn try_from(value: RedisConnectionPayload) -> Result<Self, Self::Error> {
        Self::new(RedisConnectionInput {
            host: value.host,
            port: value.port,
            username: value.username,
            password: value.password,
            database: value.database,
            use_tls: value.use_tls,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct InstallationPayload {
    postgres: PostgresConnectionPayload,
    redis: RedisConnectionPayload,
    administrator: InitialAdministratorPayload,
    #[serde(default)]
    advanced: AdvancedSetupOverrides,
}

impl TryFrom<InstallationPayload> for SetupInstallationInput {
    type Error = SetupInputError;

    fn try_from(value: InstallationPayload) -> Result<Self, Self::Error> {
        SetupInstallationInput::new(SetupInstallationInputParts {
            postgres: value.postgres.try_into()?,
            redis: value.redis.try_into()?,
            administrator: value.administrator.try_into()?,
            advanced: value.advanced,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InitialAdministratorPayload {
    username: String,
    email: String,
    password: String,
}

impl TryFrom<InitialAdministratorPayload> for InitialAdministrator {
    type Error = SetupInputError;

    fn try_from(value: InitialAdministratorPayload) -> Result<Self, Self::Error> {
        Self::new(InitialAdministratorInput {
            username: value.username,
            email: value.email,
            password: value.password,
        })
    }
}

#[derive(Serialize)]
pub(super) struct ConnectionTestResponse {
    status: &'static str,
}

impl ConnectionTestResponse {
    pub(super) const fn valid() -> Self {
        Self { status: "ok" }
    }
}

#[derive(Serialize)]
pub(super) struct InstallationResponse {
    state: &'static str,
    restart_required: bool,
}

impl InstallationResponse {
    pub(super) const fn completed() -> Self {
        Self {
            state: "installed",
            restart_required: true,
        }
    }
}

fn default_postgres_port() -> u16 {
    configuration::InstallationProfile::default().database.port
}

fn default_redis_port() -> u16 {
    configuration::InstallationProfile::default().redis.port
}

fn default_postgres_use_tls() -> bool {
    !matches!(
        configuration::InstallationProfile::default().database.ssl_mode,
        configuration::DatabaseSslMode::Disable
    )
}

fn default_redis_use_tls() -> bool {
    matches!(configuration::InstallationProfile::default().redis.scheme, configuration::RedisScheme::Rediss)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::InstallationPayload;

    #[test]
    fn installation_payload_rejects_removed_owner_replacement_fields() {
        let payload = json!({
            "postgres": {
                "host": "postgres.internal",
                "username": "taco",
                "password": "secret",
                "database": "taco"
            },
            "redis": {"host": "redis.internal"},
            "administrator": {
                "username": "owner",
                "email": "owner@example.test",
                "password": "owner-secret"
            },
            "replace_installation_owner": true,
            "confirm_owner_replacement": true
        });

        assert!(serde_json::from_value::<InstallationPayload>(payload).is_err());
    }
}
