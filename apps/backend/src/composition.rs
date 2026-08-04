use configuration::Settings;

use self::{
    app_state_wiring::{AppStateAssembly, assemble_app_state, build_captcha_service, build_runtime_foundation},
    audit_wiring::{AuditServiceParts, audit_outbox_config, build_audit_services},
    core_wiring::{UserServicesWiring, build_user_services},
    file_wiring::build_file_services,
    runtime_config::RuntimeFileConfig,
    scheduler_wiring::{SchedulerServicesWiring, build_scheduler_services},
};
use crate::{BackendResult, app_state::AppState};

pub(crate) mod access_catalog;
mod app_state_wiring;
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
#[cfg(test)]
pub(crate) use core_wiring::ensure_enabled_system_administrator;
#[cfg(test)]
pub(crate) use router_wiring::build_public_router;
pub use router_wiring::create_app;

pub async fn build_app_state(settings: &Settings) -> BackendResult<AppState> {
    let foundation = build_runtime_foundation(settings).await?;
    let files = build_file_services(
        &settings.data_directory,
        foundation.database.clone(),
        std::sync::Arc::new(RuntimeFileConfig::new(foundation.system.use_case.clone())),
    )?;
    let users = build_user_services(UserServicesWiring {
        settings,
        database: foundation.database.clone(),
        system: foundation.system.use_case.clone(),
        observer: foundation.observability.infrastructure_observer.clone(),
    })
    .await?;
    let captcha = build_captcha_service(
        settings,
        foundation.system.use_case.clone(),
        foundation.observability.infrastructure_observer.clone(),
    )
    .await?;
    let scheduler = build_scheduler_services(SchedulerServicesWiring {
        settings,
        database: foundation.database.clone(),
        system: foundation.system.use_case.clone(),
        logs: foundation.observability.logs.clone(),
        retention: foundation.observability.retention.clone(),
        file_cleanup: files.cleanup.clone(),
        observer: foundation.observability.infrastructure_observer.clone(),
    })?;
    let audit = build_audit_services(AuditServiceParts {
        database: foundation.database,
        system: foundation.system.use_case.clone(),
        location_resolver: users.location_resolver.clone(),
        outbox: audit_outbox_config(settings)?,
    })?;
    Ok(assemble_app_state(AppStateAssembly {
        users,
        rbac: foundation.rbac,
        system: foundation.system,
        files,
        audit,
        observability: foundation.observability,
        scheduler,
        captcha,
        authorization: foundation.authorization,
        endpoints: foundation.endpoints,
    }))
}
