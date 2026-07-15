use kernel::{error::LocalizedError, pagination::CursorPageRequest};
use types::http::{DATE_OR_RFC3339_FORMAT, DateTimeRangeError, parse_date_time_range};

use crate::{
    api::dto::{ListUsersQuery, UserExportQuery},
    application::{AppError, AppResult, UserListFilter},
};

const USER_CREATE_TIME_FILTER_ERROR_KEY: &str = "errors.user.invalid_created_time_filter";
const USER_CREATE_TIME_RANGE_ERROR_KEY: &str = "errors.user.invalid_created_time_range";
pub(super) fn list_user_filter(query: ListUsersQuery) -> AppResult<UserListFilter> {
    let mut fields = UserFilterFields::from(query);
    let page = CursorPageRequest {
        limit: fields.limit,
        cursor: fields.cursor.take(),
    };
    build_filter(fields, page)
}

pub(super) fn export_user_filter(query: &UserExportQuery) -> AppResult<UserListFilter> {
    build_filter(UserFilterFields::from(query), CursorPageRequest::default())
}

fn build_filter(fields: UserFilterFields, page: CursorPageRequest) -> AppResult<UserListFilter> {
    let range = parse_date_time_range(fields.begin_time.as_deref(), fields.end_time.as_deref()).map_err(created_time_error)?;
    Ok(UserListFilter {
        page,
        username: fields.username,
        nick_name: fields.nick_name,
        phonenumber: fields.phonenumber,
        email: fields.email,
        sex: fields.sex,
        status: fields.status,
        dept_id: fields.dept_id,
        dept_name: fields.dept_name,
        post_ids: split_ids(fields.post_ids),
        role_ids: split_ids(fields.role_ids),
        begin_time: range.begin_time,
        end_time: range.end_time,
    })
}

fn created_time_error(error: DateTimeRangeError) -> AppError {
    let localized = match error {
        DateTimeRangeError::InvalidBoundary(_) => LocalizedError::new(USER_CREATE_TIME_FILTER_ERROR_KEY).with_param("format", DATE_OR_RFC3339_FORMAT),
        DateTimeRangeError::Reversed => LocalizedError::new(USER_CREATE_TIME_RANGE_ERROR_KEY),
    };
    AppError::InvalidInput(localized)
}

fn split_ids(value: Option<String>) -> Vec<String> {
    value
        .map(|value| value.split(',').map(str::trim).filter(|item| !item.is_empty()).map(str::to_owned).collect())
        .unwrap_or_default()
}

struct UserFilterFields {
    limit: u64,
    cursor: Option<String>,
    username: Option<String>,
    nick_name: Option<String>,
    phonenumber: Option<String>,
    email: Option<String>,
    sex: Option<String>,
    status: Option<String>,
    dept_id: Option<String>,
    dept_name: Option<String>,
    post_ids: Option<String>,
    role_ids: Option<String>,
    begin_time: Option<String>,
    end_time: Option<String>,
}

impl From<ListUsersQuery> for UserFilterFields {
    fn from(query: ListUsersQuery) -> Self {
        Self {
            limit: query.limit,
            cursor: query.cursor,
            username: query.username,
            nick_name: query.nick_name,
            phonenumber: query.phonenumber,
            email: query.email,
            sex: query.sex,
            status: query.status,
            dept_id: query.dept_id,
            dept_name: query.dept_name,
            post_ids: query.post_ids,
            role_ids: query.role_ids,
            begin_time: query.begin_time,
            end_time: query.end_time,
        }
    }
}

impl From<&UserExportQuery> for UserFilterFields {
    fn from(query: &UserExportQuery) -> Self {
        Self {
            limit: kernel::pagination::DEFAULT_CURSOR_LIMIT,
            cursor: None,
            username: query.username.clone(),
            nick_name: query.nick_name.clone(),
            phonenumber: query.phonenumber.clone(),
            email: query.email.clone(),
            sex: query.sex.clone(),
            status: query.status.clone(),
            dept_id: query.dept_id.clone(),
            dept_name: query.dept_name.clone(),
            post_ids: query.post_ids.clone(),
            role_ids: query.role_ids.clone(),
            begin_time: query.begin_time.clone(),
            end_time: query.end_time.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};

    use super::*;

    #[test]
    fn export_filter_preserves_rfc3339_instant_precision_and_id_filters() {
        let query = UserExportQuery {
            post_ids: Some(" 1, 2 ".into()),
            role_ids: Some("3".into()),
            begin_time: Some("2026-07-08T20:00:00.001+08:00".into()),
            end_time: Some("2026-07-08T20:00:00.002+08:00".into()),
            ..Default::default()
        };

        let filter = export_user_filter(&query).unwrap();

        assert_eq!(filter.begin_time, Some(timestamp("2026-07-08T12:00:00.001Z")));
        assert_eq!(filter.end_time, Some(timestamp("2026-07-08T12:00:00.002Z")));
        assert_eq!(filter.post_ids, vec!["1", "2"]);
        assert_eq!(filter.role_ids, vec!["3"]);
    }

    #[test]
    fn invalid_and_reversed_ranges_map_to_stable_user_errors() {
        let invalid = UserExportQuery {
            begin_time: Some("invalid".into()),
            ..Default::default()
        };
        let reversed = UserExportQuery {
            begin_time: Some("2026-07-08T12:00:00.001Z".into()),
            end_time: Some("2026-07-08T12:00:00Z".into()),
            ..Default::default()
        };

        assert_error_key(export_user_filter(&invalid), USER_CREATE_TIME_FILTER_ERROR_KEY);
        assert_error_key(export_user_filter(&reversed), USER_CREATE_TIME_RANGE_ERROR_KEY);
    }

    fn assert_error_key(result: AppResult<UserListFilter>, expected: &str) {
        let Err(AppError::InvalidInput(error)) = result else {
            panic!("expected invalid input error");
        };
        assert_eq!(error.key(), expected);
    }

    fn timestamp(value: &str) -> OffsetDateTime {
        OffsetDateTime::parse(value, &Rfc3339).unwrap()
    }
}
