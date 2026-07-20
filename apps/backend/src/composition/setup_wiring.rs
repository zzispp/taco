use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use async_trait::async_trait;
use configuration::BootstrapInputs;
use installation::application::{RandomJwtSecretGenerator, SetupDependencies, SetupPortFailure, SetupService, SetupUseCase, ShutdownSignal};
use tokio::sync::Notify;

mod infrastructure;

use infrastructure::SetupInfrastructure;

#[derive(Clone)]
pub(crate) struct SetupShutdown {
    requested: Arc<AtomicBool>,
    notify: Arc<Notify>,
}

impl SetupShutdown {
    pub(crate) fn new() -> Self {
        Self {
            requested: Arc::new(AtomicBool::new(false)),
            notify: Arc::new(Notify::new()),
        }
    }

    pub(crate) async fn wait_for_request(&self) {
        loop {
            let notified = self.notify.notified();
            if self.requested.load(Ordering::Acquire) {
                return;
            }
            notified.await;
        }
    }
}

#[async_trait]
impl ShutdownSignal for SetupShutdown {
    async fn request_shutdown(&self) -> Result<(), SetupPortFailure> {
        self.requested.store(true, Ordering::Release);
        self.notify.notify_one();
        Ok(())
    }
}

pub(crate) fn build_setup_use_case(bootstrap: &BootstrapInputs, shutdown: SetupShutdown) -> Arc<dyn SetupUseCase> {
    let infrastructure = Arc::new(SetupInfrastructure::new(bootstrap));
    Arc::new(SetupService::new(SetupDependencies {
        installation_owner_validator: infrastructure.clone(),
        postgres_tester: infrastructure.clone(),
        redis_tester: infrastructure.clone(),
        existing_installation_detector: infrastructure.clone(),
        data_resetter: infrastructure.clone(),
        migrator: infrastructure.clone(),
        owner_provisioner: infrastructure.clone(),
        state_writer: infrastructure,
        jwt_secret_generator: Arc::new(RandomJwtSecretGenerator),
        shutdown: Arc::new(shutdown),
    }))
}

#[cfg(test)]
mod tests {
    use installation::application::ShutdownSignal;

    use super::SetupShutdown;

    #[tokio::test]
    async fn setup_shutdown_unblocks_the_server_waiter() {
        let shutdown = SetupShutdown::new();
        let waiter = shutdown.clone();
        let task = tokio::spawn(async move {
            waiter.wait_for_request().await;
        });

        shutdown.request_shutdown().await.unwrap();
        task.await.unwrap();
    }
}
