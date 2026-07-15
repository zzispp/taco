use rbac::domain::DataScopeFilter;
use sqlx::{AssertSqlSafe, PgConnection, query_as};
use storage::{StorageError, StorageResult, database::to_i64};
use time::OffsetDateTime;

use crate::{
    application::{AppResult, UserExportRequest, UserExportSink, UserListFilter},
    domain::User,
    infra::user_repository::{filter_sql, mapping::user, record::UserRecord},
};

use super::{UserQueries, relations};

struct ExportWindow {
    limit: u64,
    boundary: Option<(OffsetDateTime, String)>,
}

struct ScopedExportSpec<'a> {
    filter: &'a UserListFilter,
    scope: &'a DataScopeFilter,
    window: ExportWindow,
}

impl UserQueries {
    pub async fn export_users(&self, request: UserExportRequest, sink: &mut dyn UserExportSink) -> AppResult<()> {
        let mut transaction = self.database.begin_consistent_snapshot().await.map_err(super::super::mapping::storage_error)?;
        let mut boundary = None;
        loop {
            let records = export_records(
                &mut transaction,
                &request,
                ExportWindow {
                    limit: request.batch_size,
                    boundary,
                },
            )
            .await
            .map_err(super::super::mapping::storage_error)?;
            if records.is_empty() {
                break;
            }
            boundary = records.last().map(|record| (record.create_time, record.user_id.clone()));
            let loaded = u64::try_from(records.len()).map_err(numeric_error)?;
            let users = hydrate(&mut transaction, records).await.map_err(super::super::mapping::storage_error)?;
            sink.append(&users)?;
            if loaded < request.batch_size {
                break;
            }
        }
        transaction
            .commit()
            .await
            .map_err(StorageError::from)
            .map_err(super::super::mapping::storage_error)
    }
}

async fn export_records(connection: &mut PgConnection, request: &UserExportRequest, window: ExportWindow) -> StorageResult<Vec<UserRecord>> {
    match &request.scope {
        Some(scope) => {
            scoped_records(
                connection,
                ScopedExportSpec {
                    filter: &request.filter,
                    scope,
                    window,
                },
            )
            .await
        }
        None => filtered_records(connection, &request.filter, window).await,
    }
}

async fn filtered_records(connection: &mut PgConnection, filter: &UserListFilter, window: ExportWindow) -> StorageResult<Vec<UserRecord>> {
    let statement = filter_sql::filtered_users(
        "AND ($13::timestamptz IS NULL OR (u.create_time,u.user_id) > ($13,$14)) \
         ORDER BY u.create_time ASC,u.user_id ASC LIMIT $15",
    );
    bind_unscoped(query_as::<_, UserRecord>(AssertSqlSafe(statement)), filter)
        .bind(window.boundary.as_ref().map(|value| value.0))
        .bind(window.boundary.as_ref().map(|value| &value.1))
        .bind(to_i64(window.limit)?)
        .fetch_all(connection)
        .await
        .map_err(StorageError::from)
}

async fn scoped_records(connection: &mut PgConnection, spec: ScopedExportSpec<'_>) -> StorageResult<Vec<UserRecord>> {
    let statement = filter_sql::scoped_users(
        "AND ($17::timestamptz IS NULL OR (u.create_time,u.user_id) > ($17,$18)) \
         ORDER BY u.create_time ASC,u.user_id ASC LIMIT $19",
    );
    bind_scoped(query_as::<_, UserRecord>(AssertSqlSafe(statement)), spec.filter, spec.scope)
        .bind(spec.window.boundary.as_ref().map(|value| value.0))
        .bind(spec.window.boundary.as_ref().map(|value| &value.1))
        .bind(to_i64(spec.window.limit)?)
        .fetch_all(connection)
        .await
        .map_err(StorageError::from)
}

async fn hydrate(connection: &mut PgConnection, records: Vec<UserRecord>) -> StorageResult<Vec<User>> {
    let ids = records.iter().map(|record| record.user_id.clone()).collect::<Vec<_>>();
    let mut relations = relations::load_in_connection(connection, &ids).await?;
    records
        .into_iter()
        .map(|record| {
            let relation = relations::take(&mut relations, &record.user_id)?;
            user(record, relation)
        })
        .collect()
}

fn bind_unscoped<'q>(
    query: sqlx::query::QueryAs<'q, sqlx::Postgres, UserRecord, sqlx::postgres::PgArguments>,
    filter: &'q UserListFilter,
) -> sqlx::query::QueryAs<'q, sqlx::Postgres, UserRecord, sqlx::postgres::PgArguments> {
    query
        .bind(&filter.username)
        .bind(&filter.phonenumber)
        .bind(&filter.status)
        .bind(&filter.dept_id)
        .bind(filter.begin_time)
        .bind(filter.end_time)
        .bind(&filter.nick_name)
        .bind(&filter.dept_name)
        .bind(&filter.email)
        .bind(&filter.sex)
        .bind(&filter.post_ids)
        .bind(&filter.role_ids)
}

fn bind_scoped<'q>(
    query: sqlx::query::QueryAs<'q, sqlx::Postgres, UserRecord, sqlx::postgres::PgArguments>,
    filter: &'q UserListFilter,
    scope: &'q DataScopeFilter,
) -> sqlx::query::QueryAs<'q, sqlx::Postgres, UserRecord, sqlx::postgres::PgArguments> {
    query
        .bind(scope.data_scope.code())
        .bind(&scope.user_id)
        .bind(&scope.dept_id)
        .bind(&scope.dept_ids)
        .bind(&filter.username)
        .bind(&filter.phonenumber)
        .bind(&filter.status)
        .bind(&filter.dept_id)
        .bind(filter.begin_time)
        .bind(filter.end_time)
        .bind(&filter.nick_name)
        .bind(&filter.dept_name)
        .bind(&filter.email)
        .bind(&filter.sex)
        .bind(&filter.post_ids)
        .bind(&filter.role_ids)
}

fn numeric_error(error: impl std::fmt::Display) -> crate::application::AppError {
    crate::application::AppError::Infrastructure(format!("user export numeric conversion error: {error}"))
}
