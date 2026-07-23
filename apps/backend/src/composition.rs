use std::sync::Arc;

use ::system::application::SystemUseCase;
use captcha::{
    application::{CaptchaProvider, CaptchaService, CaptchaUseCase},
    infra::RedisCaptchaStore,
    providers::cap::CapProvider,
};
use configuration::Settings;
use storage::connect_database;

use self::{
    audit_wiring::{AuditServiceParts, AuditServices, audit_outbox_config, build_audit_services},
    core_wiring::{SystemServices, UserServices, build_system_services, build_user_services},
    file_wiring::{FileServices, build_file_services},
    rbac_wiring::{RbacServices, build_rbac_services},
    routes::authorization_config,
    runtime_config::{CaptchaSystemConfig, RuntimeFileConfig},
    scheduler_wiring::{SchedulerServices, build_scheduler_services},
    tracing_runtime::{ObservabilityServices, build_observability_services, observability_export_config},
};
use crate::{BackendResult, app_state::AppState, migration};

pub(crate) mod access_catalog;
mod audit_wiring;
mod core_wiring;
mod file_wiring;
pub(crate) mod http_pipeline;
mod rbac_wiring;
mod router_wiring;
mod routes;
mod runtime_config;
mod scheduler_wiring;
mod system_log_cleanup_execution;
#[cfg(test)]
pub(crate) mod tests;
pub(crate) mod tracing_config_listener;
pub(crate) mod tracing_runtime;

pub(crate) use core_wiring::bootstrap_administrator;
pub(crate) use core_wiring::ensure_enabled_system_administrator;
#[cfg(test)]
pub(crate) use router_wiring::build_public_router;
pub use router_wiring::create_app;

struct AppStateAssembly {
    users: UserServices,
    rbac: RbacServices,
    system: SystemServices,
    files: FileServices,
    audit: AuditServices,
    observability: ObservabilityServices,
    scheduler: SchedulerServices,
    captcha: Arc<dyn CaptchaUseCase>,
    authorization: rbac::application::AuthorizationConfig,
    endpoints: access_catalog::EndpointCatalog,
}

pub async fn build_app_state(settings: &Settings) -> BackendResult<AppState> {
    let database = connect_database(&settings.database_url()?).await?;
    migration::ensure_runtime_schema_ready(database.raw_pool()).await?;
    ensure_enabled_system_administrator(database.clone()).await?;
    let observability = build_observability_services(database.clone()).await?;
    let rbac = build_rbac_services(settings, database.clone(), observability.infrastructure_observer.clone()).await?;
    let endpoints = access_catalog::EndpointCatalog::build()?;
    let authorization = authorization_config(&endpoints)?;
    rbac.use_case.validate_protected_handlers(&authorization)?;
    let system = build_system_services(settings, database.clone(), observability.infrastructure_observer.clone()).await?;
    let files = build_file_services(
        &settings.data_directory,
        database.clone(),
        Arc::new(RuntimeFileConfig::new(system.use_case.clone())),
    )?;
    let users = build_user_services(
        settings,
        database.clone(),
        system.use_case.clone(),
        observability.infrastructure_observer.clone(),
    )
    .await?;
    let captcha = build_captcha_service(settings, system.use_case.clone(), observability.infrastructure_observer.clone()).await?;
    let scheduler = build_scheduler_services(
        settings,
        database.clone(),
        system.use_case.clone(),
        observability.logs.clone(),
        observability.retention.clone(),
        files.cleanup.clone(),
        observability.infrastructure_observer.clone(),
    )?;
    let audit = build_audit_services(AuditServiceParts {
        database,
        system: system.use_case.clone(),
        location_resolver: users.location_resolver.clone(),
        outbox: audit_outbox_config(settings)?,
    })?;
    Ok(assemble_app_state(AppStateAssembly {
        users,
        rbac,
        system,
        files,
        audit,
        observability,
        scheduler,
        captcha,
        authorization,
        endpoints,
    }))
}

fn assemble_app_state(parts: AppStateAssembly) -> AppState {
    let AppStateAssembly {
        users,
        rbac,
        system,
        files,
        audit,
        observability,
        scheduler,
        captcha,
        authorization,
        endpoints,
    } = parts;
    let system_log_export_config = observability_export_config(system.use_case.clone());
    AppState {
        users: users.use_case,
        tokens: users.tokens,
        session_cleanup_runtime: users.session_cleanup_runtime,
        rbac: rbac.use_case,
        rbac_admin: rbac.admin,
        rbac_audited_admin: rbac.audited_admin,
        rbac_cache_refresher: rbac.cache_refresher,
        system: system.use_case,
        system_audited: system.audited,
        notices: system.notices,
        notices_audited: system.notices_audited,
        metrics: system.metrics,
        captcha,
        files: files.use_case,
        audit: audit.use_case,
        audit_outbox: audit.outbox,
        audit_outbox_runtime: audit.runtime,
        audit_export_config: audit.export_config,
        system_logs: observability.logs,
        system_log_exporter: observability.exporter,
        system_log_export_config,
        system_log_runtime: observability.system_log_runtime,
        _tracing_config_listener_runtime: observability.config_listener_runtime,
        tracing_config_listener_health: observability.config_listener_health,
        http_log_state: observability.http_log_state,
        ip_location_resolver: users.location_resolver,
        scheduler: scheduler.use_case,
        scheduler_audited: scheduler.audited,
        scheduler_export_config: scheduler.export_config,
        scheduler_runtime: scheduler.runtime,
        authorization,
        endpoints,
    }
}

async fn build_captcha_service(
    settings: &Settings,
    system: Arc<dyn SystemUseCase>,
    observer: taco_tracing::InfrastructureObserver,
) -> BackendResult<Arc<dyn CaptchaUseCase>> {
    let store = RedisCaptchaStore::connect(&settings.redis_url()?, settings.redis.key_prefix.clone(), observer).await?;
    let providers: Vec<Arc<dyn CaptchaProvider>> = vec![Arc::new(CapProvider::new(store))];
    Ok(Arc::new(CaptchaService::new(CaptchaSystemConfig::new(system), providers)))
}
