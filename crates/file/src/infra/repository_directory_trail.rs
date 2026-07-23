use sqlx::{Postgres, QueryBuilder};
use storage::Database;

use crate::FileResult;
use crate::application::{DirectoryTrailEntry, FileAccessScope};
use crate::domain::DirectoryId;

use super::repository_support::{scope_query, storage_error};

#[derive(sqlx::FromRow)]
struct DirectoryTrailRecord {
    id: String,
    parent_id: Option<String>,
    name: String,
}

pub(super) async fn directory_trail(database: &Database, actor: &FileAccessScope, directory_id: DirectoryId) -> FileResult<Vec<DirectoryTrailEntry>> {
    directory_trail_query(actor, directory_id)
        .build_query_as::<DirectoryTrailRecord>()
        .fetch_all(database.pool())
        .await
        .map_err(storage_error)
        .map(|records| {
            records
                .into_iter()
                .map(|record| DirectoryTrailEntry {
                    id: record.id,
                    parent_id: record.parent_id,
                    name: record.name,
                })
                .collect()
        })
}

fn directory_trail_query(actor: &FileAccessScope, directory_id: DirectoryId) -> QueryBuilder<Postgres> {
    let mut query = QueryBuilder::new(
        "WITH RECURSIVE directory_trail AS (SELECT e.entry_id,e.parent_id,e.name,e.space_id,ARRAY[e.entry_id]::TEXT[] AS visited_entry_ids,FALSE AS has_cycle,0 AS depth FROM file_entry e JOIN file_space s ON s.space_id=e.space_id WHERE e.entry_id=",
    );
    query.push_bind(directory_id.to_string()).push(" AND e.kind='folder' AND e.status='active' AND");
    scope_query(&mut query, actor, "s");
    query.push(
        " UNION ALL SELECT parent.entry_id,parent.parent_id,parent.name,parent.space_id,directory_trail.visited_entry_ids || parent.entry_id::TEXT,parent.entry_id::TEXT=ANY(directory_trail.visited_entry_ids),directory_trail.depth+1 FROM directory_trail JOIN file_entry parent ON parent.entry_id=directory_trail.parent_id AND parent.space_id=directory_trail.space_id AND parent.kind='folder' AND parent.status='active' WHERE NOT directory_trail.has_cycle) SELECT entry_id AS id,parent_id,name FROM directory_trail WHERE NOT EXISTS (SELECT 1 FROM directory_trail WHERE has_cycle) AND (SELECT parent_id IS NULL FROM directory_trail ORDER BY depth DESC LIMIT 1) ORDER BY depth DESC",
    );
    query
}

#[cfg(test)]
mod tests {
    use crate::application::FileAccessScope;
    use crate::domain::DirectoryId;

    use super::directory_trail_query;

    #[test]
    fn directory_trail_sql_is_recursive_scoped_and_rejects_invalid_chains() {
        let query = directory_trail_query(&FileAccessScope::self_only("actor", None), DirectoryId::new());
        let query_sql = query.sql();
        let sql = query_sql.as_str();

        assert!(sql.contains("WITH RECURSIVE directory_trail AS"));
        assert!(sql.contains("JOIN file_space s ON s.space_id=e.space_id"));
        assert!(sql.contains("s.owner_user_id=$2"));
        assert!(sql.contains("parent.kind='folder' AND parent.status='active'"));
        assert!(sql.contains("parent.entry_id::TEXT=ANY(directory_trail.visited_entry_ids)"));
        assert!(sql.contains("NOT EXISTS (SELECT 1 FROM directory_trail WHERE has_cycle)"));
        assert!(sql.contains("SELECT parent_id IS NULL FROM directory_trail ORDER BY depth DESC LIMIT 1"));
        assert!(sql.ends_with("ORDER BY depth DESC"));
    }
}
