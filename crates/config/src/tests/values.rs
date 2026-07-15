use super::*;

mod captcha;
mod connections;
mod jwt;
mod loading;
mod runtime;

const CONFIG_EXAMPLE: &str = include_str!("../../../../config/config.example.yaml");
const MINIMAL_CONFIG_WITHOUT_AUTO_MIGRATE: &str = r#"
server:
  host: "127.0.0.1"
  port: 3000
database:
  url:
  scheme: "postgres"
  host: "localhost"
  port: 5435
  username: "postgres"
  password: "unit-test-password"
  name: "postgres"
jwt:
  secret: "config-test-jwt-secret-32-bytes!"
captcha:
  cloudflare_turnstile:
    secret_key: "config-test-turnstile-secret"
auth:
  whitelist: []
  refresh_cookie:
    secure: true
    domain:
    path: "/api/auth"
user:
  online_sessions:
    cleanup_interval_ms: 60000
    cleanup_batch_size: 1000
cors:
  allowed_origins: ["https://admin.example.test"]
  allowed_methods: ["*"]
  allowed_headers: ["*"]
  exposed_headers: ["*"]
  allow_credentials: false
  max_age_seconds:
http:
  request_timeout_ms: 30000
  compression_enabled: true
metrics:
  enabled: true
audit:
  outbox:
    worker_count: 4
    claim_batch_size: 64
    poll_interval_ms: 250
    lease_duration_ms: 30000
    retry_delay_ms: 5000
    cleanup_interval_ms: 3600000
    cleanup_batch_size: 1000
    processed_retention_days: 7
client_info:
  ip_location:
    request_timeout_ms: 3000
scheduler:
  http_client:
    request_timeout_ms: 30000
  runtime:
    reconcile_interval_ms: 1000
redis:
  url: "redis://default:@localhost:6381?protocol=resp3"
  scheme: "redis"
  host: "localhost"
  port: 6381
  username: "default"
  password: ""
  database:
  protocol: "resp3"
  key_prefix: "hook"
tracing:
  log_level: "info"
  file:
    enabled: false
    directory: "logs"
    prefix: "hook.log"
"#;

fn minimal_config_without_auto_migrate() -> String {
    MINIMAL_CONFIG_WITHOUT_AUTO_MIGRATE.into()
}

fn scheduler_yaml() -> &'static str {
    "scheduler:\n  http_client:\n    request_timeout_ms: 30000\n  runtime:\n    reconcile_interval_ms: 1000\n"
}

fn captcha_yaml() -> &'static str {
    "captcha:\n  cloudflare_turnstile:\n    secret_key: \"config-test-turnstile-secret\"\n"
}

fn audit_yaml() -> &'static str {
    "audit:\n  outbox:\n    worker_count: 4\n    claim_batch_size: 64\n    poll_interval_ms: 250\n    lease_duration_ms: 30000\n    retry_delay_ms: 5000\n    cleanup_interval_ms: 3600000\n    cleanup_batch_size: 1000\n    processed_retention_days: 7\n"
}

fn client_info_yaml() -> &'static str {
    "client_info:\n  ip_location:\n    request_timeout_ms: 3000\n"
}

fn user_yaml() -> &'static str {
    "user:\n  online_sessions:\n    cleanup_interval_ms: 60000\n    cleanup_batch_size: 1000\n"
}

fn deserialize_settings(value: &str) -> Result<Settings, config_rs::ConfigError> {
    config_rs::Config::builder()
        .add_source(config_rs::File::from_str(value, config_rs::FileFormat::Yaml))
        .build()?
        .try_deserialize()
}
