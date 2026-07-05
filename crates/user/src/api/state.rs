use std::sync::Arc;

use rbac::application::RbacUseCase;

use crate::{api::TokenService, application::UserUseCase};

#[derive(Clone)]
pub struct ApiState {
    pub users: Arc<dyn UserUseCase>,
    pub tokens: TokenService,
    pub rbac: Arc<dyn RbacUseCase>,
}

impl ApiState {
    pub fn new(users: Arc<dyn UserUseCase>, tokens: TokenService, rbac: Arc<dyn RbacUseCase>) -> Self {
        Self { users, tokens, rbac }
    }
}
