mod detail_presenter;
mod dto;
mod error;
mod export;
mod handlers;
mod input;
mod presentation;
mod presenter;

use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, post, put},
};
use kernel::runtime_config::ExportConfigProvider;

use crate::application::{SchedulerError, SchedulerRuntimeHandle, SchedulerUseCase};

pub use error::SchedulerApiError;

#[derive(Clone)]
pub struct SchedulerApiState {
    pub scheduler: Arc<dyn SchedulerUseCase>,
    pub export_config: Arc<dyn ExportConfigProvider<Error = SchedulerError>>,
    _runtime: SchedulerRuntimeHandle,
}

pub struct SchedulerApiStateParts {
    pub scheduler: Arc<dyn SchedulerUseCase>,
    pub export_config: Arc<dyn ExportConfigProvider<Error = SchedulerError>>,
    pub runtime: SchedulerRuntimeHandle,
}

impl SchedulerApiState {
    pub fn new(parts: SchedulerApiStateParts) -> Self {
        Self {
            scheduler: parts.scheduler,
            export_config: parts.export_config,
            _runtime: parts.runtime,
        }
    }
}

pub fn create_router(state: SchedulerApiState) -> Router {
    Router::new()
        .route("/system/jobs", get(handlers::jobs::list_jobs))
        .route("/system/jobs/export", post(handlers::jobs::export_jobs))
        .route("/system/jobs/importable", get(handlers::jobs::importable_tasks))
        .route("/system/jobs/import", post(handlers::jobs::import_job))
        .route("/system/jobs/cron/next-times", post(handlers::jobs::cron_next_times))
        .route("/system/jobs/batch", delete(handlers::jobs::delete_jobs))
        .route(
            "/system/jobs/{id}",
            get(handlers::jobs::get_job).put(handlers::jobs::replace_job).delete(handlers::jobs::delete_job),
        )
        .route("/system/jobs/{id}/status", put(handlers::jobs::update_job_status))
        .route("/system/jobs/{id}/run", post(handlers::jobs::run_job))
        .route("/system/job-logs", get(handlers::logs::list_job_logs))
        .route("/system/job-logs/export", post(handlers::logs::export_job_logs))
        .route("/system/job-logs/clean", delete(handlers::logs::clear_job_logs))
        .route("/system/job-logs/batch", delete(handlers::logs::delete_job_logs))
        .route("/system/job-logs/{id}/detail", get(handlers::logs::get_job_log_detail))
        .route("/system/job-logs/{id}", get(handlers::logs::get_job_log).delete(handlers::logs::delete_job_log))
        .with_state(state)
}
