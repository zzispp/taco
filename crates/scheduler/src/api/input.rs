use chrono::{DateTime, Utc};
use kernel::pagination::PageRequest;

use crate::{
    application::{ImportJobCommand, ReplaceJobCommand, SchedulerError, SchedulerResult, UpdateJobStatusCommand},
    domain::{ConcurrentPolicy, ExecutionOutcome, JobListFilter, JobLogListFilter, JobStatus, MisfirePolicy, TriggerType},
};

use super::dto::{ImportJobRequest, JobListQuery, JobLogListQuery, ReplaceJobRequest, UpdateJobStatusRequest};

pub fn job_filter(query: &JobListQuery) -> SchedulerResult<JobListFilter> {
    let begin_time = parse_time(query.begin_time.as_deref(), "begin_time")?;
    let end_time = parse_time(query.end_time.as_deref(), "end_time")?;
    validate_range(begin_time, end_time)?;
    Ok(JobListFilter {
        name: clean_optional(query.job_name.as_deref()),
        group: clean_optional(query.job_group.as_deref()),
        status: parse_optional_code(query.status.as_deref(), JobStatus::parse, "errors.scheduler.invalid_status")?,
        begin_time,
        end_time,
    })
}

pub fn log_filter(query: &JobLogListQuery) -> SchedulerResult<JobLogListFilter> {
    let begin_time = parse_time(query.begin_time.as_deref(), "begin_time")?;
    let end_time = parse_time(query.end_time.as_deref(), "end_time")?;
    validate_range(begin_time, end_time)?;
    Ok(JobLogListFilter {
        name: clean_optional(query.job_name.as_deref()),
        group: clean_optional(query.job_group.as_deref()),
        outcome: parse_optional_code(query.status.as_deref(), ExecutionOutcome::parse, "errors.scheduler.invalid_log_status")?,
        trigger: parse_optional_code(query.trigger_type.as_deref(), parse_trigger_type, "errors.scheduler.invalid_trigger_type")?,
        begin_time,
        end_time,
    })
}

pub fn page_request(page: u64, page_size: u64) -> PageRequest {
    PageRequest { page, page_size }
}

pub fn import_command(request: ImportJobRequest, operator: String) -> SchedulerResult<ImportJobCommand> {
    Ok(ImportJobCommand {
        task_key: request.task_key,
        name: request.job_name,
        group: request.job_group,
        cron_expression: request.cron_expression,
        misfire_policy: parse_code(&request.misfire_policy, MisfirePolicy::parse, "errors.scheduler.invalid_misfire_policy")?,
        concurrent: parse_code(&request.concurrent, ConcurrentPolicy::parse, "errors.scheduler.invalid_concurrent")?,
        task_params: request.task_params,
        remark: request.remark,
        operator,
    })
}

pub fn replace_command(id: String, request: ReplaceJobRequest, operator: String) -> SchedulerResult<ReplaceJobCommand> {
    Ok(ReplaceJobCommand {
        id,
        name: request.job_name,
        group: request.job_group,
        cron_expression: request.cron_expression,
        misfire_policy: parse_code(&request.misfire_policy, MisfirePolicy::parse, "errors.scheduler.invalid_misfire_policy")?,
        concurrent: parse_code(&request.concurrent, ConcurrentPolicy::parse, "errors.scheduler.invalid_concurrent")?,
        task_params: request.task_params,
        remark: request.remark,
        operator,
    })
}

pub fn status_command(id: String, request: UpdateJobStatusRequest, operator: String) -> SchedulerResult<UpdateJobStatusCommand> {
    Ok(UpdateJobStatusCommand {
        id,
        status: parse_code(&request.status, JobStatus::parse, "errors.scheduler.invalid_status")?,
        operator,
    })
}

fn parse_time(value: Option<&str>, field: &'static str) -> SchedulerResult<Option<DateTime<Utc>>> {
    let Some(value) = value.filter(|value| !value.trim().is_empty()) else {
        return Ok(None);
    };
    DateTime::parse_from_rfc3339(value.trim())
        .map(|value| Some(value.with_timezone(&Utc)))
        .map_err(|error| {
            hook_tracing::error_with_fields!("invalid scheduler date filter", &error, field = field);
            SchedulerError::InvalidInput(crate::application::localized_param("errors.scheduler.invalid_date_filter", "field", field))
        })
}

fn validate_range(begin: Option<DateTime<Utc>>, end: Option<DateTime<Utc>>) -> SchedulerResult<()> {
    if begin.zip(end).is_some_and(|(begin, end)| begin > end) {
        return Err(SchedulerError::InvalidInput(crate::application::localized(
            "errors.scheduler.invalid_date_range",
        )));
    }
    Ok(())
}

fn parse_code<T>(value: &str, parser: impl FnOnce(&str) -> Option<T>, key: &'static str) -> SchedulerResult<T> {
    parser(value).ok_or_else(|| SchedulerError::InvalidInput(crate::application::localized(key)))
}

fn parse_optional_code<T>(value: Option<&str>, parser: impl Fn(&str) -> Option<T>, key: &'static str) -> SchedulerResult<Option<T>> {
    value
        .filter(|value| !value.trim().is_empty())
        .map(|value| parse_code(value, parser, key))
        .transpose()
}

fn parse_trigger_type(value: &str) -> Option<TriggerType> {
    match value {
        "scheduled" => Some(TriggerType::Scheduled),
        "manual" => Some(TriggerType::Manual),
        "misfire" => Some(TriggerType::Misfire),
        _ => None,
    }
}

fn clean_optional(value: Option<&str>) -> Option<String> {
    value.map(str::trim).filter(|value| !value.is_empty()).map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use crate::{
        application::SchedulerError,
        domain::{ExecutionOutcome, TriggerType},
    };

    use super::{job_filter, log_filter};
    use crate::api::dto::{JobListQuery, JobLogListQuery};

    #[test]
    fn job_filter_parses_rfc3339_at_the_api_boundary() {
        let query = job_query(Some(" 2026-07-10T16:30:00+08:00 "), Some("2026-07-10T09:00:00Z"));

        let filter = job_filter(&query).unwrap();

        assert_eq!(filter.begin_time, Some(Utc.with_ymd_and_hms(2026, 7, 10, 8, 30, 0).unwrap()));
        assert_eq!(filter.end_time, Some(Utc.with_ymd_and_hms(2026, 7, 10, 9, 0, 0).unwrap()));
    }

    #[test]
    fn job_filter_rejects_invalid_dates_and_reversed_ranges() {
        let invalid = job_filter(&job_query(Some("2026-07-10"), None)).unwrap_err();
        let reversed = job_filter(&job_query(Some("2026-07-10T10:00:00Z"), Some("2026-07-10T09:00:00Z"))).unwrap_err();

        assert_invalid_key(invalid, "errors.scheduler.invalid_date_filter");
        assert_invalid_key(reversed, "errors.scheduler.invalid_date_range");
    }

    #[test]
    fn log_filter_uses_the_execution_outcome_contract() {
        let mut query = log_query();
        query.status = Some("3".into());
        assert_eq!(log_filter(&query).unwrap().outcome, Some(ExecutionOutcome::Interrupted));

        query.status = Some("unexpected".into());
        assert_invalid_key(log_filter(&query).unwrap_err(), "errors.scheduler.invalid_log_status");
    }

    #[test]
    fn log_filter_maps_public_trigger_values() {
        let cases = [
            ("scheduled", TriggerType::Scheduled),
            ("manual", TriggerType::Manual),
            ("misfire", TriggerType::Misfire),
        ];
        for (wire_value, expected) in cases {
            let mut query = log_query();
            query.trigger_type = Some(wire_value.into());
            assert_eq!(log_filter(&query).unwrap().trigger, Some(expected));
        }
    }

    #[test]
    fn log_filter_rejects_database_and_unknown_trigger_codes() {
        for value in ["S", "F", "M", "unexpected"] {
            let mut query = log_query();
            query.trigger_type = Some(value.into());
            assert_invalid_key(log_filter(&query).unwrap_err(), "errors.scheduler.invalid_trigger_type");
        }
    }

    fn job_query(begin_time: Option<&str>, end_time: Option<&str>) -> JobListQuery {
        JobListQuery {
            page: 1,
            page_size: 20,
            job_name: None,
            job_group: None,
            status: None,
            begin_time: begin_time.map(str::to_owned),
            end_time: end_time.map(str::to_owned),
        }
    }

    fn log_query() -> JobLogListQuery {
        JobLogListQuery {
            page: 1,
            page_size: 20,
            job_name: None,
            job_group: None,
            status: None,
            trigger_type: None,
            begin_time: None,
            end_time: None,
        }
    }

    fn assert_invalid_key(error: SchedulerError, expected: &'static str) {
        match error {
            SchedulerError::InvalidInput(details) => assert_eq!(details.key(), expected),
            other => panic!("expected invalid input, got {other:?}"),
        }
    }
}
