use configuration::{
    AuthSettings, CorsSettings, DatabaseSettings, HttpSettings, JwtSettings, MetricsSettings, RedisSettings, ServerSettings, Settings, TracingFileSettings,
    TracingSettings, UploadSettings,
};

use super::routes::{auth_whitelist, ensure_auth_whitelist_rule};

const TEST_SERVER_PORT: u16 = 3000;
const TEST_DATABASE_PORT: u16 = 5432;
const TEST_REDIS_PORT: u16 = 6379;
const TEST_HTTP_TIMEOUT_MS: u64 = 30_000;
const TEST_REDIS_DATABASE: u16 = 0;

#[test]
fn ensure_auth_whitelist_rule_adds_rule_once() {
    let mut rules = vec![];

    ensure_auth_whitelist_rule(&mut rules, &["GET"], "/api/auth/me");
    ensure_auth_whitelist_rule(&mut rules, &["GET"], "/api/auth/me");

    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].methods, vec!["GET"]);
    assert_eq!(rules[0].path_pattern, "/api/auth/me");
}

#[test]
fn auth_whitelist_includes_public_avatar_files() {
    let rules = auth_whitelist(&test_settings());

    let avatar_rule = rules.iter().find(|rule| rule.path_pattern == "/uploads/avatars/{*file}");

    assert_eq!(avatar_rule.map(|rule| rule.methods.clone()), Some(vec!["GET".to_owned()]));
}

fn test_settings() -> Settings {
    Settings {
        server: test_server_settings(),
        database: test_database_settings(),
        jwt: JwtSettings { secret: "secret".into() },
        auth: AuthSettings { whitelist: vec![] },
        cors: test_cors_settings(),
        http: test_http_settings(),
        metrics: MetricsSettings { enabled: true },
        redis: test_redis_settings(),
        uploads: UploadSettings::default(),
        tracing: test_tracing_settings(),
    }
}

fn test_server_settings() -> ServerSettings {
    ServerSettings {
        host: "127.0.0.1".into(),
        port: TEST_SERVER_PORT,
    }
}

fn test_database_settings() -> DatabaseSettings {
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

fn test_cors_settings() -> CorsSettings {
    CorsSettings {
        allowed_origins: vec!["*".into()],
        allowed_methods: vec!["*".into()],
        allowed_headers: vec!["*".into()],
        exposed_headers: vec!["*".into()],
        allow_credentials: false,
        max_age_seconds: None,
    }
}

fn test_http_settings() -> HttpSettings {
    HttpSettings {
        request_timeout_ms: TEST_HTTP_TIMEOUT_MS,
        compression_enabled: true,
    }
}

fn test_redis_settings() -> RedisSettings {
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

fn test_tracing_settings() -> TracingSettings {
    TracingSettings {
        log_level: "info".into(),
        file: TracingFileSettings {
            enabled: false,
            directory: "logs".into(),
            prefix: "taco.log".into(),
        },
    }
}
