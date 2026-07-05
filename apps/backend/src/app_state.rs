use std::sync::Arc;

use rbac::application::{AuthorizationConfig, RbacAdminUseCase, RbacUseCase};
use system::application::SystemUseCase;
use user::{api::TokenService, application::UserUseCase};

pub struct AppState {
    pub users: Arc<dyn UserUseCase>,
    pub tokens: TokenService,
    pub rbac: Arc<dyn RbacUseCase>,
    pub rbac_admin: Arc<dyn RbacAdminUseCase>,
    pub system: Arc<dyn SystemUseCase>,
    pub authorization: AuthorizationConfig,
}
