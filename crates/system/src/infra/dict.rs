use kernel::pagination::CursorPage;
use sqlx::{AssertSqlSafe, query, query_as, query_scalar};
use storage::{Database, StorageError, StorageResult};
use time::OffsetDateTime;

use crate::{
    application::{DictDataListFilter, DictTypeListFilter, SystemResult},
    domain::{DictData, DictDataInput, DictType, DictTypeInput},
};

use super::{
    mapping::{dict_data, dict_type},
    record::{DictDataRecord, DictTypeRecord},
};

pub(super) const TYPE_COLUMNS: &str = "dict_id,dict_name,dict_type,status,remark,create_time";
pub(super) const DATA_COLUMNS: &str = "dict_code,dict_sort,dict_label,dict_value,dict_type,css_class,list_class,is_default,status,remark,create_time";

#[path = "dict_pages.rs"]
mod pages;
pub(super) use pages::{data_filtered_query, type_filtered_query};

#[derive(Clone)]
pub struct DictQueries {
    pub(super) database: Database,
}

impl DictQueries {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn page_types(&self, filter: DictTypeListFilter) -> SystemResult<CursorPage<DictType>> {
        pages::page_types(&self.database, filter).await
    }

    pub async fn list_types(&self, filter: DictTypeListFilter) -> StorageResult<Vec<DictType>> {
        let mut query = pages::type_filtered_query(&filter);
        query.push(" ORDER BY dict_id ASC");
        query
            .build_query_as::<DictTypeRecord>()
            .fetch_all(self.database.pool())
            .await
            .map_err(StorageError::from)?
            .into_iter()
            .map(dict_type)
            .collect()
    }

    pub async fn create_type(&self, input: DictTypeInput) -> StorageResult<DictType> {
        let id = self.database.next_id();
        query("INSERT INTO sys_dict_type (dict_id,dict_name,dict_type,status,remark,create_time) VALUES ($1,$2,$3,$4,$5,$6)")
            .bind(&id)
            .bind(input.dict_name)
            .bind(input.dict_type)
            .bind(input.status)
            .bind(input.remark)
            .bind(OffsetDateTime::now_utc())
            .execute(self.database.pool())
            .await?;
        self.find_type(&id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn replace_type(&self, id: &str, input: DictTypeInput) -> StorageResult<DictType> {
        let old = self.find_type(id).await?.ok_or(StorageError::NotFound)?;
        let mut tx = self.database.pool().begin().await?;
        let result = query("UPDATE sys_dict_type SET dict_name=$2,dict_type=$3,status=$4,remark=$5,update_time=CURRENT_TIMESTAMP WHERE dict_id=$1")
            .bind(id)
            .bind(&input.dict_name)
            .bind(&input.dict_type)
            .bind(&input.status)
            .bind(&input.remark)
            .execute(&mut *tx)
            .await?;
        ensure_rows(result.rows_affected())?;
        if old.dict_type != input.dict_type {
            query("UPDATE sys_dict_data SET dict_type=$2,update_time=CURRENT_TIMESTAMP WHERE dict_type=$1")
                .bind(old.dict_type)
                .bind(&input.dict_type)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await.map_err(StorageError::from)?;
        self.find_type(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete_type(&self, id: &str) -> StorageResult<()> {
        let result = query("DELETE FROM sys_dict_type WHERE dict_id = $1")
            .bind(id)
            .execute(self.database.pool())
            .await?;
        ensure_rows(result.rows_affected())
    }

    pub async fn delete_types_many(&self, ids: &[String]) -> StorageResult<()> {
        let mut tx = self.database.pool().begin().await?;
        let result = query("DELETE FROM sys_dict_type WHERE dict_id = ANY($1)").bind(ids).execute(&mut *tx).await?;
        ensure_batch_rows(result.rows_affected(), ids.len())?;
        tx.commit().await.map_err(StorageError::from)
    }

    pub async fn find_type(&self, id: &str) -> StorageResult<Option<DictType>> {
        query_as::<_, DictTypeRecord>(AssertSqlSafe(format!("SELECT {TYPE_COLUMNS} FROM sys_dict_type WHERE dict_id = $1")))
            .bind(id)
            .fetch_optional(self.database.pool())
            .await
            .map_err(StorageError::from)?
            .map(dict_type)
            .transpose()
    }

    pub async fn type_options(&self) -> StorageResult<Vec<DictType>> {
        query_as::<_, DictTypeRecord>(AssertSqlSafe(format!(
            "SELECT {TYPE_COLUMNS} FROM sys_dict_type WHERE status='0' ORDER BY dict_id ASC"
        )))
        .fetch_all(self.database.pool())
        .await
        .map_err(StorageError::from)?
        .into_iter()
        .map(dict_type)
        .collect()
    }

    pub async fn type_has_data(&self, dict_type: &str) -> StorageResult<bool> {
        query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM sys_dict_data WHERE dict_type=$1)")
            .bind(dict_type)
            .fetch_one(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    pub async fn page_data(&self, filter: DictDataListFilter) -> SystemResult<CursorPage<DictData>> {
        pages::page_data(&self.database, filter).await
    }

    pub async fn create_data(&self, input: DictDataInput) -> StorageResult<DictData> {
        let id = self.database.next_id();
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
            .execute(self.database.pool())
            .await?;
        self.find_data(&id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn replace_data(&self, id: &str, input: DictDataInput) -> StorageResult<DictData> {
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
            .execute(self.database.pool())
            .await?;
        ensure_rows(result.rows_affected())?;
        self.find_data(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete_data(&self, id: &str) -> StorageResult<()> {
        let result = query("DELETE FROM sys_dict_data WHERE dict_code = $1")
            .bind(id)
            .execute(self.database.pool())
            .await?;
        ensure_rows(result.rows_affected())
    }

    pub async fn delete_data_many(&self, ids: &[String]) -> StorageResult<()> {
        let mut tx = self.database.pool().begin().await?;
        let result = query("DELETE FROM sys_dict_data WHERE dict_code = ANY($1)").bind(ids).execute(&mut *tx).await?;
        ensure_batch_rows(result.rows_affected(), ids.len())?;
        tx.commit().await.map_err(StorageError::from)
    }

    pub async fn find_data(&self, id: &str) -> StorageResult<Option<DictData>> {
        query_as::<_, DictDataRecord>(AssertSqlSafe(format!("SELECT {DATA_COLUMNS} FROM sys_dict_data WHERE dict_code = $1")))
            .bind(id)
            .fetch_optional(self.database.pool())
            .await
            .map_err(StorageError::from)?
            .map(dict_data)
            .transpose()
    }

    pub async fn data_by_type(&self, dict_type: &str) -> StorageResult<Vec<DictData>> {
        query_as::<_, DictDataRecord>(AssertSqlSafe(format!(
            "SELECT {DATA_COLUMNS} FROM sys_dict_data WHERE dict_type=$1 AND status='0' ORDER BY dict_sort ASC"
        )))
        .bind(dict_type)
        .fetch_all(self.database.pool())
        .await
        .map_err(StorageError::from)?
        .into_iter()
        .map(dict_data)
        .collect()
    }
}

pub(super) fn insert_data_sql() -> &'static str {
    "INSERT INTO sys_dict_data (dict_code,dict_sort,dict_label,dict_value,dict_type,css_class,list_class,is_default,status,remark,create_time) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)"
}
pub(super) fn update_data_sql() -> &'static str {
    "UPDATE sys_dict_data SET dict_sort=$2,dict_label=$3,dict_value=$4,dict_type=$5,css_class=$6,list_class=$7,is_default=$8,status=$9,remark=$10,update_time=CURRENT_TIMESTAMP WHERE dict_code=$1"
}
pub(super) fn ensure_rows(rows: u64) -> StorageResult<()> {
    if rows == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

pub(super) fn ensure_batch_rows(rows: u64, expected: usize) -> StorageResult<()> {
    if rows != expected as u64 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
#[path = "dict_tests.rs"]
mod tests;
