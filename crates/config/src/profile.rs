use serde::{Deserialize, Serialize};

use crate::{
    AuditOutboxSettings, AuditSettings, ClientInfoSettings, ClientIpLocationSettings, DatabaseScheme, DatabaseSettings, DatabaseSslMode, HttpSettings,
    JwtSettings, MetricsSettings, OnlineSessionSettings, RedisProtocol, RedisScheme, RedisSettings, SchedulerHttpClientSettings, SchedulerRuntimeSettings,
    SchedulerSettings, UserSettings,
};

const DEFAULT_DATABASE_PORT: u16 = 5_432;
const DEFAULT_REDIS_PORT: u16 = 6_379;
const DEFAULT_HTTP_REQUEST_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_ONLINE_SESSION_CLEANUP_INTERVAL_MS: u64 = 60_000;
const DEFAULT_ONLINE_SESSION_CLEANUP_BATCH_SIZE: usize = 1_000;
const DEFAULT_AUDIT_OUTBOX_WORKER_COUNT: usize = 4;
const DEFAULT_AUDIT_OUTBOX_CLAIM_BATCH_SIZE: usize = 64;
const DEFAULT_AUDIT_OUTBOX_POLL_INTERVAL_MS: u64 = 250;
const DEFAULT_AUDIT_OUTBOX_LEASE_DURATION_MS: u64 = 30_000;
const DEFAULT_AUDIT_OUTBOX_RETRY_DELAY_MS: u64 = 5_000;
const DEFAULT_AUDIT_OUTBOX_CLEANUP_INTERVAL_MS: u64 = 3_600_000;
const DEFAULT_AUDIT_OUTBOX_CLEANUP_BATCH_SIZE: usize = 1_000;
const DEFAULT_AUDIT_OUTBOX_PROCESSED_RETENTION_DAYS: u64 = 7;
const DEFAULT_IP_LOCATION_REQUEST_TIMEOUT_MS: u64 = 3_000;
const DEFAULT_SCHEDULER_HTTP_REQUEST_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_SCHEDULER_RECONCILE_INTERVAL_MS: u64 = 1_000;
const DEFAULT_REDIS_KEY_PREFIX: &str = "taco:";

/// The encrypted, installation-owned configuration profile.
///
/// Listener and filesystem paths are bootstrap inputs, so they deliberately do
/// not appear in this persisted structure.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InstallationProfile {
    pub database: DatabaseSettings,
    pub jwt: JwtSettings,
    pub user: UserSettings,
    pub http: HttpSettings,
    pub metrics: MetricsSettings,
    pub audit: AuditSettings,
    pub client_info: ClientInfoSettings,
    pub redis: RedisSettings,
    pub scheduler: SchedulerSettings,
}

/// The only payload stored in the encrypted installation state file.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PersistedInstallation {
    pub complete: bool,
    pub profile: InstallationProfile,
}

impl PersistedInstallation {
    pub const fn completed(profile: InstallationProfile) -> Self {
        Self { complete: true, profile }
    }
}

impl Default for InstallationProfile {
    fn default() -> Self {
        Self {
            database: default_database_settings(),
            jwt: JwtSettings { secret: String::new() },
            user: default_user_settings(),
            http: default_http_settings(),
            metrics: MetricsSettings { enabled: true },
            audit: default_audit_settings(),
            client_info: default_client_info_settings(),
            redis: default_redis_settings(),
            scheduler: default_scheduler_settings(),
        }
    }
}

fn default_database_settings() -> DatabaseSettings {
    DatabaseSettings {
        scheme: DatabaseScheme::Postgres,
        ssl_mode: DatabaseSslMode::VerifyFull,
        host: String::new(),
        port: DEFAULT_DATABASE_PORT,
        username: String::new(),
        password: String::new(),
        name: String::new(),
    }
}

fn default_user_settings() -> UserSettings {
    UserSettings {
        online_sessions: OnlineSessionSettings {
            cleanup_interval_ms: DEFAULT_ONLINE_SESSION_CLEANUP_INTERVAL_MS,
            cleanup_batch_size: DEFAULT_ONLINE_SESSION_CLEANUP_BATCH_SIZE,
        },
    }
}

fn default_http_settings() -> HttpSettings {
    HttpSettings {
        request_timeout_ms: DEFAULT_HTTP_REQUEST_TIMEOUT_MS,
        compression_enabled: true,
    }
}

fn default_audit_settings() -> AuditSettings {
    AuditSettings {
        outbox: AuditOutboxSettings {
            worker_count: DEFAULT_AUDIT_OUTBOX_WORKER_COUNT,
            claim_batch_size: DEFAULT_AUDIT_OUTBOX_CLAIM_BATCH_SIZE,
            poll_interval_ms: DEFAULT_AUDIT_OUTBOX_POLL_INTERVAL_MS,
            lease_duration_ms: DEFAULT_AUDIT_OUTBOX_LEASE_DURATION_MS,
            retry_delay_ms: DEFAULT_AUDIT_OUTBOX_RETRY_DELAY_MS,
            cleanup_interval_ms: DEFAULT_AUDIT_OUTBOX_CLEANUP_INTERVAL_MS,
            cleanup_batch_size: DEFAULT_AUDIT_OUTBOX_CLEANUP_BATCH_SIZE,
            processed_retention_days: DEFAULT_AUDIT_OUTBOX_PROCESSED_RETENTION_DAYS,
        },
    }
}

fn default_client_info_settings() -> ClientInfoSettings {
    ClientInfoSettings {
        ip_location: ClientIpLocationSettings {
            request_timeout_ms: DEFAULT_IP_LOCATION_REQUEST_TIMEOUT_MS,
        },
    }
}

fn default_redis_settings() -> RedisSettings {
    RedisSettings {
        scheme: RedisScheme::Rediss,
        host: String::new(),
        port: DEFAULT_REDIS_PORT,
        username: None,
        password: None,
        database: None,
        protocol: Some(RedisProtocol::Resp3),
        key_prefix: DEFAULT_REDIS_KEY_PREFIX.into(),
    }
}

fn default_scheduler_settings() -> SchedulerSettings {
    SchedulerSettings {
        http_client: SchedulerHttpClientSettings {
            request_timeout_ms: DEFAULT_SCHEDULER_HTTP_REQUEST_TIMEOUT_MS,
        },
        runtime: SchedulerRuntimeSettings {
            reconcile_interval_ms: DEFAULT_SCHEDULER_RECONCILE_INTERVAL_MS,
        },
    }
}
