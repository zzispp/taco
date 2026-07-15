use super::*;

mod cors;
mod runtime;
mod values;

const TEST_SCHEDULER_REQUEST_TIMEOUT_MS: u64 = 30_000;
const TEST_SCHEDULER_RECONCILE_INTERVAL_MS: u64 = 1_000;
pub(super) const TEST_JWT_SECRET: &str = "config-test-jwt-secret-32-bytes!";
const TEST_DATABASE_PASSWORD: &str = "unit-test-password";

pub(super) fn settings_with_database(database: DatabaseSettings) -> Settings {
    Settings {
        server: ServerSettings {
            host: "127.0.0.1".into(),
            port: 3000,
        },
        database,
        jwt: jwt_settings(),
        captcha: captcha_settings(),
        auth: AuthSettings {
            whitelist: vec![],
            refresh_cookie: refresh_cookie_settings(),
        },
        user: user_settings(),
        cors: cors_settings(),
        http: http_settings(),
        metrics: metrics_settings(),
        audit: audit_settings(),
        client_info: client_info_settings(),
        redis: redis_settings(),
        scheduler: scheduler_settings(),
        uploads: UploadSettings::default(),
        tracing: tracing_settings(),
    }
}

pub(super) fn settings_with_captcha(captcha: CaptchaSettings) -> Settings {
    Settings {
        captcha,
        ..settings_with_database(database_parts())
    }
}

pub(super) fn settings_with_jwt(jwt: JwtSettings) -> Settings {
    Settings {
        jwt,
        ..settings_with_database(database_parts())
    }
}

pub(super) fn settings_with_redis(redis: RedisSettings) -> Settings {
    Settings {
        redis,
        ..settings_with_database(database_parts())
    }
}

pub(super) fn settings_with_tracing(tracing: TracingSettings) -> Settings {
    Settings {
        tracing,
        ..settings_with_database(database_parts())
    }
}

pub(super) fn settings_with_cors(cors: CorsSettings) -> Settings {
    Settings {
        cors,
        ..settings_with_database(database_parts())
    }
}

pub(super) fn settings_with_http(http: HttpSettings) -> Settings {
    Settings {
        http,
        ..settings_with_database(database_parts())
    }
}

pub(super) fn database_parts() -> DatabaseSettings {
    DatabaseSettings {
        auto_migrate: false,
        url: None,
        scheme: "postgres".into(),
        host: "localhost".into(),
        port: 5435,
        username: "postgres".into(),
        password: Some(TEST_DATABASE_PASSWORD.into()),
        name: "postgres".into(),
    }
}

pub(super) fn jwt_settings() -> JwtSettings {
    JwtSettings {
        secret: TEST_JWT_SECRET.into(),
    }
}

pub(super) fn captcha_settings() -> CaptchaSettings {
    CaptchaSettings {
        cloudflare_turnstile: CloudflareTurnstileSettings {
            secret_key: "config-test-turnstile-secret".into(),
        },
    }
}

pub(super) fn refresh_cookie_settings() -> RefreshCookieSettings {
    RefreshCookieSettings {
        secure: true,
        domain: None,
        path: "/api/auth".into(),
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
        url: Some("redis://default:@localhost:6381?protocol=resp3".into()),
        scheme: "redis".into(),
        host: "localhost".into(),
        port: 6381,
        username: Some("default".into()),
        password: Some(String::new()),
        database: None,
        protocol: Some("resp3".into()),
        key_prefix: "hook".into(),
    }
}

pub(super) fn cors_settings() -> CorsSettings {
    CorsSettings {
        allowed_origins: vec!["https://admin.example.test".into()],
        allowed_methods: vec!["*".into()],
        allowed_headers: vec!["*".into()],
        exposed_headers: vec!["*".into()],
        allow_credentials: false,
        max_age_seconds: None,
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
        http_client: SchedulerHttpClientSettings {
            request_timeout_ms: TEST_SCHEDULER_REQUEST_TIMEOUT_MS,
        },
        runtime: SchedulerRuntimeSettings {
            reconcile_interval_ms: TEST_SCHEDULER_RECONCILE_INTERVAL_MS,
        },
    }
}

pub(super) fn tracing_settings() -> TracingSettings {
    TracingSettings {
        log_level: "info".into(),
        file: TracingFileSettings::default(),
    }
}
