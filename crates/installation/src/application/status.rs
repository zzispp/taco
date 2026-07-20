use crate::domain::InstallationState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InstallationStatus {
    state: InstallationState,
}

impl InstallationStatus {
    pub const fn setup() -> Self {
        Self::new(InstallationState::Setup)
    }

    pub const fn installed() -> Self {
        Self::new(InstallationState::Installed)
    }

    pub const fn state(self) -> InstallationState {
        self.state
    }

    const fn new(state: InstallationState) -> Self {
        Self { state }
    }
}
