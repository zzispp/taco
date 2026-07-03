use std::sync::Arc;

use crate::application::{RbacAdminUseCase, RbacUseCase};

#[derive(Clone)]
pub struct RbacApiState {
    pub rbac: Arc<dyn RbacUseCase>,
    pub rbac_admin: Arc<dyn RbacAdminUseCase>,
}

impl RbacApiState {
    pub fn new(rbac: Arc<dyn RbacUseCase>, rbac_admin: Arc<dyn RbacAdminUseCase>) -> Self {
        Self { rbac, rbac_admin }
    }
}
