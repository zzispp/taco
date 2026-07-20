use async_trait::async_trait;
use configuration::PersistedInstallation;

use crate::domain::{InitialAdministrator, PostgresConnection, RedisConnection};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SetupPortFailure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnerProvisioningFailure {
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallationStateWriteFailure {
    AlreadyExists,
    Failed,
}

#[async_trait]
pub trait PostgresConnectionTester: Send + Sync + 'static {
    async fn test_postgres_connection(&self, connection: &PostgresConnection) -> Result<(), SetupPortFailure>;
}

#[async_trait]
pub trait RedisConnectionTester: Send + Sync + 'static {
    async fn test_redis_connection(&self, connection: &RedisConnection) -> Result<(), SetupPortFailure>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstallationOwnerValidationFailure;

/// Validates the complete initial administrator identity before setup can
/// destructively reset the selected infrastructure targets.
pub trait InstallationOwnerValidator: Send + Sync + 'static {
    fn validate_installation_owner(&self, administrator: &InitialAdministrator) -> Result<(), InstallationOwnerValidationFailure>;
}

/// Removes all data from the validated PostgreSQL and Redis targets before a
/// new initial installation migrates its schema.
#[async_trait]
pub trait InitialInstallationDataResetter: Send + Sync + 'static {
    async fn reset_initial_data(&self, postgres: &PostgresConnection, redis: &RedisConnection) -> Result<(), SetupPortFailure>;
}

#[async_trait]
pub trait InitialInstallationMigrator: Send + Sync + 'static {
    async fn migrate_initial_schema(&self, connection: &PostgresConnection) -> Result<(), SetupPortFailure>;
}

#[async_trait]
pub trait InstallationOwnerProvisioner: Send + Sync + 'static {
    async fn provision_installation_owner(&self, connection: &PostgresConnection, administrator: &InitialAdministrator)
    -> Result<(), OwnerProvisioningFailure>;
}

#[async_trait]
pub trait InstallationStateWriter: Send + Sync + 'static {
    async fn write_completed_installation(&self, installation: PersistedInstallation) -> Result<(), InstallationStateWriteFailure>;
}

#[async_trait]
pub trait ShutdownSignal: Send + Sync + 'static {
    async fn request_shutdown(&self) -> Result<(), SetupPortFailure>;
}

pub trait JwtSecretGenerator: Send + Sync + 'static {
    fn generate_jwt_secret(&self) -> Result<String, SetupPortFailure>;
}
