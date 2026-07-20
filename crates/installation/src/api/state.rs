use std::sync::Arc;

use crate::application::{InstallationStatus, SetupUseCase};

#[derive(Clone, Copy)]
pub(super) struct InstallationApiState {
    pub(super) status: InstallationStatus,
}

impl InstallationApiState {
    pub(super) const fn new(status: InstallationStatus) -> Self {
        Self { status }
    }
}

#[derive(Clone)]
pub struct SetupApiState {
    pub(super) status: InstallationStatus,
    pub(super) setup: Arc<dyn SetupUseCase>,
}

impl SetupApiState {
    pub fn new(setup: Arc<dyn SetupUseCase>) -> Self {
        Self {
            status: InstallationStatus::setup(),
            setup,
        }
    }
}
