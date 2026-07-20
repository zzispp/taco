mod bootstrap;
mod connection_types;
mod connections;
mod environment;
mod error;
mod installation_state;
mod profile;
mod runtime_settings;
mod settings;
mod validation;

pub use bootstrap::{BootstrapInputError, BootstrapInputs, ConfigEncryptionKey, DEFAULT_LISTEN_ADDR, DataDirectory};
pub use connection_types::{DatabaseScheme, DatabaseSslMode, RedisProtocol, RedisScheme};
pub use environment::{EnvironmentReadError, EnvironmentReader};
pub use error::SettingsError;
pub use installation_state::{
    INSTALLATION_STATE_ENVELOPE_ALGORITHM, INSTALLATION_STATE_ENVELOPE_VERSION, INSTALLATION_STATE_FILE_NAME, InstallationStateEnvelope,
    InstallationStateError, InstallationStateRead, InstallationStateStore, decrypt_installation_state, encrypt_installation_state,
};
pub use profile::{InstallationProfile, PersistedInstallation};
pub use settings::{
    AuditOutboxSettings, AuditSettings, ClientInfoSettings, ClientIpLocationSettings, DatabaseSettings, HttpSettings, JwtSettings, MetricsSettings,
    OnlineSessionSettings, RedisSettings, SchedulerHttpClientSettings, SchedulerRuntimeSettings, SchedulerSettings, ServerSettings, Settings, UploadSettings,
    UserSettings,
};

#[cfg(test)]
mod tests;
