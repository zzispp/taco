use sqlx::PgConnection;
use storage::{StorageError, database::to_i64};

use crate::{
    application::{RbacResult, RoleExportRequest, RoleExportSink, RoleListFilter, cursor::RoleBoundary},
    domain::DataScopeFilter,
    infra::{
        mapping::{role, storage_error},
        records::RoleRecord,
    },
};

use super::{RoleQueries, pages::role_query};

struct ExportBatch<'a> {
    filter: &'a RoleListFilter,
    scope: Option<&'a DataScopeFilter>,
    boundary: Option<&'a RoleBoundary>,
    limit: u64,
}

impl RoleQueries {
    pub async fn export_roles(&self, request: RoleExportRequest, sink: &mut dyn RoleExportSink) -> RbacResult<()> {
        let mut transaction = self.database.begin_consistent_snapshot().await.map_err(storage_error)?;
        let mut boundary = None;
        loop {
            let records = export_batch(
                &mut transaction,
                ExportBatch {
                    filter: &request.filter,
                    scope: request.scope.as_ref(),
                    boundary: boundary.as_ref(),
                    limit: request.batch_size,
                },
            )
            .await?;
            if records.is_empty() {
                break;
            }
            boundary = records.last().map(|record| RoleBoundary {
                role_sort: record.role_sort,
                role_id: record.role_id.clone(),
            });
            let loaded = u64::try_from(records.len()).map_err(numeric_error)?;
            let roles = records.into_iter().map(role).collect::<Result<Vec<_>, _>>().map_err(storage_error)?;
            sink.append(&roles)?;
            if loaded < request.batch_size {
                break;
            }
        }
        transaction.commit().await.map_err(StorageError::from).map_err(storage_error)
    }
}

async fn export_batch(connection: &mut PgConnection, batch: ExportBatch<'_>) -> RbacResult<Vec<RoleRecord>> {
    let mut query = role_query(batch.filter, batch.scope);
    if let Some(boundary) = batch.boundary {
        query.push(" AND (r.role_sort,r.role_id)>(").push_bind(boundary.role_sort);
        query.push(",").push_bind(boundary.role_id.clone()).push(")");
    }
    query.push(" ORDER BY r.role_sort ASC,r.role_id ASC LIMIT ");
    query.push_bind(to_i64(batch.limit).map_err(numeric_error)?);
    query
        .build_query_as::<RoleRecord>()
        .fetch_all(connection)
        .await
        .map_err(StorageError::from)
        .map_err(storage_error)
}

fn numeric_error(error: impl std::fmt::Display) -> crate::application::RbacError {
    crate::application::RbacError::Infrastructure(format!("role export numeric conversion error: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::pagination::CursorPageRequest;

    #[test]
    fn export_query_uses_forward_keyset_without_offset() {
        let filter = RoleListFilter {
            page: CursorPageRequest::default(),
            role_name: None,
            role_key: None,
            status: None,
            system: None,
            begin_time: None,
            end_time: None,
        };
        let boundary = RoleBoundary {
            role_sort: 10,
            role_id: "role-1".into(),
        };
        let mut query = role_query(&filter, None);
        query.push(" AND (r.role_sort,r.role_id)>(").push_bind(boundary.role_sort);
        query
            .push(",")
            .push_bind(boundary.role_id)
            .push(") ORDER BY r.role_sort ASC,r.role_id ASC LIMIT ");
        query.push_bind(100_i64);
        let sql = query.sql();
        let sql = sql.as_str();

        assert!(sql.contains("(r.role_sort,r.role_id)>("));
        assert!(sql.contains("ORDER BY r.role_sort ASC,r.role_id ASC"));
        assert!(!sql.contains("OFFSET"));
    }
}
