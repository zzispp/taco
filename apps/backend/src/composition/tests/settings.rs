use configuration::{
    AuditOutboxSettings, AuditSettings, AuthSettings, CaptchaSettings, ClientInfoSettings, ClientIpLocationSettings, CloudflareTurnstileSettings, CorsSettings,
    DatabaseSettings, HttpSettings, JwtSettings, MetricsSettings, OnlineSessionSettings, RedisSettings, RefreshCookieSettings, SchedulerHttpClientSettings,
    SchedulerRuntimeSettings, SchedulerSettings, ServerSettings, Settings, TracingFileSettings, TracingSettings, UploadSettings, UserSettings,
};

const TEST_SERVER_PORT: u16 = 3_000;
const TEST_DATABASE_PORT: u16 = 5_432;
const TEST_REDIS_PORT: u16 = 6_379;
const TEST_HTTP_TIMEOUT_MS: u64 = 30_000;
const TEST_REDIS_DATABASE: u16 = 0;
const TEST_AUDIT_WORKER_COUNT: usize = 4;
const TEST_AUDIT_CLAIM_BATCH_SIZE: usize = 64;
const TEST_AUDIT_POLL_INTERVAL_MS: u64 = 250;
const TEST_AUDIT_LEASE_DURATION_MS: u64 = 30_000;
const TEST_AUDIT_RETRY_DELAY_MS: u64 = 5_000;
const TEST_AUDIT_CLEANUP_INTERVAL_MS: u64 = 3_600_000;
const TEST_AUDIT_CLEANUP_BATCH_SIZE: usize = 1_000;
const TEST_AUDIT_PROCESSED_RETENTION_DAYS: u64 = 7;
const TEST_JWT_SECRET: &str = "config-test-jwt-secret-32-bytes!";

pub(crate) fn test_settings() -> Settings {
    Settings {
        server: ServerSettings {
            host: "127.0.0.1".into(),
            port: TEST_SERVER_PORT,
        },
        database: database_settings(),
        jwt: JwtSettings {
            secret: TEST_JWT_SECRET.into(),
        },
        captcha: CaptchaSettings {
            cloudflare_turnstile: CloudflareTurnstileSettings {
                secret_key: "test-turnstile-secret".into(),
            },
        },
        auth: AuthSettings {
            whitelist: vec![],
            refresh_cookie: RefreshCookieSettings {
                secure: true,
                domain: None,
                path: "/api/auth".into(),
            },
        },
        user: UserSettings {
            online_sessions: OnlineSessionSettings {
                cleanup_interval_ms: 60_000,
                cleanup_batch_size: 1_000,
            },
        },
        cors: cors_settings(),
        http: HttpSettings {
            request_timeout_ms: TEST_HTTP_TIMEOUT_MS,
            compression_enabled: true,
        },
        metrics: MetricsSettings { enabled: true },
        audit: audit_settings(),
        client_info: client_info_settings(),
        redis: redis_settings(),
        scheduler: scheduler_settings(),
        uploads: UploadSettings::default(),
        tracing: tracing_settings(),
    }
}

fn audit_settings() -> AuditSettings {
    AuditSettings {
        outbox: AuditOutboxSettings {
            worker_count: TEST_AUDIT_WORKER_COUNT,
            claim_batch_size: TEST_AUDIT_CLAIM_BATCH_SIZE,
            poll_interval_ms: TEST_AUDIT_POLL_INTERVAL_MS,
            lease_duration_ms: TEST_AUDIT_LEASE_DURATION_MS,
            retry_delay_ms: TEST_AUDIT_RETRY_DELAY_MS,
            cleanup_interval_ms: TEST_AUDIT_CLEANUP_INTERVAL_MS,
            cleanup_batch_size: TEST_AUDIT_CLEANUP_BATCH_SIZE,
            processed_retention_days: TEST_AUDIT_PROCESSED_RETENTION_DAYS,
        },
    }
}

fn client_info_settings() -> ClientInfoSettings {
    ClientInfoSettings {
        ip_location: ClientIpLocationSettings { request_timeout_ms: 3_000 },
    }
}

fn scheduler_settings() -> SchedulerSettings {
    SchedulerSettings {
        http_client: SchedulerHttpClientSettings { request_timeout_ms: 30_000 },
        runtime: SchedulerRuntimeSettings { reconcile_interval_ms: 1_000 },
    }
}

fn database_settings() -> DatabaseSettings {
    DatabaseSettings {
        auto_migrate: false,
        url: None,
        scheme: "postgres".into(),
        host: "localhost".into(),
        port: TEST_DATABASE_PORT,
        username: "postgres".into(),
        password: Some("postgres".into()),
        name: "postgres".into(),
    }
}

fn cors_settings() -> CorsSettings {
    CorsSettings {
        allowed_origins: vec!["https://admin.example.test".into()],
        allowed_methods: vec!["*".into()],
        allowed_headers: vec!["*".into()],
        exposed_headers: vec!["*".into()],
        allow_credentials: false,
        max_age_seconds: None,
    }
}

fn redis_settings() -> RedisSettings {
    RedisSettings {
        url: None,
        scheme: "redis".into(),
        host: "localhost".into(),
        port: TEST_REDIS_PORT,
        username: None,
        password: None,
        database: Some(TEST_REDIS_DATABASE),
        protocol: Some("resp3".into()),
        key_prefix: "taco".into(),
    }
}

fn tracing_settings() -> TracingSettings {
    TracingSettings {
        log_level: "info".into(),
        file: TracingFileSettings {
            enabled: false,
            directory: "logs".into(),
            prefix: "taco.log".into(),
        },
    }
}
