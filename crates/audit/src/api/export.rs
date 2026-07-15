use kernel::{
    excel::{StreamingXlsxWriter, TemporaryXlsxFile},
    pagination::CursorDirection,
    runtime_config::ExportBatchConfig,
};
use time::format_description::well_known::Rfc3339;
use types::http::{Locale, translate_message, translate_message_with_params};

use crate::{
    application::{AuditCursorQuery, AuditError, AuditResult, AuditSnapshot, LoginCursorBoundary, OperationCursorBoundary, login_boundary, operation_boundary},
    domain::{LoginEventType, LoginLog, LoginLogFilter, OperationLogFilter, OperationLogSummary},
};

use super::{AuditApiState, presenter::render_location};

const OPERATION_HEADERS: &[&str] = &[
    "excel.audit.operation.headers.id",
    "excel.audit.operation.headers.title",
    "excel.audit.operation.headers.business_type",
    "excel.audit.operation.headers.method",
    "excel.audit.operation.headers.request_method",
    "excel.audit.operation.headers.operator_type",
    "excel.audit.operation.headers.operator_name",
    "excel.audit.operation.headers.department_name",
    "excel.audit.operation.headers.url",
    "excel.audit.operation.headers.ip",
    "excel.audit.operation.headers.location",
    "excel.audit.operation.headers.status",
    "excel.audit.operation.headers.time",
    "excel.audit.operation.headers.cost_time",
];
const LOGIN_HEADERS: &[&str] = &[
    "excel.audit.login.headers.id",
    "excel.audit.login.headers.username",
    "excel.audit.login.headers.ip",
    "excel.audit.login.headers.location",
    "excel.audit.login.headers.browser",
    "excel.audit.login.headers.os",
    "excel.audit.login.headers.status",
    "excel.audit.login.headers.event_type",
    "excel.audit.login.headers.message",
    "excel.audit.login.headers.time",
];

pub struct ExportRequest<'a, F> {
    pub state: &'a AuditApiState,
    pub filter: F,
    pub batch: ExportBatchConfig,
    pub locale: Locale,
}

pub async fn operation_logs(request: ExportRequest<'_, OperationLogFilter>) -> AuditResult<TemporaryXlsxFile> {
    let ExportRequest { state, filter, batch, locale } = request;
    let headers = headers(locale, OPERATION_HEADERS);
    let mut writer = StreamingXlsxWriter::new(&translate_message(locale, "excel.audit.operation.sheet"), &headers).map_err(excel_error)?;
    let mut export = state.audit.begin_export().await?;
    let mut cursor = ExportCursor::<OperationCursorBoundary>::new(batch.page_size)?;
    loop {
        let result = export.page_operations(filter.clone(), cursor.request()?).await?;
        if result.items.is_empty() {
            break;
        }
        let boundary = operation_boundary(
            result
                .items
                .last()
                .ok_or_else(|| AuditError::Infrastructure("operation export page lost its last row".into()))?,
            filter.sort_field,
        );
        let has_more = cursor.advance(boundary, result.snapshot.clone(), result.has_next)?;
        let rows = result
            .items
            .into_iter()
            .map(|item| operation_row(item, locale))
            .collect::<AuditResult<Vec<_>>>()?;
        writer.append_rows(&rows).map_err(excel_error)?;
        if !has_more {
            break;
        }
    }
    export.finish().await?;
    writer.finish().map_err(excel_error)
}

pub async fn login_logs(request: ExportRequest<'_, LoginLogFilter>) -> AuditResult<TemporaryXlsxFile> {
    let ExportRequest { state, filter, batch, locale } = request;
    let headers = headers(locale, LOGIN_HEADERS);
    let mut writer = StreamingXlsxWriter::new(&translate_message(locale, "excel.audit.login.sheet"), &headers).map_err(excel_error)?;
    let mut export = state.audit.begin_export().await?;
    let mut cursor = ExportCursor::<LoginCursorBoundary>::new(batch.page_size)?;
    loop {
        let result = export.page_logins(filter.clone(), cursor.request()?).await?;
        if result.items.is_empty() {
            break;
        }
        let boundary = login_boundary(
            result
                .items
                .last()
                .ok_or_else(|| AuditError::Infrastructure("login export page lost its last row".into()))?,
            filter.sort_field,
        );
        let has_more = cursor.advance(boundary, result.snapshot.clone(), result.has_next)?;
        let rows = result.items.into_iter().map(|item| login_row(item, locale)).collect::<AuditResult<Vec<_>>>()?;
        writer.append_rows(&rows).map_err(excel_error)?;
        if !has_more {
            break;
        }
    }
    export.finish().await?;
    writer.finish().map_err(excel_error)
}

fn operation_row(summary: OperationLogSummary, locale: Locale) -> AuditResult<Vec<String>> {
    Ok(vec![
        summary.id,
        translate_message(locale, &summary.title_key),
        translate_message(locale, summary.business_type.message_key()),
        summary.handler,
        summary.request_method,
        translate_message(locale, summary.operator_type.message_key()),
        summary.operator_name,
        summary.department_name,
        summary.operation_url,
        summary.operation_ip,
        render_location(&summary.operation_location, locale),
        translate_message(locale, summary.status.message_key()),
        timestamp(summary.operation_time)?,
        summary.cost_time_ms.to_string(),
    ])
}

fn login_row(value: LoginLog, locale: Locale) -> AuditResult<Vec<String>> {
    let params = value
        .message_params
        .iter()
        .map(|(key, value)| (key.as_str(), value.clone()))
        .collect::<Vec<_>>();
    Ok(vec![
        value.id,
        value.username,
        value.ip_address,
        render_location(&value.login_location, locale),
        value.browser,
        value.os,
        translate_message(locale, value.status.message_key()),
        translate_message(locale, event_key(value.event_type)),
        translate_message_with_params(locale, &value.message_key, &params),
        timestamp(value.login_time)?,
    ])
}

fn event_key(event_type: LoginEventType) -> &'static str {
    match event_type {
        LoginEventType::LoginSuccess => "audit.event_type.login_success",
        LoginEventType::LoginFailure => "audit.event_type.login_failure",
        LoginEventType::RegisterSuccess => "audit.event_type.register_success",
        LoginEventType::RegisterFailure => "audit.event_type.register_failure",
        LoginEventType::LogoutSuccess => "audit.event_type.logout_success",
        LoginEventType::LogoutFailure => "audit.event_type.logout_failure",
        LoginEventType::RefreshSuccess => "audit.event_type.refresh_success",
        LoginEventType::RefreshFailure => "audit.event_type.refresh_failure",
    }
}

fn headers(locale: Locale, keys: &[&str]) -> Vec<String> {
    keys.iter().map(|key| translate_message(locale, key)).collect()
}

fn timestamp(value: time::OffsetDateTime) -> AuditResult<String> {
    value.format(&Rfc3339).map_err(|error| AuditError::Infrastructure(error.to_string()))
}

fn excel_error(error: impl std::fmt::Display) -> AuditError {
    AuditError::Infrastructure(format!("audit spreadsheet generation failed: {error}"))
}

#[derive(Debug)]
struct ExportCursor<B> {
    limit: u64,
    boundary: Option<B>,
    snapshot: Option<AuditSnapshot>,
}

impl<B: Clone> ExportCursor<B> {
    fn new(limit: u64) -> AuditResult<Self> {
        if limit == 0 {
            return Err(AuditError::Infrastructure("audit export page size must be positive".into()));
        }
        Ok(Self {
            limit,
            boundary: None,
            snapshot: None,
        })
    }

    fn request(&self) -> AuditResult<AuditCursorQuery<B>> {
        Ok(AuditCursorQuery {
            limit: self.limit,
            direction: CursorDirection::Next,
            boundary: self.boundary.clone(),
            snapshot: self.snapshot.clone(),
        })
    }

    fn advance(&mut self, boundary: B, snapshot: Option<AuditSnapshot>, has_more: bool) -> AuditResult<bool> {
        self.snapshot = Some(snapshot.ok_or_else(|| AuditError::Infrastructure("audit export page is missing its snapshot".into()))?);
        self.boundary = Some(boundary);
        Ok(has_more)
    }
}

#[cfg(test)]
mod tests;
