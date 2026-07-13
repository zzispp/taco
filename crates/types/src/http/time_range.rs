use time::{Date, Duration, Month, OffsetDateTime, Time, UtcOffset, format_description::well_known::Rfc3339};

pub const DATE_OR_RFC3339_FORMAT: &str = "YYYY-MM-DD / RFC3339";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateTimeRange {
    pub begin_time: Option<OffsetDateTime>,
    pub end_time: Option<OffsetDateTime>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateTimeRangeError {
    InvalidBoundary(DateTimeRangeField),
    Reversed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateTimeRangeField {
    BeginTime,
    EndTime,
}

impl DateTimeRangeField {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BeginTime => "begin_time",
            Self::EndTime => "end_time",
        }
    }
}

pub fn parse_date_time_range(begin_time: Option<&str>, end_time: Option<&str>) -> Result<DateTimeRange, DateTimeRangeError> {
    let begin_time = parse_boundary(begin_time, TimeBoundary::Start, DateTimeRangeField::BeginTime)?;
    let end_time = parse_boundary(end_time, TimeBoundary::End, DateTimeRangeField::EndTime)?;
    if begin_time.zip(end_time).is_some_and(|(begin, end)| begin > end) {
        return Err(DateTimeRangeError::Reversed);
    }
    Ok(DateTimeRange { begin_time, end_time })
}

fn parse_boundary(value: Option<&str>, boundary: TimeBoundary, field: DateTimeRangeField) -> Result<Option<OffsetDateTime>, DateTimeRangeError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if let Some(date) = parse_date(value) {
        return date_boundary(date, boundary).map(Some).ok_or(DateTimeRangeError::InvalidBoundary(field));
    }
    OffsetDateTime::parse(value, &Rfc3339)
        .map(|value| Some(value.to_offset(UtcOffset::UTC)))
        .map_err(|_| DateTimeRangeError::InvalidBoundary(field))
}

fn parse_date(value: &str) -> Option<Date> {
    if !has_iso_date_shape(value) {
        return None;
    }
    let mut parts = value.split('-');
    let year = parts.next()?.parse::<i32>().ok()?;
    let month = parts.next()?.parse::<u8>().ok().and_then(|month| Month::try_from(month).ok())?;
    let day = parts.next()?.parse::<u8>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Date::from_calendar_date(year, month, day).ok()
}

fn has_iso_date_shape(value: &str) -> bool {
    let [year_1, year_2, year_3, year_4, b'-', month_1, month_2, b'-', day_1, day_2] = value.as_bytes() else {
        return false;
    };
    [year_1, year_2, year_3, year_4, month_1, month_2, day_1, day_2]
        .into_iter()
        .all(|digit| digit.is_ascii_digit())
}

fn date_boundary(date: Date, boundary: TimeBoundary) -> Option<OffsetDateTime> {
    let start = date.with_time(Time::MIDNIGHT).assume_utc();
    match boundary {
        TimeBoundary::Start => Some(start),
        TimeBoundary::End => date.next_day().map(|next| next.with_time(Time::MIDNIGHT).assume_utc() - Duration::NANOSECOND),
    }
}

#[derive(Clone, Copy)]
enum TimeBoundary {
    Start,
    End,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_date_range_covers_the_complete_utc_day() {
        let range = parse_date_time_range(Some("2026-07-08"), Some("2026-07-08")).unwrap();

        assert_eq!(range.begin_time, Some(timestamp("2026-07-08T00:00:00Z")));
        assert_eq!(range.end_time, Some(timestamp("2026-07-08T23:59:59.999999999Z")));
    }

    #[test]
    fn rfc3339_range_is_normalized_to_utc_without_losing_precision() {
        let range = parse_date_time_range(Some("2026-07-08T20:00:00.001+08:00"), Some("2026-07-08T20:00:00.002+08:00")).unwrap();

        assert_eq!(range.begin_time, Some(timestamp("2026-07-08T12:00:00.001Z")));
        assert_eq!(range.end_time, Some(timestamp("2026-07-08T12:00:00.002Z")));
    }

    #[test]
    fn blank_boundaries_are_absent() {
        assert_eq!(
            parse_date_time_range(Some("  "), None).unwrap(),
            DateTimeRange {
                begin_time: None,
                end_time: None,
            }
        );
    }

    #[test]
    fn invalid_boundary_identifies_the_query_field() {
        assert_eq!(
            parse_date_time_range(None, Some("2026-99-99")),
            Err(DateTimeRangeError::InvalidBoundary(DateTimeRangeField::EndTime))
        );
        assert_eq!(DateTimeRangeField::EndTime.as_str(), "end_time");
    }

    #[test]
    fn legacy_date_requires_zero_padded_iso_shape() {
        assert_eq!(
            parse_date_time_range(Some("2026-7-08"), None),
            Err(DateTimeRangeError::InvalidBoundary(DateTimeRangeField::BeginTime))
        );
    }

    #[test]
    fn reversed_range_is_rejected_at_instant_precision() {
        assert_eq!(
            parse_date_time_range(Some("2026-07-08T12:00:00.001Z"), Some("2026-07-08T12:00:00Z")),
            Err(DateTimeRangeError::Reversed)
        );
    }

    fn timestamp(value: &str) -> OffsetDateTime {
        OffsetDateTime::parse(value, &Rfc3339).unwrap()
    }
}
