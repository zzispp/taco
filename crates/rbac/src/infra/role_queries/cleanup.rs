use sqlx::{Postgres, Transaction, query};
use storage::StorageResult;

const ROLE_RELATION_TABLES: &[&str] = &["sys_user_role", "sys_role_menu", "sys_role_dept"];

pub(super) async fn delete_role_relations(tx: &mut Transaction<'_, Postgres>, role_ids: &[String]) -> StorageResult<()> {
    if role_ids.is_empty() {
        return Ok(());
    }
    for table in ROLE_RELATION_TABLES {
        query(sqlx::AssertSqlSafe(format!("DELETE FROM {table} WHERE role_id=ANY($1)")))
            .bind(role_ids)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}
