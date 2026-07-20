use thiserror::Error;

use crate::domain::SetupInputError;

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum SetupError {
    #[error("invalid setup input: {0}")]
    InvalidInput(#[from] SetupInputError),
    #[error("initial administrator input is invalid")]
    InstallationOwnerInvalid,
    #[error("installation has already completed")]
    AlreadyInstalled,
    #[error("PostgreSQL connection test failed")]
    PostgresConnectionFailed,
    #[error("Redis connection test failed")]
    RedisConnectionFailed,
    #[error("the selected PostgreSQL database already contains a Taco installation")]
    ExistingInstallationDetected,
    #[error("failed to inspect the selected PostgreSQL database for an existing Taco installation")]
    ExistingInstallationDetectionFailed,
    #[error("installation data reset failed")]
    InstallationDataResetFailed,
    #[error("initial database migration failed")]
    MigrationFailed,
    #[error("installation owner provisioning failed")]
    InstallationOwnerProvisioningFailed,
    #[error("JWT signing key generation failed")]
    JwtGenerationFailed,
    #[error("generated JWT signing key is invalid")]
    InvalidGeneratedJwt,
    #[error("installation state already exists")]
    InstallationStateAlreadyExists,
    #[error("installation state persistence failed")]
    InstallationStatePersistenceFailed,
    #[error("graceful shutdown signal failed")]
    ShutdownFailed,
}

impl SetupError {
    pub const fn localization_key(&self) -> &'static str {
        match self {
            Self::InvalidInput(_) => "errors.installation.invalid_input",
            Self::InstallationOwnerInvalid => "errors.installation.initial_administrator_invalid",
            Self::AlreadyInstalled | Self::InstallationStateAlreadyExists => "errors.installation.already_installed",
            Self::PostgresConnectionFailed => "errors.installation.postgres_connection_failed",
            Self::RedisConnectionFailed => "errors.installation.redis_connection_failed",
            Self::ExistingInstallationDetected => "errors.installation.existing_installation_detected",
            Self::ExistingInstallationDetectionFailed => "errors.installation.existing_installation_detection_failed",
            Self::InstallationDataResetFailed => "errors.installation.data_reset_failed",
            Self::MigrationFailed => "errors.installation.migration_failed",
            Self::InstallationOwnerProvisioningFailed => "errors.installation.owner_provisioning_failed",
            Self::JwtGenerationFailed | Self::InvalidGeneratedJwt => "errors.installation.jwt_generation_failed",
            Self::InstallationStatePersistenceFailed => "errors.installation.state_persistence_failed",
            Self::ShutdownFailed => "errors.installation.shutdown_failed",
        }
    }
}
