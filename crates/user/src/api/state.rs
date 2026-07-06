use std::sync::Arc;

use captcha::application::CaptchaUseCase;
use rbac::application::RbacUseCase;

use crate::{
    api::TokenService,
    application::{SystemConfigProvider, UserUseCase},
};

#[derive(Clone)]
pub struct ApiState {
    pub users: Arc<dyn UserUseCase>,
    pub tokens: TokenService,
    pub rbac: Arc<dyn RbacUseCase>,
    pub config: Arc<dyn SystemConfigProvider>,
    pub captcha: Arc<dyn CaptchaUseCase>,
}

impl ApiState {
    pub fn new(
        users: Arc<dyn UserUseCase>,
        tokens: TokenService,
        rbac: Arc<dyn RbacUseCase>,
        config: Arc<dyn SystemConfigProvider>,
        captcha: Arc<dyn CaptchaUseCase>,
    ) -> Self {
        Self {
            users,
            tokens,
            rbac,
            config,
            captcha,
        }
    }
}
