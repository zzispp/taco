use std::{fs, path::Path, sync::Arc};

use async_trait::async_trait;
use configuration::{BootstrapInputs, InstallationProfile, InstallationStateRead, InstallationStateStore, PersistedInstallation, Settings};
use installation::application::{
    ExistingInstallationVerificationFailure, ExistingInstallationVerifier, InstallationConnections, InstallationRecoveryService, RandomJwtSecretGenerator,
};
use storage::connect_database;
use user::{
    application::{InstallationOwnerUseCase, UserService},
    infra::{Argon2PasswordHasher, StorageUserRepository},
};

use crate::{BackendResult, migration};

pub(super) async fn reconfigure(args: Vec<String>, connections_path: &str) -> BackendResult<()> {
    let bootstrap = BootstrapInputs::load_from_args(args)?;
    let store = InstallationStateStore::new(&bootstrap.data_dir);
    let installation = required_installation_state(&store, &bootstrap)?;
    let connections: InstallationConnections = read_json(connections_path)?;
    let service = recovery_service();
    let recovered = service.reconfigure(installation, connections).await?;
    Settings::from_persisted_installation(recovered.clone(), &bootstrap)?;
    store.write(&bootstrap.config_encryption_key, &recovered)?;
    Ok(())
}

pub(super) async fn recover(args: Vec<String>, profile_path: &str) -> BackendResult<()> {
    let bootstrap = BootstrapInputs::load_from_args(args)?;
    let store = InstallationStateStore::new(&bootstrap.data_dir);
    reject_existing_state(&store, &bootstrap)?;
    let profile = read_json(profile_path)?;
    let service = recovery_service();
    let recovered = service.recover(profile).await?;
    Settings::from_persisted_installation(recovered.clone(), &bootstrap)?;
    store.write(&bootstrap.config_encryption_key, &recovered)?;
    Ok(())
}

fn recovery_service() -> InstallationRecoveryService {
    InstallationRecoveryService::new(Arc::new(RecoveryVerifier), Arc::new(RandomJwtSecretGenerator))
}

fn required_installation_state(store: &InstallationStateStore, bootstrap: &BootstrapInputs) -> BackendResult<PersistedInstallation> {
    match store.read(&bootstrap.config_encryption_key)? {
        InstallationStateRead::Present(installation) => Ok(installation),
        InstallationStateRead::Absent => Err("installation state is absent; use `installation recover --profile <path>`".into()),
    }
}

fn reject_existing_state(store: &InstallationStateStore, bootstrap: &BootstrapInputs) -> BackendResult<()> {
    match store.read::<PersistedInstallation>(&bootstrap.config_encryption_key)? {
        InstallationStateRead::Absent => Ok(()),
        InstallationStateRead::Present(_) => Err("installation state already exists; use `installation reconfigure --connections <path>`".into()),
    }
}

fn read_json<T>(path: impl AsRef<Path>) -> BackendResult<T>
where
    T: serde::de::DeserializeOwned,
{
    let path = path.as_ref();
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes).map_err(|error| format!("invalid recovery configuration {}: {error}", path.display()).into())
}

struct RecoveryVerifier;

#[async_trait]
impl ExistingInstallationVerifier for RecoveryVerifier {
    async fn verify_existing_installation(&self, profile: &InstallationProfile) -> Result<(), ExistingInstallationVerificationFailure> {
        let database = verified_database(profile).await?;
        verify_installation_owner(&database).await?;
        verify_redis(profile).await
    }
}

async fn verified_database(profile: &InstallationProfile) -> Result<storage::Database, ExistingInstallationVerificationFailure> {
    let url = profile.database.url().map_err(|error| recovery_failure("database_url", &error))?;
    let database = connect_database(&url).await.map_err(|error| recovery_failure("database_connection", &error))?;
    migration::ensure_runtime_schema_ready(database.raw_pool())
        .await
        .map_err(|error| recovery_failure("schema_readiness", error.as_ref()))?;
    Ok(database)
}

async fn verify_installation_owner(database: &storage::Database) -> Result<(), ExistingInstallationVerificationFailure> {
    let users = UserService::new(StorageUserRepository::new(database.clone()), Argon2PasswordHasher);
    let has_owner = users
        .has_installation_owner()
        .await
        .map_err(|error| recovery_failure("installation_owner", &error))?;
    has_owner.then_some(()).ok_or(ExistingInstallationVerificationFailure::InstallationOwnerMissing)
}

async fn verify_redis(profile: &InstallationProfile) -> Result<(), ExistingInstallationVerificationFailure> {
    let url = profile.redis.url().map_err(|error| redis_failure("redis_url", &error))?;
    let client = redis::Client::open(url).map_err(|error| redis_failure("redis_client", &error))?;
    let mut manager = client
        .get_connection_manager()
        .await
        .map_err(|error| redis_failure("redis_connection", &error))?;
    let response: String = redis::cmd("PING")
        .query_async(&mut manager)
        .await
        .map_err(|error| redis_failure("redis_ping", &error))?;
    (response == "PONG")
        .then_some(())
        .ok_or(ExistingInstallationVerificationFailure::RedisConnectionFailed)
}

fn recovery_failure(error_stage: &'static str, error: &(dyn std::error::Error + 'static)) -> ExistingInstallationVerificationFailure {
    taco_tracing::error_with_fields!("installation recovery verification failed", error, stage = error_stage);
    ExistingInstallationVerificationFailure::SchemaNotReady
}

fn redis_failure(error_stage: &'static str, error: &(dyn std::error::Error + 'static)) -> ExistingInstallationVerificationFailure {
    taco_tracing::error_with_fields!("installation recovery Redis verification failed", error, stage = error_stage);
    ExistingInstallationVerificationFailure::RedisConnectionFailed
}
