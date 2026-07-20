use async_trait::async_trait;
use configuration::{BootstrapInputs, ConfigEncryptionKey, DataDirectory, InstallationStateRead, InstallationStateStore, PersistedInstallation};
use installation::{
    application::{
        ExistingInstallationDetector, InitialInstallationDataResetter, InitialInstallationMigrator, InstallationOwnerProvisioner,
        InstallationOwnerValidationFailure, InstallationOwnerValidator, InstallationStateWriteFailure, InstallationStateWriter, OwnerProvisioningFailure,
        PostgresConnectionTester, RedisConnectionTester, SetupPortFailure, postgres_connection_settings, redis_connection_settings,
    },
    domain::{InitialAdministrator, PostgresConnection, RedisConnection},
};
use sqlx::query_scalar;
use storage::{Database, connect_database};
use user::{
    application::{AppError, InstallationOwnerInput, InstallationOwnerUseCase, UserService, validate_initial_installation_owner},
    infra::{Argon2PasswordHasher, StorageUserRepository},
};

use crate::migration;

const POSTGRES_CONNECTION_STAGE: &str = "postgres_connection";
const EXISTING_INSTALLATION_STAGE: &str = "existing_installation_detection";
const REDIS_CONNECTION_STAGE: &str = "redis_connection";
const POSTGRES_RESET_STAGE: &str = "postgres_reset";
const REDIS_RESET_STAGE: &str = "redis_reset";
const MIGRATION_STAGE: &str = "migration";
const OWNER_PROVISIONING_STAGE: &str = "owner_provisioning";
const STATE_READ_STAGE: &str = "installation_state_read";
const STATE_WRITE_STAGE: &str = "installation_state_write";

pub(super) struct SetupInfrastructure {
    data_dir: DataDirectory,
    config_encryption_key: ConfigEncryptionKey,
}

impl SetupInfrastructure {
    pub(super) fn new(bootstrap: &BootstrapInputs) -> Self {
        Self {
            data_dir: bootstrap.data_dir.clone(),
            config_encryption_key: bootstrap.config_encryption_key.clone(),
        }
    }

    async fn database(&self, connection: &PostgresConnection) -> Result<Database, SetupPortFailure> {
        let url = postgres_connection_settings(connection)
            .url()
            .map_err(|error| setup_port_failure(POSTGRES_CONNECTION_STAGE, &error))?;
        connect_database(&url)
            .await
            .map_err(|error| setup_port_failure(POSTGRES_CONNECTION_STAGE, &error))
    }

    async fn redis_manager(&self, connection: &RedisConnection) -> Result<redis::aio::ConnectionManager, SetupPortFailure> {
        let url = redis_connection_settings(connection)
            .url()
            .map_err(|error| setup_port_failure(REDIS_CONNECTION_STAGE, &error))?;
        let client = redis::Client::open(url).map_err(|error| setup_port_failure(REDIS_CONNECTION_STAGE, &error))?;
        client
            .get_connection_manager()
            .await
            .map_err(|error| setup_port_failure(REDIS_CONNECTION_STAGE, &error))
    }
}

#[async_trait]
impl PostgresConnectionTester for SetupInfrastructure {
    async fn test_postgres_connection(&self, connection: &PostgresConnection) -> Result<(), SetupPortFailure> {
        let database = self.database(connection).await?;
        query_scalar::<_, i32>("SELECT 1")
            .fetch_one(database.raw_pool())
            .await
            .map(|_| ())
            .map_err(|error| setup_port_failure(POSTGRES_CONNECTION_STAGE, &error))
    }
}

#[async_trait]
impl ExistingInstallationDetector for SetupInfrastructure {
    async fn has_existing_installation(&self, connection: &PostgresConnection) -> Result<bool, SetupPortFailure> {
        let database = self.database(connection).await?;
        query_scalar::<_, bool>(
            "SELECT to_regclass('public.sys_installation_owner') IS NOT NULL OR (to_regclass('public._sqlx_migrations') IS NOT NULL AND to_regclass('public.sys_user') IS NOT NULL)",
        )
            .fetch_one(database.raw_pool())
            .await
            .map_err(|error| setup_port_failure(EXISTING_INSTALLATION_STAGE, &error))
    }
}

#[async_trait]
impl RedisConnectionTester for SetupInfrastructure {
    async fn test_redis_connection(&self, connection: &RedisConnection) -> Result<(), SetupPortFailure> {
        let mut manager = self.redis_manager(connection).await?;
        let response: String = redis::cmd("PING")
            .query_async(&mut manager)
            .await
            .map_err(|error| setup_port_failure(REDIS_CONNECTION_STAGE, &error))?;
        (response == "PONG")
            .then_some(())
            .ok_or_else(|| unexpected_redis_response(REDIS_CONNECTION_STAGE, "PONG", &response))
    }
}

#[async_trait]
impl InitialInstallationDataResetter for SetupInfrastructure {
    async fn reset_initial_data(&self, postgres: &PostgresConnection, redis: &RedisConnection) -> Result<(), SetupPortFailure> {
        let database = self.database(postgres).await?;
        migration::reset_public_schema(database.raw_pool())
            .await
            .map_err(|error| setup_port_failure(POSTGRES_RESET_STAGE, error.as_ref()))?;
        let mut manager = self.redis_manager(redis).await?;
        let response: String = redis::cmd("FLUSHALL")
            .query_async(&mut manager)
            .await
            .map_err(|error| setup_port_failure(REDIS_RESET_STAGE, &error))?;
        (response == "OK")
            .then_some(())
            .ok_or_else(|| unexpected_redis_response(REDIS_RESET_STAGE, "OK", &response))
    }
}

#[async_trait]
impl InitialInstallationMigrator for SetupInfrastructure {
    async fn migrate_initial_schema(&self, connection: &PostgresConnection) -> Result<(), SetupPortFailure> {
        let database = self.database(connection).await?;
        migration::up(database.raw_pool(), None)
            .await
            .map_err(|error| setup_port_failure(MIGRATION_STAGE, error.as_ref()))
    }
}

impl InstallationOwnerValidator for SetupInfrastructure {
    fn validate_installation_owner(&self, administrator: &InitialAdministrator) -> Result<(), InstallationOwnerValidationFailure> {
        validate_initial_installation_owner(&installation_owner_input(administrator)).map_err(|_| InstallationOwnerValidationFailure)
    }
}

#[async_trait]
impl InstallationOwnerProvisioner for SetupInfrastructure {
    async fn provision_installation_owner(
        &self,
        connection: &PostgresConnection,
        administrator: &InitialAdministrator,
    ) -> Result<(), OwnerProvisioningFailure> {
        let database = self.database(connection).await.map_err(|_| OwnerProvisioningFailure::Failed)?;
        let users = UserService::new(StorageUserRepository::new(database), Argon2PasswordHasher);
        let input = installation_owner_input(administrator);
        users.create_installation_owner(input).await.map(|_| ()).map_err(owner_failure)
    }
}

#[async_trait]
impl InstallationStateWriter for SetupInfrastructure {
    async fn write_completed_installation(&self, installation: PersistedInstallation) -> Result<(), InstallationStateWriteFailure> {
        let store = InstallationStateStore::new(&self.data_dir);
        match store.read::<PersistedInstallation>(&self.config_encryption_key) {
            Ok(InstallationStateRead::Absent) => store
                .write(&self.config_encryption_key, &installation)
                .map_err(|error| installation_state_failure(STATE_WRITE_STAGE, &error)),
            Ok(InstallationStateRead::Present(_)) => Err(InstallationStateWriteFailure::AlreadyExists),
            Err(error) => Err(installation_state_failure(STATE_READ_STAGE, &error)),
        }
    }
}

fn setup_port_failure<E: std::error::Error + ?Sized>(stage: &'static str, error: &E) -> SetupPortFailure {
    taco_tracing::error_with_fields!("setup infrastructure operation failed", error, stage = stage);
    SetupPortFailure
}

fn installation_state_failure<E: std::error::Error + ?Sized>(stage: &'static str, error: &E) -> InstallationStateWriteFailure {
    taco_tracing::error_with_fields!("setup installation state operation failed", error, stage = stage);
    InstallationStateWriteFailure::Failed
}

fn owner_failure(error: AppError) -> OwnerProvisioningFailure {
    taco_tracing::error_with_fields!("setup installation owner provisioning failed", &error, stage = OWNER_PROVISIONING_STAGE);
    OwnerProvisioningFailure::Failed
}

fn installation_owner_input(administrator: &InitialAdministrator) -> InstallationOwnerInput {
    InstallationOwnerInput {
        username: administrator.username().into(),
        email: administrator.email().into(),
        password: administrator.password().into(),
    }
}

fn unexpected_redis_response(stage: &'static str, expected: &str, actual: &str) -> SetupPortFailure {
    taco_tracing::error_with_fields!(
        "setup Redis operation returned an unexpected response",
        stage = stage,
        expected = expected,
        actual = actual
    );
    SetupPortFailure
}

#[cfg(test)]
mod tests {
    use std::{net::SocketAddr, path::PathBuf};

    use configuration::{ConfigEncryptionKey, InstallationProfile};
    use installation::{
        application::{
            InstallationOwnerValidationFailure, InstallationOwnerValidator, InstallationStateWriteFailure, InstallationStateWriter,
            postgres_connection_settings, redis_connection_settings,
        },
        domain::{InitialAdministrator, InitialAdministratorInput, PostgresConnection, PostgresConnectionInput, RedisConnection, RedisConnectionInput},
    };

    use super::SetupInfrastructure;

    const TEST_ROOT_KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

    #[test]
    fn setup_connections_preserve_tls_mapping_and_optional_redis_values() {
        let database = postgres_connection_settings(&postgres_connection());
        let redis = redis_connection_settings(&redis_connection());

        assert_eq!(database.ssl_mode, configuration::DatabaseSslMode::VerifyFull);
        assert_eq!(redis.scheme, configuration::RedisScheme::Redis);
        assert_eq!(redis.username, None);
        assert_eq!(redis.password, None);
        assert_eq!(redis.database, Some(0));
    }

    #[tokio::test]
    async fn state_writer_accepts_only_an_absent_installation_state() {
        let directory = tempfile::tempdir().unwrap();
        let writer = SetupInfrastructure::new(&bootstrap(directory.path().to_owned()));
        let installation = configuration::PersistedInstallation::completed(InstallationProfile::default());

        writer.write_completed_installation(installation.clone()).await.unwrap();
        let error = writer.write_completed_installation(installation).await.unwrap_err();

        assert_eq!(error, InstallationStateWriteFailure::AlreadyExists);
    }

    #[test]
    fn installation_owner_validator_reuses_user_owned_preflight_rules() {
        let directory = tempfile::tempdir().unwrap();
        let infrastructure = SetupInfrastructure::new(&bootstrap(directory.path().to_owned()));
        let administrator = InitialAdministrator::new(InitialAdministratorInput {
            username: "owner".into(),
            email: "owner@example.test".into(),
            password: "short".into(),
        })
        .unwrap();

        assert_eq!(
            infrastructure.validate_installation_owner(&administrator),
            Err(InstallationOwnerValidationFailure)
        );
    }

    fn bootstrap(data_dir: PathBuf) -> configuration::BootstrapInputs {
        configuration::BootstrapInputs::new(
            configuration::DataDirectory::new(data_dir).unwrap(),
            ConfigEncryptionKey::parse(TEST_ROOT_KEY).unwrap(),
            "127.0.0.1:3000".parse::<SocketAddr>().unwrap(),
        )
    }

    fn postgres_connection() -> PostgresConnection {
        PostgresConnection::new(PostgresConnectionInput {
            host: "postgres.internal".into(),
            port: 5_432,
            username: "taco".into(),
            password: "secret".into(),
            database: "taco".into(),
            use_tls: true,
        })
        .unwrap()
    }

    fn redis_connection() -> RedisConnection {
        RedisConnection::new(RedisConnectionInput {
            host: "redis.internal".into(),
            port: 6_379,
            username: None,
            password: None,
            database: Some(0),
            use_tls: false,
        })
        .unwrap()
    }
}
