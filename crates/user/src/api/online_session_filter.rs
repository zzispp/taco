use kernel::error::LocalizedError;
use time::{Date, Month, Time};

use crate::application::{AppError, AppResult, OnlineSession, OnlineSessionFilter};

const DATE_FILTER_FORMAT: &str = "YYYY-MM-DD";
const MILLISECONDS_PER_SECOND: i64 = 1_000;
const MILLISECOND_BEFORE_NEXT_DAY: i64 = 1;
const ONLINE_LOGIN_TIME_FILTER_ERROR_KEY: &str = "errors.user.invalid_online_login_time_filter";
const ONLINE_LOGIN_TIME_OVERFLOW_ERROR: &str = "online login time filter timestamp overflow";

pub(super) struct OnlineSessionMatcher {
    filter: OnlineSessionFilter,
    begin_millis: Option<i64>,
    end_millis: Option<i64>,
}

impl OnlineSessionMatcher {
    pub(super) fn new(filter: OnlineSessionFilter) -> AppResult<Self> {
        Ok(Self {
            begin_millis: parse_boundary_millis(&filter.begin_time, TimeBoundary::Start)?,
            end_millis: parse_boundary_millis(&filter.end_time, TimeBoundary::End)?,
            filter,
        })
    }

    pub(super) fn matches(&self, session: &OnlineSession) -> bool {
        case_insensitive_contains_filter(&session.ipaddr, &self.filter.ipaddr)
            && case_insensitive_contains_filter(&session.user_name, &self.filter.user_name)
            && case_insensitive_contains_filter(&session.login_location, &self.filter.login_location)
            && case_insensitive_contains_filter(&session.browser, &self.filter.browser)
            && case_insensitive_contains_filter(&session.os, &self.filter.os)
            && self.begin_millis.is_none_or(|start| session.login_time >= start)
            && self.end_millis.is_none_or(|end| session.login_time <= end)
    }
}

fn case_insensitive_contains_filter(value: &str, filter: &Option<String>) -> bool {
    filter.as_ref().is_none_or(|needle| value.to_lowercase().contains(&needle.to_lowercase()))
}

fn parse_boundary_millis(value: &Option<String>, boundary: TimeBoundary) -> AppResult<Option<i64>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let date = parse_date(value)?;
    boundary_millis(date, boundary).map(Some)
}

fn parse_date(value: &str) -> AppResult<Date> {
    let mut parts = value.split('-');
    let Some(year) = parts.next() else {
        return Err(invalid_date_filter());
    };
    let Some(month) = parts.next() else {
        return Err(invalid_date_filter());
    };
    let Some(day) = parts.next() else {
        return Err(invalid_date_filter());
    };
    if parts.next().is_some() {
        return Err(invalid_date_filter());
    }
    calendar_date(year, month, day)
}

fn calendar_date(year: &str, month: &str, day: &str) -> AppResult<Date> {
    let year = year.parse::<i32>().map_err(|_| invalid_date_filter())?;
    let month = month
        .parse::<u8>()
        .ok()
        .and_then(|value| Month::try_from(value).ok())
        .ok_or_else(invalid_date_filter)?;
    let day = day.parse::<u8>().map_err(|_| invalid_date_filter())?;
    Date::from_calendar_date(year, month, day).map_err(|_| invalid_date_filter())
}

fn boundary_millis(date: Date, boundary: TimeBoundary) -> AppResult<i64> {
    match boundary {
        TimeBoundary::Start => start_of_day_millis(date),
        TimeBoundary::End => end_of_day_millis(date),
    }
}

fn start_of_day_millis(date: Date) -> AppResult<i64> {
    date.with_time(Time::MIDNIGHT)
        .assume_utc()
        .unix_timestamp()
        .checked_mul(MILLISECONDS_PER_SECOND)
        .ok_or_else(timestamp_overflow)
}

fn end_of_day_millis(date: Date) -> AppResult<i64> {
    let next_day = date.next_day().ok_or_else(timestamp_overflow)?;
    start_of_day_millis(next_day)?
        .checked_sub(MILLISECOND_BEFORE_NEXT_DAY)
        .ok_or_else(timestamp_overflow)
}

fn invalid_date_filter() -> AppError {
    AppError::InvalidInput(LocalizedError::new(ONLINE_LOGIN_TIME_FILTER_ERROR_KEY).with_param("format", DATE_FILTER_FORMAT))
}

fn timestamp_overflow() -> AppError {
    AppError::Infrastructure(ONLINE_LOGIN_TIME_OVERFLOW_ERROR.into())
}

#[derive(Clone, Copy)]
enum TimeBoundary {
    Start,
    End,
}
