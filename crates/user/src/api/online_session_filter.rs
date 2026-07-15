use kernel::error::LocalizedError;
use time::OffsetDateTime;
use types::http::{DATE_OR_RFC3339_FORMAT, DateTimeRangeError, parse_date_time_range};

use kernel::pagination::CursorPageRequest;
use rbac::domain::DataScopeFilter;

use crate::api::dto::OnlineSessionsQuery;
use crate::application::{AppError, AppResult, OnlineSessionFilter, OnlineSessionPageRequest, OnlineSessionSearch};

const NANOSECONDS_PER_MILLISECOND: i128 = 1_000_000;
const ONLINE_LOGIN_TIME_FILTER_ERROR_KEY: &str = "errors.user.invalid_online_login_time_filter";
const ONLINE_LOGIN_TIME_RANGE_ERROR_KEY: &str = "errors.user.invalid_online_login_time_range";
const ONLINE_LOGIN_TIME_OVERFLOW_ERROR: &str = "online login time filter timestamp overflow";

pub(super) fn online_session_page_request(query: OnlineSessionsQuery, scope: Option<DataScopeFilter>) -> AppResult<OnlineSessionPageRequest> {
    let filter = OnlineSessionFilter::from(query.clone());
    let page = CursorPageRequest {
        limit: query.limit,
        cursor: query.cursor,
    };
    validate_page(&page)?;
    let (begin_time, end_time) = login_time_millis_range(&filter)?;
    let request = OnlineSessionPageRequest {
        page,
        search: OnlineSessionSearch {
            ipaddr: filter.ipaddr,
            user_name: filter.user_name,
            login_location: filter.login_location,
            browser: filter.browser,
            os: filter.os,
            begin_time,
            end_time,
        },
        scope,
    };
    crate::application::cursor::OnlineCursorCodec::new(&request)?.decode(&request.page)?;
    Ok(request)
}

fn validate_page(page: &CursorPageRequest) -> AppResult<()> {
    crate::application::cursor::validate_cursor_request(page)
}

fn login_time_millis_range(filter: &OnlineSessionFilter) -> AppResult<(Option<i64>, Option<i64>)> {
    let range = parse_date_time_range(filter.begin_time.as_deref(), filter.end_time.as_deref()).map_err(login_time_error)?;
    Ok((to_millis(range.begin_time)?, to_millis(range.end_time)?))
}

fn to_millis(timestamp: Option<OffsetDateTime>) -> AppResult<Option<i64>> {
    timestamp
        .map(|value| i64::try_from(value.unix_timestamp_nanos().div_euclid(NANOSECONDS_PER_MILLISECOND)).map_err(|_| timestamp_overflow()))
        .transpose()
}

fn login_time_error(error: DateTimeRangeError) -> AppError {
    match error {
        DateTimeRangeError::InvalidBoundary(_) => invalid_date_filter(),
        DateTimeRangeError::Reversed => AppError::InvalidInput(LocalizedError::new(ONLINE_LOGIN_TIME_RANGE_ERROR_KEY)),
    }
}

fn invalid_date_filter() -> AppError {
    AppError::InvalidInput(LocalizedError::new(ONLINE_LOGIN_TIME_FILTER_ERROR_KEY).with_param("format", DATE_OR_RFC3339_FORMAT))
}

fn timestamp_overflow() -> AppError {
    AppError::Infrastructure(ONLINE_LOGIN_TIME_OVERFLOW_ERROR.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const JULY_8_2026_START_UTC_MILLIS: i64 = 1_783_468_800_000;
    const JULY_8_2026_END_UTC_MILLIS: i64 = 1_783_555_199_999;
    const JULY_8_2026_NOON_UTC_MILLIS: i64 = 1_783_512_000_000;

    #[test]
    fn legacy_date_boundaries_cover_the_complete_utc_day() {
        let filter = login_time_filter("2026-07-08", "2026-07-08");

        assert_eq!(
            login_time_millis_range(&filter).unwrap(),
            (Some(JULY_8_2026_START_UTC_MILLIS), Some(JULY_8_2026_END_UTC_MILLIS))
        );
    }

    #[test]
    fn rfc3339_boundaries_are_converted_to_utc_milliseconds() {
        let filter = login_time_filter("2026-07-08T20:00:00.000+08:00", "2026-07-08T12:00:00.000Z");

        assert_eq!(
            login_time_millis_range(&filter).unwrap(),
            (Some(JULY_8_2026_NOON_UTC_MILLIS), Some(JULY_8_2026_NOON_UTC_MILLIS))
        );
    }

    #[test]
    fn matcher_rejects_reversed_login_time_range() {
        let result = online_session_page_request(
            OnlineSessionsQuery {
                limit: 10,
                begin_time: Some("2026-07-08T12:00:00.001Z".into()),
                end_time: Some("2026-07-08T12:00:00.000Z".into()),
                ..Default::default()
            },
            None,
        );

        let Err(AppError::InvalidInput(error)) = result else {
            panic!("expected invalid login time range");
        };
        assert_eq!(error.key(), "errors.user.invalid_online_login_time_range");
    }

    fn login_time_filter(begin_time: &str, end_time: &str) -> OnlineSessionFilter {
        OnlineSessionFilter {
            begin_time: Some(begin_time.into()),
            end_time: Some(end_time.into()),
            ..Default::default()
        }
    }
}
