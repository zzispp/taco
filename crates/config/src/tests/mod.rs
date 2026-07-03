use super::*;

mod cors;
mod values;

pub(super) fn settings_with_database(database: DatabaseSettings) -> Settings {
    Settings {
        server: ServerSettings {
            host: "127.0.0.1".into(),
            port: 3000,
        },
        database,
        jwt: jwt_settings(),
        admin: admin_settings(),
        auth: AuthSettings { whitelist: vec![] },
        cors: cors_settings(),
        http: http_settings(),
        metrics: metrics_settings(),
        redis: redis_settings(),
        tracing: tracing_settings(),
    }
}

pub(super) fn settings_with_jwt(jwt: JwtSettings) -> Settings {
    Settings {
        jwt,
        ..settings_with_database(database_parts())
    }
}

pub(super) fn settings_with_admin(admin: AdminSettings) -> Settings {
    Settings {
        admin,
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
        access_token_ttl_seconds: 900,
        refresh_token_ttl_seconds: 604800,
    }
}

pub(super) fn admin_settings() -> AdminSettings {
    AdminSettings {
        id: "00000000-0000-7000-8000-000000000000".into(),
        username: "admin".into(),
        email: "admin@example.com".into(),
        role: "admin".into(),
        is_active: true,
        password_hash: "admin-password-hash-from-config".into(),
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

pub(super) fn tracing_settings() -> TracingSettings {
    TracingSettings {
        log_level: "info".into(),
        file: TracingFileSettings::default(),
    }
}
