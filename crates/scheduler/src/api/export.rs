use kernel::{
    excel::{StreamingXlsxWriter, TemporaryXlsxFile},
    pagination::CursorDirection,
    runtime_config::ExportBatchConfig,
};
use types::http::{Locale, translate_message};

use crate::{
    application::{
        ExecutionLogSummary, JobView, SchedulerCursorPoint, SchedulerCursorQuery, SchedulerError, SchedulerResult, job_point, log_point,
        tasks::sanitize_http_invoke_target,
    },
    domain::{JobListFilter, JobLogListFilter},
};

use super::{
    SchedulerApiState,
    presentation::{concurrent_policy, execution_outcome, job_status, registry_status, runtime_error, trigger_type},
    presenter::{execution_response, rfc3339},
};

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

pub async fn export_jobs(request: ExportRequest<'_, JobListFilter>) -> SchedulerResult<TemporaryXlsxFile> {
    let ExportRequest { state, filter, batch, locale } = request;
    let headers = translated_headers(locale, JOB_HEADER_KEYS);
    let mut writer = StreamingXlsxWriter::new(&translate_message(locale, "excel.scheduler.job.sheet"), &headers).map_err(excel_error)?;
    let mut export = state.scheduler.begin_export().await?;
    let mut cursor = ExportCursor::new(batch.page_size)?;
    loop {
        let result = export.page_jobs(filter.clone(), cursor.request()).await?;
        if result.items.is_empty() {
            break;
        }
        let boundary = job_point(
            &result
                .items
                .last()
                .ok_or_else(|| SchedulerError::Infrastructure("job export batch lost its last row".into()))?
                .job,
        );
        let has_more = cursor.advance(boundary, result.snapshot.clone(), result.has_next)?;
        let rows = result.items.into_iter().map(|job| job_row(job, locale)).collect::<SchedulerResult<Vec<_>>>()?;
        writer.append_rows(&rows).map_err(excel_error)?;
        if !has_more {
            break;
        }
    }
    export.finish().await?;
    writer.finish().map_err(excel_error)
}

pub async fn export_job_logs(request: ExportRequest<'_, JobLogListFilter>) -> SchedulerResult<TemporaryXlsxFile> {
    let ExportRequest { state, filter, batch, locale } = request;
    let headers = translated_headers(locale, LOG_HEADER_KEYS);
    let mut writer = StreamingXlsxWriter::new(&translate_message(locale, "excel.scheduler.job_log.sheet"), &headers).map_err(excel_error)?;
    let mut export = state.scheduler.begin_export().await?;
    let mut cursor = ExportCursor::new(batch.page_size)?;
    loop {
        let result = export.page_job_logs(filter.clone(), cursor.request()).await?;
        if result.items.is_empty() {
            break;
        }
        let boundary = log_point(
            result
                .items
                .last()
                .ok_or_else(|| SchedulerError::Infrastructure("job log export batch lost its last row".into()))?,
        );
        let has_more = cursor.advance(boundary, result.snapshot.clone(), result.has_next)?;
        let rows = result
            .items
            .into_iter()
            .map(|execution| log_row(execution, locale))
            .collect::<SchedulerResult<Vec<_>>>()?;
        writer.append_rows(&rows).map_err(excel_error)?;
        if !has_more {
            break;
        }
    }
    export.finish().await?;
    writer.finish().map_err(excel_error)
}

#[derive(Debug)]
struct ExportCursor {
    limit: u64,
    boundary: Option<SchedulerCursorPoint>,
    snapshot: Option<SchedulerCursorPoint>,
}

impl ExportCursor {
    fn new(limit: u64) -> SchedulerResult<Self> {
        if limit == 0 {
            return Err(SchedulerError::Infrastructure("export page size is zero after validation".into()));
        }
        Ok(Self {
            limit,
            boundary: None,
            snapshot: None,
        })
    }

    fn request(&self) -> SchedulerCursorQuery {
        SchedulerCursorQuery {
            limit: self.limit,
            direction: CursorDirection::Next,
            boundary: self.boundary.clone(),
            snapshot: self.snapshot.clone(),
        }
    }

    fn advance(&mut self, boundary: SchedulerCursorPoint, snapshot: Option<SchedulerCursorPoint>, has_more: bool) -> SchedulerResult<bool> {
        self.snapshot = Some(snapshot.ok_or_else(|| SchedulerError::Infrastructure("scheduler export batch is missing its snapshot".into()))?);
        self.boundary = Some(boundary);
        Ok(has_more)
    }
}

fn job_row(view: JobView, locale: Locale) -> SchedulerResult<Vec<String>> {
    let job = view.job;
    let runtime_error = job
        .runtime_error
        .as_ref()
        .map(|error| runtime_error(error.code).localized(locale))
        .unwrap_or_default();
    let invoke_target = sanitize_http_invoke_target(&job.task_key, &job.invoke_target);
    Ok(vec![
        job.name,
        job.group,
        job.task_key,
        invoke_target,
        job.cron_expression,
        job_status(job.status).localized(locale),
        concurrent_policy(job.concurrent).localized(locale),
        registry_status(view.registry_status).localized(locale),
        job.next_run_at.map(rfc3339).transpose()?.unwrap_or_default(),
        runtime_error,
    ])
}

fn log_row(execution: ExecutionLogSummary, locale: Locale) -> SchedulerResult<Vec<String>> {
    let trigger = trigger_type(execution.trigger).localized(locale);
    let outcome = execution_outcome(execution.outcome).localized(locale);
    let log = execution_response(execution, locale)?;
    Ok(vec![
        log.job_name,
        log.job_group,
        log.task_key,
        trigger,
        outcome,
        log.job_message,
        log.scheduled_at,
        log.start_time.unwrap_or_default(),
        log.end_time,
    ])
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
