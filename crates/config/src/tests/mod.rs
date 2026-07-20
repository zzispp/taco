use std::{net::SocketAddr, path::PathBuf};

use super::*;

mod bootstrap;
mod connections;
mod installation_state;
mod jwt;
mod profile;
mod runtime;

pub(super) const TEST_JWT_SECRET: &str = "config-test-jwt-secret-32-bytes!";
const TEST_DATABASE_PASSWORD: &str = "unit-test-password";
const TEST_CONFIG_ENCRYPTION_KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

pub(super) fn valid_installation_profile() -> InstallationProfile {
    InstallationProfile {
        database: database_parts(),
        jwt: jwt_settings(),
        user: user_settings(),
        http: http_settings(),
        metrics: metrics_settings(),
        audit: audit_settings(),
        client_info: client_info_settings(),
        redis: redis_settings(),
        scheduler: scheduler_settings(),
    }
}

pub(super) fn settings_with_database(database: DatabaseSettings) -> Settings {
    let mut settings = valid_settings();
    settings.database = database;
    settings
}

pub(super) fn settings_with_jwt(jwt: JwtSettings) -> Settings {
    let mut settings = valid_settings();
    settings.jwt = jwt;
    settings
}

pub(super) fn settings_with_http(http: HttpSettings) -> Settings {
    let mut settings = valid_settings();
    settings.http = http;
    settings
}

pub(super) fn valid_settings() -> Settings {
    Settings::from_installation_profile(valid_installation_profile(), &bootstrap_inputs()).unwrap()
}

pub(super) fn bootstrap_inputs() -> BootstrapInputs {
    bootstrap_inputs_at(PathBuf::from("/var/lib/taco"), "127.0.0.1:3000".parse().unwrap())
}

pub(super) fn bootstrap_inputs_at(data_dir: PathBuf, listen_addr: SocketAddr) -> BootstrapInputs {
    BootstrapInputs::new(
        DataDirectory::new(data_dir).unwrap(),
        ConfigEncryptionKey::parse(TEST_CONFIG_ENCRYPTION_KEY).unwrap(),
        listen_addr,
    )
}

pub(super) fn database_parts() -> DatabaseSettings {
    DatabaseSettings {
        scheme: DatabaseScheme::Postgres,
        ssl_mode: DatabaseSslMode::Disable,
        host: "localhost".into(),
        port: 5_435,
        username: "postgres".into(),
        password: TEST_DATABASE_PASSWORD.into(),
        name: "postgres".into(),
    }
}

pub(super) fn jwt_settings() -> JwtSettings {
    JwtSettings {
        secret: TEST_JWT_SECRET.into(),
    }
}

pub(super) fn user_settings() -> UserSettings {
    UserSettings {
        online_sessions: OnlineSessionSettings {
            cleanup_interval_ms: 60_000,
            cleanup_batch_size: 1_000,
        },
    }
}

pub(super) fn redis_settings() -> RedisSettings {
    RedisSettings {
        scheme: RedisScheme::Redis,
        host: "localhost".into(),
        port: 6_381,
        username: Some("default".into()),
        password: None,
        database: None,
        protocol: Some(RedisProtocol::Resp3),
        key_prefix: "taco".into(),
    }
}

pub(super) fn http_settings() -> HttpSettings {
    HttpSettings {
        request_timeout_ms: 30_000,
        compression_enabled: true,
    }
}

pub(super) fn metrics_settings() -> MetricsSettings {
    MetricsSettings { enabled: true }
}

pub(super) fn audit_settings() -> AuditSettings {
    AuditSettings {
        outbox: AuditOutboxSettings {
            worker_count: 4,
            claim_batch_size: 64,
            poll_interval_ms: 250,
            lease_duration_ms: 30_000,
            retry_delay_ms: 5_000,
            cleanup_interval_ms: 3_600_000,
            cleanup_batch_size: 1_000,
            processed_retention_days: 7,
        },
    }
}

pub(super) fn client_info_settings() -> ClientInfoSettings {
    ClientInfoSettings {
        ip_location: ClientIpLocationSettings { request_timeout_ms: 3_000 },
    }
}

pub(super) fn scheduler_settings() -> SchedulerSettings {
    SchedulerSettings {
        http_client: SchedulerHttpClientSettings { request_timeout_ms: 30_000 },
        runtime: SchedulerRuntimeSettings { reconcile_interval_ms: 1_000 },
    }
}
