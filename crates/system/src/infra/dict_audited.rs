use audit_contract::AuditOutboxRecord;
use sqlx::query;
use storage::{StorageError, StorageResult};
use time::OffsetDateTime;

use crate::domain::{DictData, DictDataInput, DictType, DictTypeInput};

use super::{
    audited_transaction::commit_audited_write,
    dict::{DictQueries, ensure_batch_rows, ensure_rows, insert_data_sql, update_data_sql},
};

impl DictQueries {
    pub(in crate::infra) async fn create_type_with_audit(&self, input: DictTypeInput, audit: &AuditOutboxRecord) -> StorageResult<DictType> {
        let id = self.database.next_id();
        let mut transaction = self.database.pool().begin().await?;
        query("INSERT INTO sys_dict_type (dict_id,dict_name,dict_type,status,remark,create_time) VALUES ($1,$2,$3,$4,$5,$6)")
            .bind(&id)
            .bind(input.dict_name)
            .bind(input.dict_type)
            .bind(input.status)
            .bind(input.remark)
            .bind(OffsetDateTime::now_utc())
            .execute(&mut *transaction)
            .await?;
        commit_audited_write(transaction, audit).await?;
        self.find_type(&id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::infra) async fn replace_type_with_audit(&self, id: &str, input: DictTypeInput, audit: &AuditOutboxRecord) -> StorageResult<DictType> {
        let old = self.find_type(id).await?.ok_or(StorageError::NotFound)?;
        let old_type = old.dict_type;
        let renamed = old_type != input.dict_type;
        let mut transaction = self.database.pool().begin().await?;
        let result = query("UPDATE sys_dict_type SET dict_name=$2,dict_type=$3,status=$4,remark=$5,update_time=CURRENT_TIMESTAMP WHERE dict_id=$1")
            .bind(id)
            .bind(input.dict_name)
            .bind(&input.dict_type)
            .bind(input.status)
            .bind(input.remark)
            .execute(&mut *transaction)
            .await?;
        ensure_rows(result.rows_affected())?;
        if renamed {
            query("UPDATE sys_dict_data SET dict_type=$2,update_time=CURRENT_TIMESTAMP WHERE dict_type=$1")
                .bind(old_type)
                .bind(input.dict_type)
                .execute(&mut *transaction)
                .await?;
        }
        commit_audited_write(transaction, audit).await?;
        self.find_type(id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::infra) async fn delete_type_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> StorageResult<()> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query("DELETE FROM sys_dict_type WHERE dict_id = $1")
            .bind(id)
            .execute(&mut *transaction)
            .await?;
        ensure_rows(result.rows_affected())?;
        commit_audited_write(transaction, audit).await
    }

    pub(in crate::infra) async fn delete_types_many_with_audit(&self, ids: &[String], audit: &AuditOutboxRecord) -> StorageResult<()> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query("DELETE FROM sys_dict_type WHERE dict_id = ANY($1)")
            .bind(ids)
            .execute(&mut *transaction)
            .await?;
        ensure_batch_rows(result.rows_affected(), ids.len())?;
        commit_audited_write(transaction, audit).await
    }

    pub(in crate::infra) async fn create_data_with_audit(&self, input: DictDataInput, audit: &AuditOutboxRecord) -> StorageResult<DictData> {
        let id = self.database.next_id();
        let mut transaction = self.database.pool().begin().await?;
        query(insert_data_sql())
            .bind(&id)
            .bind(input.dict_sort)
            .bind(input.dict_label)
            .bind(input.dict_value)
            .bind(input.dict_type)
            .bind(input.css_class)
            .bind(input.list_class)
            .bind(input.is_default)
            .bind(input.status)
            .bind(input.remark)
            .bind(OffsetDateTime::now_utc())
            .execute(&mut *transaction)
            .await?;
        commit_audited_write(transaction, audit).await?;
        self.find_data(&id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::infra) async fn replace_data_with_audit(&self, id: &str, input: DictDataInput, audit: &AuditOutboxRecord) -> StorageResult<DictData> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query(update_data_sql())
            .bind(id)
            .bind(input.dict_sort)
            .bind(input.dict_label)
            .bind(input.dict_value)
            .bind(input.dict_type)
            .bind(input.css_class)
            .bind(input.list_class)
            .bind(input.is_default)
            .bind(input.status)
            .bind(input.remark)
            .execute(&mut *transaction)
            .await?;
        ensure_rows(result.rows_affected())?;
        commit_audited_write(transaction, audit).await?;
        self.find_data(id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::infra) async fn delete_data_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> StorageResult<()> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query("DELETE FROM sys_dict_data WHERE dict_code = $1")
            .bind(id)
            .execute(&mut *transaction)
            .await?;
        ensure_rows(result.rows_affected())?;
        commit_audited_write(transaction, audit).await
    }

    pub(in crate::infra) async fn delete_data_many_with_audit(&self, ids: &[String], audit: &AuditOutboxRecord) -> StorageResult<()> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query("DELETE FROM sys_dict_data WHERE dict_code = ANY($1)")
            .bind(ids)
            .execute(&mut *transaction)
            .await?;
        ensure_batch_rows(result.rows_affected(), ids.len())?;
        commit_audited_write(transaction, audit).await
    }
}
