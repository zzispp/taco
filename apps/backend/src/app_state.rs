use std::sync::Arc;

use audit::{
    application::{AuditError, AuditUseCase},
    infra::{AuditOutboxRuntimeHandle, StorageAuditOutboxRepository},
};
use captcha::application::CaptchaUseCase;
use client_info::IpLocationResolver;
use kernel::runtime_config::ExportConfigProvider;
use observability::application::{ObservabilityError, SystemLogUseCase};
use rbac::application::{AuthorizationConfig, RbacAdminUseCase, RbacAuditedAdminUseCase, RbacCacheRefreshUseCase, RbacUseCase};
use scheduler::application::{SchedulerAuditedUseCase, SchedulerError, SchedulerRuntimeHandle, SchedulerUseCase};
use system::application::{ServerMetricsUseCase, SystemAuditedUseCase, SystemUseCase};
use system::notice::{NoticeAuditedUseCase, NoticeUseCase};
use user::{api::TokenService, application::UserUseCase, infra::OnlineSessionCleanupRuntimeHandle};

use crate::composition::access_catalog::EndpointCatalog;

pub struct AppState {
    pub users: Arc<dyn UserUseCase>,
    pub tokens: TokenService,
    pub session_cleanup_runtime: OnlineSessionCleanupRuntimeHandle,
    pub rbac: Arc<dyn RbacUseCase>,
    pub rbac_admin: Arc<dyn RbacAdminUseCase>,
    pub rbac_audited_admin: Arc<dyn RbacAuditedAdminUseCase>,
    pub rbac_cache_refresher: Arc<dyn RbacCacheRefreshUseCase>,
    pub system: Arc<dyn SystemUseCase>,
    pub system_audited: Arc<dyn SystemAuditedUseCase>,
    pub notices: Arc<dyn NoticeUseCase>,
    pub notices_audited: Arc<dyn NoticeAuditedUseCase>,
    pub metrics: Arc<dyn ServerMetricsUseCase>,
    pub captcha: Arc<dyn CaptchaUseCase>,
    pub audit: Arc<dyn AuditUseCase>,
    pub audit_outbox: Arc<StorageAuditOutboxRepository>,
    pub audit_outbox_runtime: AuditOutboxRuntimeHandle,
    pub audit_export_config: Arc<dyn ExportConfigProvider<Error = AuditError>>,
    pub system_logs: Arc<dyn SystemLogUseCase>,
    pub system_log_export_config: Arc<dyn ExportConfigProvider<Error = ObservabilityError>>,
    pub system_log_runtime: Arc<taco_tracing::SystemLogRuntime>,
    pub _tracing_config_listener_runtime: Arc<crate::composition::tracing_runtime::TracingConfigListenerRuntime>,
    pub tracing_config_listener_health: crate::composition::tracing_runtime::TracingConfigListenerHealth,
    pub http_log_state: taco_tracing::HttpLogCaptureState,
    pub ip_location_resolver: Arc<dyn IpLocationResolver>,
    pub scheduler: Arc<dyn SchedulerUseCase>,
    pub scheduler_audited: Arc<dyn SchedulerAuditedUseCase>,
    pub scheduler_export_config: Arc<dyn ExportConfigProvider<Error = SchedulerError>>,
    pub scheduler_runtime: SchedulerRuntimeHandle,
    pub authorization: AuthorizationConfig,
    pub endpoints: EndpointCatalog,
}
