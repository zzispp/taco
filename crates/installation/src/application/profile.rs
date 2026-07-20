use configuration::{DatabaseSettings, DatabaseSslMode, InstallationProfile, RedisScheme, RedisSettings};

use crate::domain::{AdvancedSetupOverrides, PostgresConnection, RedisConnection};

pub(super) fn installation_profile(
    postgres: &PostgresConnection,
    redis: &RedisConnection,
    jwt_secret: String,
    advanced: &AdvancedSetupOverrides,
) -> InstallationProfile {
    let mut profile = InstallationProfile::default();
    apply_connections(&mut profile, postgres, redis, jwt_secret);
    apply_advanced_overrides(&mut profile, advanced);
    profile
}

/// Maps the PostgreSQL setup input onto the installation profile's database defaults.
pub fn postgres_connection_settings(connection: &PostgresConnection) -> DatabaseSettings {
    let mut settings = InstallationProfile::default().database;
    settings.ssl_mode = if connection.use_tls() {
        DatabaseSslMode::VerifyFull
    } else {
        DatabaseSslMode::Disable
    };
    settings.host = connection.host().into();
    settings.port = connection.port();
    settings.username = connection.username().into();
    settings.password = connection.password().into();
    settings.name = connection.database().into();
    settings
}

/// Maps the Redis setup input onto the installation profile's Redis defaults.
pub fn redis_connection_settings(connection: &RedisConnection) -> RedisSettings {
    let mut settings = InstallationProfile::default().redis;
    settings.scheme = if connection.use_tls() { RedisScheme::Rediss } else { RedisScheme::Redis };
    settings.host = connection.host().into();
    settings.port = connection.port();
    settings.username = connection.username().map(str::to_owned);
    settings.password = connection.password().map(str::to_owned);
    settings.database = connection.database();
    settings
}

fn apply_connections(profile: &mut InstallationProfile, postgres: &PostgresConnection, redis: &RedisConnection, jwt_secret: String) {
    profile.database = postgres_connection_settings(postgres);
    profile.jwt.secret = jwt_secret;
    profile.redis = redis_connection_settings(redis);
}

fn apply_advanced_overrides(profile: &mut InstallationProfile, advanced: &AdvancedSetupOverrides) {
    apply_optional(&mut profile.http.request_timeout_ms, advanced.http_request_timeout_ms);
    apply_optional(&mut profile.http.compression_enabled, advanced.compression_enabled);
    apply_optional(&mut profile.metrics.enabled, advanced.metrics_enabled);
    apply_optional(
        &mut profile.user.online_sessions.cleanup_interval_ms,
        advanced.online_session_cleanup_interval_ms,
    );
    apply_optional(&mut profile.user.online_sessions.cleanup_batch_size, advanced.online_session_cleanup_batch_size);
    apply_optional(&mut profile.audit.outbox.worker_count, advanced.audit_outbox_worker_count);
    apply_optional(&mut profile.audit.outbox.claim_batch_size, advanced.audit_outbox_claim_batch_size);
    apply_optional(&mut profile.audit.outbox.poll_interval_ms, advanced.audit_outbox_poll_interval_ms);
    apply_optional(&mut profile.audit.outbox.lease_duration_ms, advanced.audit_outbox_lease_duration_ms);
    apply_optional(&mut profile.audit.outbox.retry_delay_ms, advanced.audit_outbox_retry_delay_ms);
    apply_optional(&mut profile.audit.outbox.cleanup_interval_ms, advanced.audit_outbox_cleanup_interval_ms);
    apply_optional(&mut profile.audit.outbox.cleanup_batch_size, advanced.audit_outbox_cleanup_batch_size);
    apply_optional(
        &mut profile.audit.outbox.processed_retention_days,
        advanced.audit_outbox_processed_retention_days,
    );
    apply_optional(&mut profile.client_info.ip_location.request_timeout_ms, advanced.client_ip_location_timeout_ms);
    apply_optional(&mut profile.scheduler.http_client.request_timeout_ms, advanced.scheduler_http_timeout_ms);
    apply_optional(&mut profile.scheduler.runtime.reconcile_interval_ms, advanced.scheduler_reconcile_interval_ms);
    if let Some(redis_key_prefix) = &advanced.redis_key_prefix {
        profile.redis.key_prefix = redis_key_prefix.clone();
    }
}

fn apply_optional<T>(target: &mut T, value: Option<T>) {
    if let Some(value) = value {
        *target = value;
    }
}
