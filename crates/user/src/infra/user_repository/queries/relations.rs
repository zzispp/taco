use std::collections::HashMap;

use sqlx::{PgConnection, query_as};
use storage::{Database, StorageError, StorageResult};
use types::rbac::RoleSummary;

use crate::infra::user_repository::{
    mapping::UserRelations,
    record::{UserRelationValueRecord, UserRoleRecord},
    sql,
};

pub(super) async fn load(database: &Database, user_ids: &[String]) -> StorageResult<HashMap<String, UserRelations>> {
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let (roles, posts, permissions) = tokio::try_join!(
        query_as::<_, UserRoleRecord>(sql::batch_role_query()).bind(user_ids).fetch_all(database.pool()),
        query_as::<_, UserRelationValueRecord>(sql::batch_post_query())
            .bind(user_ids)
            .fetch_all(database.pool()),
        query_as::<_, UserRelationValueRecord>(sql::batch_permission_query())
            .bind(user_ids)
            .fetch_all(database.pool()),
    )
    .map_err(StorageError::from)?;
    Ok(group(user_ids, RelationBatches { roles, posts, permissions }))
}

pub(super) async fn load_in_connection(connection: &mut PgConnection, user_ids: &[String]) -> StorageResult<HashMap<String, UserRelations>> {
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let roles = query_as::<_, UserRoleRecord>(sql::batch_role_query())
        .bind(user_ids)
        .fetch_all(&mut *connection)
        .await?;
    let posts = query_as::<_, UserRelationValueRecord>(sql::batch_post_query())
        .bind(user_ids)
        .fetch_all(&mut *connection)
        .await?;
    let permissions = query_as::<_, UserRelationValueRecord>(sql::batch_permission_query())
        .bind(user_ids)
        .fetch_all(connection)
        .await?;
    Ok(group(user_ids, RelationBatches { roles, posts, permissions }))
}

struct RelationBatches {
    roles: Vec<UserRoleRecord>,
    posts: Vec<UserRelationValueRecord>,
    permissions: Vec<UserRelationValueRecord>,
}

fn group(user_ids: &[String], batches: RelationBatches) -> HashMap<String, UserRelations> {
    let mut grouped = user_ids.iter().cloned().map(|id| (id, UserRelations::default())).collect::<HashMap<_, _>>();
    for role in batches.roles {
        if let Some(relations) = grouped.get_mut(&role.user_id) {
            relations.role_ids.push(role.role_id.clone());
            relations.roles.push(RoleSummary {
                role_id: role.role_id,
                role_name: role.role_name,
                role_key: role.role_key,
            });
        }
    }
    append_values(&mut grouped, batches.posts, |relations| &mut relations.post_ids);
    append_values(&mut grouped, batches.permissions, |relations| &mut relations.permissions);
    grouped
}

pub(super) fn take(grouped: &mut HashMap<String, UserRelations>, user_id: &str) -> StorageResult<UserRelations> {
    grouped
        .remove(user_id)
        .ok_or_else(|| StorageError::Database(format!("relation batch is missing requested user: {user_id}")))
}

fn append_values(grouped: &mut HashMap<String, UserRelations>, records: Vec<UserRelationValueRecord>, field: impl Fn(&mut UserRelations) -> &mut Vec<String>) {
    for record in records {
        if let Some(relations) = grouped.get_mut(&record.user_id) {
            field(relations).push(record.value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_relation_batches_without_per_user_queries() {
        let users = vec!["u1".into(), "u2".into()];
        let grouped = group(
            &users,
            RelationBatches {
                roles: vec![role("u1", "r1"), role("u2", "r2")],
                posts: vec![value("u1", "p1"), value("u2", "p2")],
                permissions: vec![value("u1", "read"), value("u2", "write")],
            },
        );

        assert_eq!(grouped["u1"].role_ids, vec!["r1"]);
        assert_eq!(grouped["u1"].post_ids, vec!["p1"]);
        assert_eq!(grouped["u2"].permissions, vec!["write"]);
    }

    #[test]
    fn missing_requested_user_is_an_explicit_invariant_failure() {
        let mut grouped = HashMap::new();

        let Err(StorageError::Database(message)) = take(&mut grouped, "missing-user") else {
            panic!("missing relation entry must fail explicitly");
        };

        assert_eq!(message, "relation batch is missing requested user: missing-user");
    }

    fn role(user_id: &str, role_id: &str) -> UserRoleRecord {
        UserRoleRecord {
            user_id: user_id.into(),
            role_id: role_id.into(),
            role_name: role_id.into(),
            role_key: role_id.into(),
        }
    }

    fn value(user_id: &str, value: &str) -> UserRelationValueRecord {
        UserRelationValueRecord {
            user_id: user_id.into(),
            value: value.into(),
        }
    }
}
