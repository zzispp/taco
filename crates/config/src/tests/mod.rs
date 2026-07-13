use super::*;

mod cors;
mod values;

const TEST_SCHEDULER_REQUEST_TIMEOUT_MS: u64 = 30_000;
const TEST_SCHEDULER_RECONCILE_INTERVAL_MS: u64 = 1_000;

pub(super) fn settings_with_database(database: DatabaseSettings) -> Settings {
    Settings {
        server: ServerSettings {
            host: "127.0.0.1".into(),
            port: 3000,
        },
        database,
        jwt: jwt_settings(),
        auth: AuthSettings { whitelist: vec![] },
        cors: cors_settings(),
        http: http_settings(),
        metrics: metrics_settings(),
        redis: redis_settings(),
        scheduler: scheduler_settings(),
        uploads: UploadSettings::default(),
        tracing: tracing_settings(),
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
        port: 5433,
        username: "postgres".into(),
        password: Some("123456".into()),
        name: "postgres".into(),
    }
}

pub(super) fn jwt_settings() -> JwtSettings {
    JwtSettings {
        secret: "jwt-secret-from-config".into(),
    }
}

pub(super) fn redis_settings() -> RedisSettings {
    RedisSettings {
        url: Some("redis://default:@localhost:6380?protocol=resp3".into()),
        scheme: "redis".into(),
        host: "localhost".into(),
        port: 6380,
        username: Some("default".into()),
        password: Some(String::new()),
        database: None,
        protocol: Some("resp3".into()),
        key_prefix: "hook".into(),
    }
}

pub(super) fn cors_settings() -> CorsSettings {
    CorsSettings {
        allowed_origins: vec!["*".into()],
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
