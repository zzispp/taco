mod detail_presenter;
mod dto;
mod endpoints;
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

use crate::application::{SchedulerAuditedUseCase, SchedulerError, SchedulerRuntimeHandle, SchedulerUseCase};

pub use endpoints::endpoint_specs;
pub use error::SchedulerApiError;

#[derive(Clone)]
pub struct SchedulerApiState {
    pub scheduler: Arc<dyn SchedulerUseCase>,
    pub audited_scheduler: Arc<dyn SchedulerAuditedUseCase>,
    pub export_config: Arc<dyn ExportConfigProvider<Error = SchedulerError>>,
    _runtime: SchedulerRuntimeHandle,
}

pub struct SchedulerApiStateParts {
    pub scheduler: Arc<dyn SchedulerUseCase>,
    pub audited_scheduler: Arc<dyn SchedulerAuditedUseCase>,
    pub export_config: Arc<dyn ExportConfigProvider<Error = SchedulerError>>,
    pub runtime: SchedulerRuntimeHandle,
}

impl SchedulerApiState {
    pub fn new(parts: SchedulerApiStateParts) -> Self {
        Self {
            scheduler: parts.scheduler,
            audited_scheduler: parts.audited_scheduler,
            export_config: parts.export_config,
            _runtime: parts.runtime,
        }
    }
}

pub fn create_router(state: SchedulerApiState) -> Router {
    use self::endpoints::{
        JOB_LOG_DELETE, JOB_LOG_DETAIL, JOB_LOGS_CLEAN, JOB_LOGS_DELETE_BATCH, JOB_LOGS_EXPORT, JOB_LOGS_LIST, JOB_REPLACE, JOB_RUN, JOB_STATUS,
        JOBS_CRON_NEXT_TIMES, JOBS_DELETE_BATCH, JOBS_EXPORT, JOBS_IMPORT, JOBS_IMPORTABLE, JOBS_LIST,
    };

    Router::new()
        .route(JOBS_LIST.api_route_path(), get(handlers::jobs::list_jobs))
        .route(JOBS_EXPORT.api_route_path(), post(handlers::jobs::export_jobs))
        .route(JOBS_IMPORTABLE.api_route_path(), get(handlers::jobs::importable_tasks))
        .route(JOBS_IMPORT.api_route_path(), post(handlers::jobs::import_job))
        .route(JOBS_CRON_NEXT_TIMES.api_route_path(), post(handlers::jobs::cron_next_times))
        .route(JOBS_DELETE_BATCH.api_route_path(), delete(handlers::jobs::delete_jobs))
        .route(
            JOB_REPLACE.api_route_path(),
            get(handlers::jobs::get_job).put(handlers::jobs::replace_job).delete(handlers::jobs::delete_job),
        )
        .route(JOB_STATUS.api_route_path(), put(handlers::jobs::update_job_status))
        .route(JOB_RUN.api_route_path(), post(handlers::jobs::run_job))
        .route(JOB_LOGS_LIST.api_route_path(), get(handlers::logs::list_job_logs))
        .route(JOB_LOGS_EXPORT.api_route_path(), post(handlers::logs::export_job_logs))
        .route(JOB_LOGS_CLEAN.api_route_path(), delete(handlers::logs::clear_job_logs))
        .route(JOB_LOGS_DELETE_BATCH.api_route_path(), delete(handlers::logs::delete_job_logs))
        .route(JOB_LOG_DETAIL.api_route_path(), get(handlers::logs::get_job_log_detail))
        .route(
            JOB_LOG_DELETE.api_route_path(),
            get(handlers::logs::get_job_log).delete(handlers::logs::delete_job_log),
        )
        .with_state(state)
}
