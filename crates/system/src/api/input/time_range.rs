use kernel::error::LocalizedError;
use types::http::{DATE_OR_RFC3339_FORMAT, DateTimeRange, DateTimeRangeError, parse_date_time_range};

use crate::application::{SystemError, SystemResult};

pub(super) fn created_time_range(begin_time: Option<&str>, end_time: Option<&str>) -> SystemResult<DateTimeRange> {
    parse_date_time_range(begin_time, end_time).map_err(map_range_error)
}

fn map_range_error(error: DateTimeRangeError) -> SystemError {
    let message = match error {
        DateTimeRangeError::InvalidBoundary(field) => LocalizedError::new("errors.system.invalid_created_time_filter")
            .with_param("field", field.as_str())
            .with_param("format", DATE_OR_RFC3339_FORMAT),
        DateTimeRangeError::Reversed => LocalizedError::new("errors.system.invalid_created_time_range"),
    };
    SystemError::InvalidInput(message)
}

#[cfg(test)]
mod tests {
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};

    use super::*;

    #[test]
    fn created_time_range_preserves_rfc3339_precision_and_offset() {
        let range = created_time_range(Some("2026-07-08T20:00:00.001+08:00"), Some("2026-07-08T20:00:00.002+08:00")).unwrap();

        assert_eq!(range.begin_time, Some(timestamp("2026-07-08T12:00:00.001Z")));
        assert_eq!(range.end_time, Some(timestamp("2026-07-08T12:00:00.002Z")));
    }

    #[test]
    fn invalid_created_time_identifies_the_field_and_format() {
        let error = created_time_range(Some("invalid"), None).unwrap_err();
        let SystemError::InvalidInput(message) = error else {
            panic!("expected invalid input");
        };

        assert_eq!(message.key(), "errors.system.invalid_created_time_filter");
        assert_eq!(param(&message, "field"), Some("begin_time"));
        assert_eq!(param(&message, "format"), Some(DATE_OR_RFC3339_FORMAT));
    }

    #[test]
    fn reversed_created_time_range_has_a_stable_error_key() {
        let error = created_time_range(Some("2026-07-08T12:00:00.001Z"), Some("2026-07-08T12:00:00Z")).unwrap_err();
        let SystemError::InvalidInput(message) = error else {
            panic!("expected invalid input");
        };

        assert_eq!(message.key(), "errors.system.invalid_created_time_range");
    }

    fn timestamp(value: &str) -> OffsetDateTime {
        OffsetDateTime::parse(value, &Rfc3339).unwrap()
    }

    fn param<'a>(message: &'a LocalizedError, key: &str) -> Option<&'a str> {
        message.params().iter().find(|param| param.key() == key).map(|param| param.value())
    }
}
