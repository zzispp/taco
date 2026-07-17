use std::sync::Arc;

use ::system::{
    application::{SystemService, SystemUseCase},
    infra::StorageSystemRepository,
};
use configuration::Settings;
use storage::connect_database;
use user::{
    application::{AdminBootstrapUseCase, BootstrapAdminInput, UserService},
    domain::User,
    infra::{Argon2PasswordHasher, StorageUserRepository},
};

use super::runtime_config::RuntimeUserConfig;
use crate::{BackendResult, migration};

pub(crate) async fn bootstrap_admin(settings: &Settings, input: BootstrapAdminInput) -> BackendResult<User> {
    let database = connect_database(&settings.database_url()?).await?;
    migration::prepare_runtime_schema(database.raw_pool(), false).await?;
    let system: Arc<dyn SystemUseCase> = Arc::new(SystemService::new(StorageSystemRepository::new(database.clone())));
    let config = RuntimeUserConfig::new(system);
    let users = UserService::with_password_policy(StorageUserRepository::new(database), Argon2PasswordHasher, config);
    users.bootstrap_admin(input).await.map_err(Into::into)
}
