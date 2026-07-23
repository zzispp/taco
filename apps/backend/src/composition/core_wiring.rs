use std::{io::Error, sync::Arc, time::Duration};

use ::system::{
    application::{ServerMetricsUseCase, SystemAuditedUseCase, SystemMetricsService, SystemService, SystemUseCase},
    infra::{RedisSystemCache, StorageSystemRepository, SysinfoServerMetricsCollector},
    notice::{NoticeAuditedUseCase, NoticeService, NoticeUseCase, StorageNoticeRepository},
};
use client_info::{IpLocationClientConfig, IpLocationResolver, PconlineIpLocationResolver};
use configuration::Settings;
use storage::Database;
use user::{
    api::{TokenService, TokenSettings},
    application::{BootstrapAdministratorInput, BootstrapAdministratorOutcome, BootstrapAdministratorRepository, UserService, UserUseCase},
    infra::{
        Argon2PasswordHasher, OnlineSessionCleanupConfig, OnlineSessionCleanupRuntimeHandle, OnlineSessionCleanupRuntimeParts, RedisLoginFailureStore,
        StorageOnlineSessionStore, StorageUserRepository, start_online_session_cleanup_runtime,
    },
};

use super::runtime_config::RuntimeUserConfig;
use crate::BackendResult;

const ADMINISTRATOR_BOOTSTRAP_COMMAND: &str = "taco --config <path> administrator bootstrap --username <username> --email <email> --password-stdin";

pub(super) struct SystemServices {
    pub use_case: Arc<dyn SystemUseCase>,
    pub audited: Arc<dyn SystemAuditedUseCase>,
    pub notices: Arc<dyn NoticeUseCase>,
    pub notices_audited: Arc<dyn NoticeAuditedUseCase>,
    pub metrics: Arc<dyn ServerMetricsUseCase>,
}

pub(super) struct UserServices {
    pub use_case: Arc<dyn UserUseCase>,
    pub tokens: TokenService,
    pub location_resolver: Arc<dyn IpLocationResolver>,
    pub session_cleanup_runtime: OnlineSessionCleanupRuntimeHandle,
}

pub(super) async fn build_system_services(
    settings: &Settings,
    database: Database,
    observer: taco_tracing::InfrastructureObserver,
) -> BackendResult<SystemServices> {
    let cache = RedisSystemCache::connect(&settings.redis_url()?, settings.redis.key_prefix.clone(), observer).await?;
    let service = Arc::new(SystemService::with_cache(StorageSystemRepository::new(database.clone()), cache));
    let use_case: Arc<dyn SystemUseCase> = service.clone();
    let audited: Arc<dyn SystemAuditedUseCase> = service;
    rebuild_system_cache(&use_case).await?;
    let notice_service = Arc::new(NoticeService::new(StorageNoticeRepository::new(database)));
    Ok(SystemServices {
        use_case,
        audited,
        notices: notice_service.clone(),
        notices_audited: notice_service,
        metrics: Arc::new(SystemMetricsService::new(SysinfoServerMetricsCollector)),
    })
}

pub(super) async fn build_user_services(
    settings: &Settings,
    database: Database,
    system: Arc<dyn SystemUseCase>,
    observer: taco_tracing::InfrastructureObserver,
) -> BackendResult<UserServices> {
    let runtime_config = RuntimeUserConfig::new(system);
    let user_repository = StorageUserRepository::new(database.clone());
    let user_service = UserService::with_password_policy(user_repository, Argon2PasswordHasher, runtime_config.clone());
    let client_info = settings.client_info_config()?;
    let location_resolver: Arc<dyn IpLocationResolver> = Arc::new(PconlineIpLocationResolver::new(
        Arc::new(runtime_config.clone()),
        IpLocationClientConfig {
            request_timeout: Duration::from_millis(client_info.ip_location.request_timeout_ms),
        },
        observer.clone(),
    )?);
    let online_sessions = Arc::new(StorageOnlineSessionStore::new(database.clone()));
    let session_cleanup_runtime = start_online_session_cleanup_runtime(OnlineSessionCleanupRuntimeParts {
        cleanup: online_sessions.clone(),
        config: online_session_cleanup_config(settings)?,
    })?;
    let login_failures = RedisLoginFailureStore::connect(&settings.redis_url()?, settings.redis.key_prefix.clone(), observer).await?;
    let use_case = user_service.with_login_security(login_failures, runtime_config.clone());
    Ok(UserServices {
        use_case: Arc::new(use_case),
        tokens: TokenService::with_ttl_reader(token_settings(settings)?, Arc::new(runtime_config), online_sessions),
        location_resolver,
        session_cleanup_runtime,
    })
}

pub(crate) async fn bootstrap_administrator(database: Database, input: BootstrapAdministratorInput) -> BackendResult<BootstrapAdministratorOutcome> {
    let system: Arc<dyn SystemUseCase> = Arc::new(SystemService::new(StorageSystemRepository::new(database.clone())));
    let runtime_config = RuntimeUserConfig::new(system);
    let repository = StorageUserRepository::new(database);
    let service = UserService::with_password_policy(repository, Argon2PasswordHasher, runtime_config);
    Ok(service.bootstrap_administrator(input).await?)
}

pub(crate) async fn ensure_enabled_system_administrator(database: Database) -> BackendResult<()> {
    let repository = StorageUserRepository::new(database);
    if repository.has_enabled_system_administrator().await? {
        return Ok(());
    }
    Err(missing_system_administrator_error().into())
}

fn online_session_cleanup_config(settings: &Settings) -> BackendResult<OnlineSessionCleanupConfig> {
    let config = settings.online_session_config()?;
    Ok(OnlineSessionCleanupConfig {
        interval: Duration::from_millis(config.cleanup_interval_ms),
        batch_size: config.cleanup_batch_size,
    })
}

fn token_settings(settings: &Settings) -> BackendResult<TokenSettings> {
    Ok(TokenSettings {
        secret: settings.jwt_secret()?,
    })
}

fn missing_system_administrator_error() -> Error {
    Error::other(format!(
        "no enabled protected system administrator exists; run `{ADMINISTRATOR_BOOTSTRAP_COMMAND}` before starting Taco"
    ))
}

async fn rebuild_system_cache(system: &Arc<dyn SystemUseCase>) -> BackendResult<()> {
    system.refresh_config_cache().await?;
    system.refresh_dict_cache().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ADMINISTRATOR_BOOTSTRAP_COMMAND, missing_system_administrator_error};

    #[test]
    fn missing_administrator_error_provides_the_recovery_command() {
        let error = missing_system_administrator_error();

        assert_eq!(
            error.to_string(),
            format!("no enabled protected system administrator exists; run `{ADMINISTRATOR_BOOTSTRAP_COMMAND}` before starting Taco")
        );
    }
}
