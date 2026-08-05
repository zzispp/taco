use std::sync::Arc;

use ::system::application::SystemUseCase;
use captcha::{
    application::{CaptchaProvider, CaptchaService, CaptchaUseCase},
    infra::RedisCaptchaStore,
    providers::cap::CapProvider,
};
use configuration::Settings;
use storage::{Database, connect_database};

use super::{
    access_catalog::EndpointCatalog,
    audit_wiring::AuditServices,
    core_wiring::{SystemServices, UserServices, build_system_services, ensure_enabled_system_administrator},
    file_wiring::FileServices,
    rbac_wiring::{RbacServices, build_rbac_services},
    routes::authorization_config,
    runtime_config::CaptchaSystemConfig,
    scheduler_wiring::SchedulerServices,
    tracing_runtime::{ObservabilityServices, build_observability_services, observability_export_config},
};
use crate::{BackendResult, app_state::AppState, migration};

pub(super) struct RuntimeFoundation {
    pub(super) database: Database,
    pub(super) observability: ObservabilityServices,
    pub(super) rbac: RbacServices,
    pub(super) system: SystemServices,
    pub(super) authorization: rbac::application::AuthorizationConfig,
    pub(super) endpoints: EndpointCatalog,
}

pub(super) struct AppStateAssembly {
    pub(super) users: UserServices,
    pub(super) rbac: RbacServices,
    pub(super) system: SystemServices,
    pub(super) files: FileServices,
    pub(super) audit: AuditServices,
    pub(super) observability: ObservabilityServices,
    pub(super) scheduler: SchedulerServices,
    pub(super) captcha: Arc<dyn CaptchaUseCase>,
    pub(super) authorization: rbac::application::AuthorizationConfig,
    pub(super) endpoints: EndpointCatalog,
}

pub(super) async fn build_runtime_foundation(settings: &Settings) -> BackendResult<RuntimeFoundation> {
    let database = connect_database(&settings.database_url()?).await?;
    migration::ensure_runtime_schema_ready(database.raw_pool()).await?;
    ensure_enabled_system_administrator(database.clone()).await?;
    let observability = build_observability_services(database.clone()).await?;
    let rbac = build_rbac_services(settings, database.clone(), observability.infrastructure_observer.clone()).await?;
    let endpoints = EndpointCatalog::build()?;
    let authorization = authorization_config(&endpoints)?;
    rbac.use_case.validate_protected_handlers(&authorization)?;
    let system = build_system_services(settings, database.clone(), observability.infrastructure_observer.clone()).await?;
    Ok(RuntimeFoundation {
        database,
        observability,
        rbac,
        system,
        authorization,
        endpoints,
    })
}

pub(super) async fn build_captcha_service(
    settings: &Settings,
    system: Arc<dyn SystemUseCase>,
    observer: taco_tracing::InfrastructureObserver,
) -> BackendResult<Arc<dyn CaptchaUseCase>> {
    let store = RedisCaptchaStore::connect(&settings.redis_url()?, settings.redis.key_prefix.clone(), observer).await?;
    let providers: Vec<Arc<dyn CaptchaProvider>> = vec![Arc::new(CapProvider::new(store))];
    Ok(Arc::new(CaptchaService::new(CaptchaSystemConfig::new(system), providers)))
}

pub(super) fn assemble_app_state(parts: AppStateAssembly) -> AppState {
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
