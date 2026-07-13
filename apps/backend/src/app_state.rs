use std::sync::Arc;

use captcha::application::CaptchaUseCase;
use kernel::runtime_config::ExportConfigProvider;
use rbac::application::{AuthorizationConfig, RbacAdminUseCase, RbacUseCase};
use scheduler::application::{SchedulerError, SchedulerRuntimeHandle, SchedulerUseCase};
use system::application::{ServerMetricsUseCase, SystemUseCase};
use user::{api::TokenService, application::UserUseCase};

pub struct AppState {
    pub users: Arc<dyn UserUseCase>,
    pub tokens: TokenService,
    pub rbac: Arc<dyn RbacUseCase>,
    pub rbac_admin: Arc<dyn RbacAdminUseCase>,
    pub system: Arc<dyn SystemUseCase>,
    pub metrics: Arc<dyn ServerMetricsUseCase>,
    pub captcha: Arc<dyn CaptchaUseCase>,
    pub scheduler: Arc<dyn SchedulerUseCase>,
    pub scheduler_export_config: Arc<dyn ExportConfigProvider<Error = SchedulerError>>,
    pub scheduler_runtime: SchedulerRuntimeHandle,
    pub authorization: AuthorizationConfig,
}
