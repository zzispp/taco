mod error;
mod ports;
mod profile;
mod recovery;
mod request;
mod secret;
mod service;
mod status;

pub use error::SetupError;
pub use ports::{
    ExistingInstallationDetector, ExistingInstallationVerificationFailure, ExistingInstallationVerifier, InitialInstallationDataResetter,
    InitialInstallationMigrator, InstallationOwnerProvisioner, InstallationOwnerValidationFailure, InstallationOwnerValidator, InstallationStateWriteFailure,
    InstallationStateWriter, JwtSecretGenerator, OwnerProvisioningFailure, PostgresConnectionTester, RedisConnectionTester, SetupPortFailure, ShutdownSignal,
};
pub use profile::{postgres_connection_settings, redis_connection_settings};
pub use recovery::{InstallationConnections, InstallationRecoveryError, InstallationRecoveryService};
pub use request::{SetupInstallationInput, SetupInstallationInputParts};
pub use secret::RandomJwtSecretGenerator;
pub use service::{InstallationCompleted, SetupDependencies, SetupService, SetupUseCase};
pub use status::InstallationStatus;

use profile::installation_profile;
use secret::valid_jwt_secret;

#[cfg(test)]
mod tests;
