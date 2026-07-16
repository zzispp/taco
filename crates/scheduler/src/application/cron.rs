use chrono::{DateTime, Utc};
use croner::parser::{CronParser, Seconds, Year};

use super::{SchedulerError, SchedulerResult, error::localized_param};

pub const NEXT_TIMES_DEFAULT_COUNT: u8 = 5;
pub const NEXT_TIMES_MAX_COUNT: u8 = 20;

pub fn validate_cron(expression: &str) -> SchedulerResult<()> {
    parse_cron(expression).map(|_| ())
}

pub fn next_times_after(expression: &str, count: Option<u8>, now: DateTime<Utc>) -> SchedulerResult<Vec<DateTime<Utc>>> {
    let count = validated_count(count)?;
    let cron = parse_cron(expression)?;
    let mut cursor = now;
    let mut values = Vec::with_capacity(usize::from(count));
    for _ in 0..count {
        cursor = next_occurrence(&cron, &cursor)?;
        values.push(cursor);
    }
    Ok(values)
}

pub fn next_time_after(expression: &str, now: DateTime<Utc>) -> SchedulerResult<DateTime<Utc>> {
    let cron = parse_cron(expression)?;
    next_occurrence(&cron, &now)
}

fn validated_count(count: Option<u8>) -> SchedulerResult<u8> {
    let count = count.unwrap_or(NEXT_TIMES_DEFAULT_COUNT);
    if count == 0 || count > NEXT_TIMES_MAX_COUNT {
        return Err(SchedulerError::InvalidInput(localized_param(
            "errors.scheduler.invalid_preview_count",
            "max",
            NEXT_TIMES_MAX_COUNT.to_string(),
        )));
    }
    Ok(count)
}

fn parse_cron(expression: &str) -> SchedulerResult<croner::Cron> {
    CronParser::builder()
        .seconds(Seconds::Required)
        .year(Year::Optional)
        .alternative_weekdays(true)
        .build()
        .parse(expression)
        .map_err(|error| cron_error("parse", error))
}

fn next_occurrence(cron: &croner::Cron, cursor: &DateTime<Utc>) -> SchedulerResult<DateTime<Utc>> {
    cron.find_next_occurrence(cursor, false).map_err(|error| cron_error("next_occurrence", error))
}

fn cron_error(operation: &'static str, error: croner::errors::CronError) -> SchedulerError {
    taco_tracing::error_with_fields!("scheduler cron operation failed", &error, operation = operation);
    SchedulerError::InvalidInput(super::error::localized("errors.scheduler.invalid_cron"))
}
