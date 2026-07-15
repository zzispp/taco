use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub jwt: JwtSettings,
    pub captcha: CaptchaSettings,
    pub auth: AuthSettings,
    pub user: UserSettings,
    pub cors: CorsSettings,
    pub http: HttpSettings,
    pub metrics: MetricsSettings,
    pub audit: AuditSettings,
    pub client_info: ClientInfoSettings,
    pub redis: RedisSettings,
    pub scheduler: SchedulerSettings,
    #[serde(default)]
    pub uploads: UploadSettings,
    pub tracing: TracingSettings,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct JwtSettings {
    pub secret: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptchaSettings {
    pub cloudflare_turnstile: CloudflareTurnstileSettings,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CloudflareTurnstileSettings {
    pub secret_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthSettings {
    pub whitelist: Vec<AuthWhitelistRule>,
    pub refresh_cookie: RefreshCookieSettings,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserSettings {
    pub online_sessions: OnlineSessionSettings,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OnlineSessionSettings {
    /// Interval between independent expired-session cleanup cycles.
    pub cleanup_interval_ms: u64,
    /// Maximum expired sessions removed by one cleanup transaction.
    pub cleanup_batch_size: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefreshCookieSettings {
    pub secure: bool,
    pub domain: Option<String>,
    pub path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthWhitelistRule {
    pub methods: Vec<String>,
    pub path_pattern: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorsSettings {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub exposed_headers: Vec<String>,
    pub allow_credentials: bool,
    pub max_age_seconds: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpSettings {
    #[serde(default = "default_request_timeout_ms")]
    pub request_timeout_ms: u64,
    #[serde(default = "default_compression_enabled")]
    pub compression_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricsSettings {
    #[serde(default = "default_metrics_enabled")]
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditSettings {
    pub outbox: AuditOutboxSettings,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditOutboxSettings {
    /// Number of independent consumers that claim durable audit messages.
    pub worker_count: usize,
    /// Maximum number of rows claimed by one consumer transaction.
    pub claim_batch_size: usize,
    /// Delay while the queue has no deliverable record.
    pub poll_interval_ms: u64,
    /// Lease duration after which a crashed consumer's record is claimable again.
    pub lease_duration_ms: u64,
    /// Delay before retrying an enrichment or projection failure.
    pub retry_delay_ms: u64,
    /// Interval for deleting only completed delivery receipts.
    pub cleanup_interval_ms: u64,
    /// Maximum completed receipts removed in one cleanup transaction.
    pub cleanup_batch_size: usize,
    /// Retention period for completed delivery receipts, not final audit logs.
    pub processed_retention_days: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientInfoSettings {
    pub ip_location: ClientIpLocationSettings,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientIpLocationSettings {
    pub request_timeout_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchedulerSettings {
    pub http_client: SchedulerHttpClientSettings,
    pub runtime: SchedulerRuntimeSettings,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchedulerHttpClientSettings {
    /// Total timeout for one scheduled HTTP request.
    pub request_timeout_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchedulerRuntimeSettings {
    /// Interval for leader health checks, notification recovery, and retries.
    pub reconcile_interval_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct UploadSettings {
    #[serde(default = "default_avatar_directory")]
    pub avatar_directory: String,
}

impl Default for UploadSettings {
    fn default() -> Self {
        Self {
            avatar_directory: default_avatar_directory(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TracingSettings {
    pub log_level: String,
    #[serde(default)]
    pub file: TracingFileSettings,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
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

fn default_avatar_directory() -> String {
    "storage/uploads/avatars".to_owned()
}
