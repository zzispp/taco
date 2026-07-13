use kernel::{excel::StreamingXlsxWriter, pagination::PageSliceRequest, runtime_config::ExportBatchConfig};
use types::http::{Locale, translate_message};

use crate::{
    application::{ExecutionLogSummary, JobView, SchedulerError, SchedulerResult},
    domain::{JobListFilter, JobLogListFilter},
};

use super::{
    SchedulerApiState,
    presentation::{concurrent_policy, execution_outcome, job_status, registry_status, runtime_error, trigger_type},
    presenter::{execution_response, rfc3339},
};

const PAGE_INCREMENT: u64 = 1;

const JOB_HEADER_KEYS: &[&str] = &[
    "excel.scheduler.job.headers.name",
    "excel.scheduler.job.headers.group",
    "excel.scheduler.job.headers.task_key",
    "excel.scheduler.job.headers.invoke_target",
    "excel.scheduler.job.headers.cron_expression",
    "excel.scheduler.job.headers.status",
    "excel.scheduler.job.headers.concurrent",
    "excel.scheduler.job.headers.registry_status",
    "excel.scheduler.job.headers.next_run_at",
    "excel.scheduler.job.headers.runtime_error",
];
const LOG_HEADER_KEYS: &[&str] = &[
    "excel.scheduler.job_log.headers.name",
    "excel.scheduler.job_log.headers.group",
    "excel.scheduler.job_log.headers.task_key",
    "excel.scheduler.job_log.headers.trigger",
    "excel.scheduler.job_log.headers.status",
    "excel.scheduler.job_log.headers.message",
    "excel.scheduler.job_log.headers.scheduled_at",
    "excel.scheduler.job_log.headers.start_time",
    "excel.scheduler.job_log.headers.end_time",
];

pub struct ExportRequest<'a, F> {
    pub state: &'a SchedulerApiState,
    pub filter: F,
    pub batch: ExportBatchConfig,
    pub locale: Locale,
}

pub async fn export_jobs(request: ExportRequest<'_, JobListFilter>) -> SchedulerResult<Vec<u8>> {
    let ExportRequest { state, filter, batch, locale } = request;
    let headers = translated_headers(locale, JOB_HEADER_KEYS);
    let mut writer = StreamingXlsxWriter::new(&translate_message(locale, "excel.scheduler.job.sheet"), &headers).map_err(excel_error)?;
    let mut page = ExportPage::new(batch.page_size)?;
    loop {
        let result = state.scheduler.export_jobs_page(filter.clone(), page.request()?).await?;
        page.observe_total(result.total)?;
        if result.items.is_empty() {
            page.ensure_complete()?;
            break;
        }
        let rows = result.items.into_iter().map(|job| job_row(job, locale)).collect::<Vec<_>>();
        writer.append_rows(&rows).map_err(excel_error)?;
        if !page.advance(rows.len())? {
            break;
        }
    }
    writer.finish().map_err(excel_error)
}

pub async fn export_job_logs(request: ExportRequest<'_, JobLogListFilter>) -> SchedulerResult<Vec<u8>> {
    let ExportRequest { state, filter, batch, locale } = request;
    let headers = translated_headers(locale, LOG_HEADER_KEYS);
    let mut writer = StreamingXlsxWriter::new(&translate_message(locale, "excel.scheduler.job_log.sheet"), &headers).map_err(excel_error)?;
    let mut page = ExportPage::new(batch.page_size)?;
    loop {
        let result = state.scheduler.export_job_logs_page(filter.clone(), page.request()?).await?;
        page.observe_total(result.total)?;
        if result.items.is_empty() {
            page.ensure_complete()?;
            break;
        }
        let rows = result.items.into_iter().map(|execution| log_row(execution, locale)).collect::<Vec<_>>();
        writer.append_rows(&rows).map_err(excel_error)?;
        if !page.advance(rows.len())? {
            break;
        }
    }
    writer.finish().map_err(excel_error)
}

#[derive(Debug)]
struct ExportPage {
    page: u64,
    page_size: u64,
    offset: u64,
    total: Option<u64>,
}

impl ExportPage {
    fn new(page_size: u64) -> SchedulerResult<Self> {
        if page_size == 0 {
            return Err(SchedulerError::Infrastructure("export page size is zero after validation".into()));
        }
        Ok(Self {
            page: constants::pagination::MIN_PAGE_NUMBER,
            page_size,
            offset: 0,
            total: None,
        })
    }

    fn request(&self) -> SchedulerResult<PageSliceRequest> {
        Ok(PageSliceRequest {
            offset: self.offset,
            limit: self.page_size,
            page: self.page,
            page_size: self.page_size,
        })
    }

    fn observe_total(&mut self, total: u64) -> SchedulerResult<()> {
        match self.total {
            None => self.total = Some(total),
            Some(expected) if expected == total => {}
            Some(_) => return Err(SchedulerError::Infrastructure("export total changed while paging".into())),
        }
        Ok(())
    }

    fn advance(&mut self, returned: usize) -> SchedulerResult<bool> {
        let total = self.required_total()?;
        let returned = u64::try_from(returned).map_err(|error| SchedulerError::Infrastructure(format!("export row count overflow: {error}")))?;
        if returned == 0 {
            self.ensure_complete()?;
            return Ok(false);
        }
        let next_offset = self
            .offset
            .checked_add(returned)
            .ok_or_else(|| SchedulerError::Infrastructure("export offset overflow".into()))?;
        if next_offset > total {
            return Err(SchedulerError::Infrastructure("export page returned more rows than the reported total".into()));
        }
        self.offset = next_offset;
        if self.offset == total {
            return Ok(false);
        }
        self.page = self
            .page
            .checked_add(PAGE_INCREMENT)
            .ok_or_else(|| SchedulerError::Infrastructure("export page overflow".into()))?;
        Ok(true)
    }

    fn ensure_complete(&self) -> SchedulerResult<()> {
        let total = self.required_total()?;
        if self.offset == total {
            return Ok(());
        }
        Err(SchedulerError::Infrastructure(
            "export page was empty before all reported rows were read".into(),
        ))
    }

    fn required_total(&self) -> SchedulerResult<u64> {
        self.total.ok_or_else(|| SchedulerError::Infrastructure("export total was not observed".into()))
    }
}

fn job_row(view: JobView, locale: Locale) -> Vec<String> {
    let runtime_error = view
        .job
        .runtime_error
        .as_ref()
        .map(|error| runtime_error(error.code).localized(locale))
        .unwrap_or_default();
    vec![
        view.job.name,
        view.job.group,
        view.job.task_key,
        view.job.invoke_target,
        view.job.cron_expression,
        job_status(view.job.status).localized(locale),
        concurrent_policy(view.job.concurrent).localized(locale),
        registry_status(view.registry_status).localized(locale),
        view.job.next_run_at.map(rfc3339).unwrap_or_default(),
        runtime_error,
    ]
}

fn log_row(execution: ExecutionLogSummary, locale: Locale) -> Vec<String> {
    let trigger = trigger_type(execution.trigger).localized(locale);
    let outcome = execution_outcome(execution.outcome).localized(locale);
    let log = execution_response(execution, locale);
    vec![
        log.job_name,
        log.job_group,
        log.task_key,
        trigger,
        outcome,
        log.job_message,
        log.scheduled_at,
        log.start_time.unwrap_or_default(),
        log.end_time,
    ]
}

fn translated_headers(locale: Locale, keys: &[&str]) -> Vec<String> {
    keys.iter().map(|key| translate_message(locale, key)).collect()
}

fn excel_error(error: String) -> SchedulerError {
    SchedulerError::Infrastructure(error)
}

#[cfg(test)]
#[path = "export_tests.rs"]
mod tests;
