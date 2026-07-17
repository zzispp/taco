use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    application::{
        ExecutionLogSummary, JobView, SchedulerCursorQuery, SchedulerCursorSlice, SchedulerExportSession, SchedulerQueryExportSession, SchedulerResult,
        service_support::{lifecycle_capabilities, registry_status},
        task::TaskCatalog,
    },
    domain::{Job, JobListFilter, JobLogListFilter},
};

pub(super) struct ServiceExportSession {
    inner: Box<dyn SchedulerQueryExportSession>,
    catalog: Arc<dyn TaskCatalog>,
}

impl ServiceExportSession {
    pub(super) fn new(inner: Box<dyn SchedulerQueryExportSession>, catalog: Arc<dyn TaskCatalog>) -> Self {
        Self { inner, catalog }
    }

    fn decorate(&self, job: Job) -> JobView {
        let registry_status = registry_status(self.catalog.as_ref(), &job);
        let param_form = self.catalog.get(&job.task_key).map(|definition| (definition.params.form)());
        JobView {
            capabilities: lifecycle_capabilities(self.catalog.as_ref(), &job),
            job,
            registry_status,
            param_form,
        }
    }
}

#[async_trait]
impl SchedulerExportSession for ServiceExportSession {
    async fn page_jobs(&mut self, filter: JobListFilter, page: SchedulerCursorQuery) -> SchedulerResult<SchedulerCursorSlice<JobView>> {
        let page = self.inner.page_jobs(filter, page).await?;
        Ok(page.map(|job| self.decorate(job)))
    }

    async fn page_job_logs(&mut self, filter: JobLogListFilter, page: SchedulerCursorQuery) -> SchedulerResult<SchedulerCursorSlice<ExecutionLogSummary>> {
        self.inner.page_execution_logs(filter, page).await
    }

    async fn finish(self: Box<Self>) -> SchedulerResult<()> {
        let Self { inner, .. } = *self;
        inner.finish().await
    }
}
