use sqlx::{Postgres, Transaction, query};
use storage::StorageResult;

pub(super) async fn delete_user_relations(tx: &mut Transaction<'_, Postgres>, user_ids: &[String]) -> StorageResult<()> {
    if user_ids.is_empty() {
        return Ok(());
    }
    query("DELETE FROM sys_user_role WHERE user_id=ANY($1)")
        .bind(user_ids)
        .execute(&mut **tx)
        .await?;
    query("DELETE FROM sys_user_post WHERE user_id=ANY($1)")
        .bind(user_ids)
        .execute(&mut **tx)
        .await?;
    Ok(())
}
