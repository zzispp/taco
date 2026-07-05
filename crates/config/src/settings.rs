use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub jwt: JwtSettings,
    pub auth: AuthSettings,
    pub cors: CorsSettings,
    pub http: HttpSettings,
    pub metrics: MetricsSettings,
    pub redis: RedisSettings,
    pub tracing: TracingSettings,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct DatabaseSettings {
    #[serde(default)]
    pub auto_migrate: bool,
    pub url: Option<String>,
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct JwtSettings {
    pub secret: String,
    pub access_token_ttl_seconds: u64,
    pub refresh_token_ttl_seconds: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct AuthSettings {
    pub whitelist: Vec<AuthWhitelistRule>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct AuthWhitelistRule {
    pub methods: Vec<String>,
    pub path_pattern: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct CorsSettings {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub exposed_headers: Vec<String>,
    pub allow_credentials: bool,
    pub max_age_seconds: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct HttpSettings {
    #[serde(default = "default_request_timeout_ms")]
    pub request_timeout_ms: u64,
    #[serde(default = "default_compression_enabled")]
    pub compression_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct MetricsSettings {
    #[serde(default = "default_metrics_enabled")]
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct RedisSettings {
    pub url: Option<String>,
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database: Option<u16>,
    pub protocol: Option<String>,
    pub key_prefix: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct TracingSettings {
    pub log_level: String,
    #[serde(default)]
    pub file: TracingFileSettings,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct TracingFileSettings {
    #[serde(default = "default_tracing_file_enabled")]
    pub enabled: bool,
    #[serde(default = "default_tracing_file_directory")]
    pub directory: String,
    #[serde(default = "default_tracing_file_prefix")]
    pub prefix: String,
}

impl Default for TracingFileSettings {
    fn default() -> Self {
        Self {
            enabled: default_tracing_file_enabled(),
            directory: default_tracing_file_directory(),
            prefix: default_tracing_file_prefix(),
        }
    }
}

fn default_request_timeout_ms() -> u64 {
    30_000
}

fn default_compression_enabled() -> bool {
    true
}

fn default_metrics_enabled() -> bool {
    true
}

fn default_tracing_file_enabled() -> bool {
    false
}

fn default_tracing_file_directory() -> String {
    "logs".to_owned()
}

fn default_tracing_file_prefix() -> String {
    "hook.log".to_owned()
}
