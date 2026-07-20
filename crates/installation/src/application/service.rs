use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use async_trait::async_trait;
use configuration::PersistedInstallation;
use tokio::sync::Mutex;

use crate::domain::{PostgresConnection, RedisConnection};

use super::{
    ExistingInstallationDetector, InitialInstallationDataResetter, InitialInstallationMigrator, InstallationOwnerProvisioner, InstallationOwnerValidator,
    InstallationStateWriteFailure, InstallationStateWriter, JwtSecretGenerator, OwnerProvisioningFailure, PostgresConnectionTester, RedisConnectionTester,
    SetupError, SetupInstallationInput, SetupInstallationInputParts, ShutdownSignal, installation_profile, valid_jwt_secret,
};

#[async_trait]
pub trait SetupUseCase: Send + Sync + 'static {
    async fn test_postgres(&self, connection: PostgresConnection) -> Result<(), SetupError>;
    async fn test_redis(&self, connection: RedisConnection) -> Result<(), SetupError>;
    async fn install(&self, input: SetupInstallationInput) -> Result<InstallationCompleted, SetupError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstallationCompleted;

#[derive(Clone)]
pub struct SetupService {
    dependencies: SetupDependencies,
    install_lock: Arc<Mutex<()>>,
    completed: Arc<AtomicBool>,
}

#[derive(Clone)]
pub struct SetupDependencies {
    pub installation_owner_validator: Arc<dyn InstallationOwnerValidator>,
    pub postgres_tester: Arc<dyn PostgresConnectionTester>,
    pub redis_tester: Arc<dyn RedisConnectionTester>,
    pub existing_installation_detector: Arc<dyn ExistingInstallationDetector>,
    pub data_resetter: Arc<dyn InitialInstallationDataResetter>,
    pub migrator: Arc<dyn InitialInstallationMigrator>,
    pub owner_provisioner: Arc<dyn InstallationOwnerProvisioner>,
    pub state_writer: Arc<dyn InstallationStateWriter>,
    pub jwt_secret_generator: Arc<dyn JwtSecretGenerator>,
    pub shutdown: Arc<dyn ShutdownSignal>,
}

impl SetupService {
    pub fn new(dependencies: SetupDependencies) -> Self {
        Self {
            dependencies,
            install_lock: Arc::new(Mutex::new(())),
            completed: Arc::new(AtomicBool::new(false)),
        }
    }

    async fn install_unlocked(&self, input: SetupInstallationInput) -> Result<InstallationCompleted, SetupError> {
        let SetupInstallationInputParts {
            postgres,
            redis,
            administrator,
            advanced,
        } = input.into_parts();
        self.validate_installation_owner(&administrator)?;
        self.test_postgres(postgres.clone()).await?;
        self.ensure_target_is_fresh(&postgres).await?;
        self.test_redis(redis.clone()).await?;
        self.reset_initial_data(&postgres, &redis).await?;
        self.run_migration(&postgres).await?;
        self.provision_owner(&postgres, &administrator).await?;
        let profile = self.build_profile(&postgres, &redis, &advanced)?;
        self.persist(profile).await?;
        self.completed.store(true, Ordering::Release);
        self.request_shutdown().await?;
        Ok(InstallationCompleted)
    }

    async fn run_migration(&self, postgres: &PostgresConnection) -> Result<(), SetupError> {
        self.dependencies
            .migrator
            .migrate_initial_schema(postgres)
            .await
            .map_err(|_| SetupError::MigrationFailed)
    }

    fn validate_installation_owner(&self, administrator: &crate::domain::InitialAdministrator) -> Result<(), SetupError> {
        self.dependencies
            .installation_owner_validator
            .validate_installation_owner(administrator)
            .map_err(|_| SetupError::InstallationOwnerInvalid)
    }

    async fn reset_initial_data(&self, postgres: &PostgresConnection, redis: &RedisConnection) -> Result<(), SetupError> {
        self.dependencies
            .data_resetter
            .reset_initial_data(postgres, redis)
            .await
            .map_err(|_| SetupError::InstallationDataResetFailed)
    }

    async fn ensure_target_is_fresh(&self, postgres: &PostgresConnection) -> Result<(), SetupError> {
        let contains_installation = self
            .dependencies
            .existing_installation_detector
            .has_existing_installation(postgres)
            .await
            .map_err(|_| SetupError::ExistingInstallationDetectionFailed)?;
        if contains_installation {
            return Err(SetupError::ExistingInstallationDetected);
        }
        Ok(())
    }

    async fn provision_owner(&self, postgres: &PostgresConnection, administrator: &crate::domain::InitialAdministrator) -> Result<(), SetupError> {
        self.dependencies
            .owner_provisioner
            .provision_installation_owner(postgres, administrator)
            .await
            .map_err(map_owner_provisioning_failure)
    }

    fn build_profile(
        &self,
        postgres: &PostgresConnection,
        redis: &RedisConnection,
        advanced: &crate::domain::AdvancedSetupOverrides,
    ) -> Result<configuration::InstallationProfile, SetupError> {
        let jwt_secret = self
            .dependencies
            .jwt_secret_generator
            .generate_jwt_secret()
            .map_err(|_| SetupError::JwtGenerationFailed)?;
        if !valid_jwt_secret(&jwt_secret) {
            return Err(SetupError::InvalidGeneratedJwt);
        }
        Ok(installation_profile(postgres, redis, jwt_secret, advanced))
    }

    async fn persist(&self, profile: configuration::InstallationProfile) -> Result<(), SetupError> {
        self.dependencies
            .state_writer
            .write_completed_installation(PersistedInstallation::completed(profile))
            .await
            .map_err(map_state_write_failure)
    }

    async fn request_shutdown(&self) -> Result<(), SetupError> {
        self.dependencies.shutdown.request_shutdown().await.map_err(|_| SetupError::ShutdownFailed)
    }
}

#[async_trait]
impl SetupUseCase for SetupService {
    async fn test_postgres(&self, connection: PostgresConnection) -> Result<(), SetupError> {
        self.dependencies
            .postgres_tester
            .test_postgres_connection(&connection)
            .await
            .map_err(|_| SetupError::PostgresConnectionFailed)
    }

    async fn test_redis(&self, connection: RedisConnection) -> Result<(), SetupError> {
        self.dependencies
            .redis_tester
            .test_redis_connection(&connection)
            .await
            .map_err(|_| SetupError::RedisConnectionFailed)
    }

    async fn install(&self, input: SetupInstallationInput) -> Result<InstallationCompleted, SetupError> {
        let _guard = self.install_lock.lock().await;
        if self.completed.load(Ordering::Acquire) {
            return Err(SetupError::AlreadyInstalled);
        }
        self.install_unlocked(input).await
    }
}

fn map_owner_provisioning_failure(error: OwnerProvisioningFailure) -> SetupError {
    match error {
        OwnerProvisioningFailure::Failed => SetupError::InstallationOwnerProvisioningFailed,
    }
}

fn map_state_write_failure(error: InstallationStateWriteFailure) -> SetupError {
    match error {
        InstallationStateWriteFailure::AlreadyExists => SetupError::InstallationStateAlreadyExists,
        InstallationStateWriteFailure::Failed => SetupError::InstallationStatePersistenceFailed,
    }
}
