use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use configuration::PersistedInstallation;

use crate::{
    application::{
        InitialInstallationDataResetter, InitialInstallationMigrator, InstallationOwnerProvisioner, InstallationOwnerValidationFailure,
        InstallationOwnerValidator, InstallationStateWriteFailure, InstallationStateWriter, JwtSecretGenerator, OwnerProvisioningFailure,
        PostgresConnectionTester, RedisConnectionTester, SetupDependencies, SetupInstallationInput, SetupInstallationInputParts, SetupPortFailure,
        SetupService, ShutdownSignal,
    },
    domain::{
        AdvancedSetupOverrides, InitialAdministrator, InitialAdministratorInput, PostgresConnection, PostgresConnectionInput, RedisConnection,
        RedisConnectionInput,
    },
};

pub(super) const TEST_JWT_SECRET: &str = "0123456789abcdef0123456789abcdef";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Call {
    ValidateOwner,
    PostgresTest,
    RedisTest,
    ResetData,
    Migrate,
    ProvisionOwner,
    GenerateJwt,
    WriteState,
    Shutdown,
}

#[derive(Clone, Copy)]
pub(super) enum FailureStage {
    OwnerValidation,
    Migration,
    DataReset,
    OwnerProvisioning,
    StateWrite,
}

pub(super) struct TestPort {
    failure: Option<FailureStage>,
    calls: Mutex<Vec<Call>>,
    installation: Mutex<Option<PersistedInstallation>>,
}

impl TestPort {
    fn new(failure: Option<FailureStage>) -> Self {
        Self {
            failure,
            calls: Mutex::new(Vec::new()),
            installation: Mutex::new(None),
        }
    }

    pub(super) fn calls(&self) -> Vec<Call> {
        self.calls.lock().unwrap().clone()
    }

    pub(super) fn written_installation(&self) -> Option<PersistedInstallation> {
        self.installation.lock().unwrap().clone()
    }

    fn record(&self, call: Call) {
        self.calls.lock().unwrap().push(call);
    }
}

#[async_trait]
impl PostgresConnectionTester for TestPort {
    async fn test_postgres_connection(&self, _: &PostgresConnection) -> Result<(), SetupPortFailure> {
        self.record(Call::PostgresTest);
        Ok(())
    }
}

#[async_trait]
impl RedisConnectionTester for TestPort {
    async fn test_redis_connection(&self, _: &RedisConnection) -> Result<(), SetupPortFailure> {
        self.record(Call::RedisTest);
        Ok(())
    }
}

impl InstallationOwnerValidator for TestPort {
    fn validate_installation_owner(&self, _: &InitialAdministrator) -> Result<(), InstallationOwnerValidationFailure> {
        self.record(Call::ValidateOwner);
        match self.failure {
            Some(FailureStage::OwnerValidation) => Err(InstallationOwnerValidationFailure),
            _ => Ok(()),
        }
    }
}

#[async_trait]
impl InitialInstallationDataResetter for TestPort {
    async fn reset_initial_data(&self, _: &PostgresConnection, _: &RedisConnection) -> Result<(), SetupPortFailure> {
        self.record(Call::ResetData);
        match self.failure {
            Some(FailureStage::DataReset) => Err(SetupPortFailure),
            _ => Ok(()),
        }
    }
}

#[async_trait]
impl InitialInstallationMigrator for TestPort {
    async fn migrate_initial_schema(&self, _: &PostgresConnection) -> Result<(), SetupPortFailure> {
        self.record(Call::Migrate);
        match self.failure {
            Some(FailureStage::Migration) => Err(SetupPortFailure),
            _ => Ok(()),
        }
    }
}

#[async_trait]
impl InstallationOwnerProvisioner for TestPort {
    async fn provision_installation_owner(&self, _: &PostgresConnection, _: &InitialAdministrator) -> Result<(), OwnerProvisioningFailure> {
        self.record(Call::ProvisionOwner);
        match self.failure {
            Some(FailureStage::OwnerProvisioning) => Err(OwnerProvisioningFailure::Failed),
            _ => Ok(()),
        }
    }
}

#[async_trait]
impl InstallationStateWriter for TestPort {
    async fn write_completed_installation(&self, installation: PersistedInstallation) -> Result<(), InstallationStateWriteFailure> {
        self.record(Call::WriteState);
        if matches!(self.failure, Some(FailureStage::StateWrite)) {
            return Err(InstallationStateWriteFailure::Failed);
        }
        *self.installation.lock().unwrap() = Some(installation);
        Ok(())
    }
}

#[async_trait]
impl ShutdownSignal for TestPort {
    async fn request_shutdown(&self) -> Result<(), SetupPortFailure> {
        self.record(Call::Shutdown);
        Ok(())
    }
}

impl JwtSecretGenerator for TestPort {
    fn generate_jwt_secret(&self) -> Result<String, SetupPortFailure> {
        self.record(Call::GenerateJwt);
        Ok(TEST_JWT_SECRET.into())
    }
}

pub(super) fn service_with(failure: Option<FailureStage>) -> (SetupService, Arc<TestPort>) {
    let port = Arc::new(TestPort::new(failure));
    let dependencies = SetupDependencies {
        installation_owner_validator: port.clone(),
        postgres_tester: port.clone(),
        redis_tester: port.clone(),
        data_resetter: port.clone(),
        migrator: port.clone(),
        owner_provisioner: port.clone(),
        state_writer: port.clone(),
        jwt_secret_generator: port.clone(),
        shutdown: port.clone(),
    };
    (SetupService::new(dependencies), port)
}

pub(super) fn installation_input(advanced: AdvancedSetupOverrides) -> SetupInstallationInput {
    installation_input_with_tls(advanced, true, true)
}

pub(super) fn installation_input_with_tls(advanced: AdvancedSetupOverrides, postgres_tls: bool, redis_tls: bool) -> SetupInstallationInput {
    SetupInstallationInput::new(SetupInstallationInputParts {
        postgres: PostgresConnection::new(PostgresConnectionInput {
            host: "postgres.internal".into(),
            port: 5_432,
            username: "taco".into(),
            password: "postgres-secret".into(),
            database: "taco".into(),
            use_tls: postgres_tls,
        })
        .unwrap(),
        redis: RedisConnection::new(RedisConnectionInput {
            host: "redis.internal".into(),
            port: 6_379,
            username: None,
            password: None,
            database: None,
            use_tls: redis_tls,
        })
        .unwrap(),
        administrator: InitialAdministrator::new(InitialAdministratorInput {
            username: "owner".into(),
            email: "owner@example.test".into(),
            password: "owner-secret".into(),
        })
        .unwrap(),
        advanced,
    })
    .unwrap()
}
