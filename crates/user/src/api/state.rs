use std::sync::Arc;

use crate::{api::TokenService, application::UserUseCase};

#[derive(Clone)]
pub struct ApiState {
    pub users: Arc<dyn UserUseCase>,
    pub tokens: TokenService,
}

impl ApiState {
    pub fn new(users: Arc<dyn UserUseCase>, tokens: TokenService) -> Self {
        Self { users, tokens }
    }
}
