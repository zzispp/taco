use kernel::pagination::Page;
use sqlx::{AssertSqlSafe, query, query_as, query_scalar};
use storage::{Database, StorageError, StorageResult};
use time::OffsetDateTime;

use crate::{
    application::{DictDataListFilter, DictTypeListFilter},
    domain::{DictData, DictDataInput, DictType, DictTypeInput},
};

use super::{
    mapping::{dict_data, dict_type},
    page,
    record::{DictDataRecord, DictTypeRecord},
};

const TYPE_COLUMNS: &str = "dict_id,dict_name,dict_type,status,remark,create_time::text AS create_time";
const DATA_COLUMNS: &str = "dict_code,dict_sort,dict_label,dict_value,dict_type,css_class,list_class,is_default,status,remark,create_time::text AS create_time";

#[derive(Clone)]
pub struct DictQueries {
    database: Database,
}

impl DictQueries {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn page_types(&self, filter: DictTypeListFilter) -> StorageResult<Page<DictType>> {
        let total = query_scalar::<_, i64>(AssertSqlSafe(type_total_sql()))
            .bind(&filter.dict_name)
            .bind(&filter.dict_type)
            .bind(&filter.status)
            .bind(&filter.begin_time)
            .bind(&filter.end_time)
            .fetch_one(self.database.pool())
            .await?;
        let rows = query_as::<_, DictTypeRecord>(AssertSqlSafe(type_page_sql()))
            .bind(&filter.dict_name)
            .bind(&filter.dict_type)
            .bind(&filter.status)
            .bind(&filter.begin_time)
            .bind(&filter.end_time)
            .bind(page::limit(filter.page)?)
            .bind(page::offset(filter.page)?)
            .fetch_all(self.database.pool())
            .await?;
        page::page(rows.into_iter().map(dict_type).collect(), total, filter.page)
    }

    pub async fn list_types(&self, filter: DictTypeListFilter) -> StorageResult<Vec<DictType>> {
        query_as::<_, DictTypeRecord>(AssertSqlSafe(type_list_sql()))
            .bind(&filter.dict_name)
            .bind(&filter.dict_type)
            .bind(&filter.status)
            .bind(&filter.begin_time)
            .bind(&filter.end_time)
            .fetch_all(self.database.pool())
            .await
            .map(|rows| rows.into_iter().map(dict_type).collect())
            .map_err(StorageError::from)
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
            .map(|row| row.map(dict_type))
            .map_err(StorageError::from)
    }

    pub async fn type_options(&self) -> StorageResult<Vec<DictType>> {
        query_as::<_, DictTypeRecord>(AssertSqlSafe(format!(
            "SELECT {TYPE_COLUMNS} FROM sys_dict_type WHERE status='0' ORDER BY dict_id ASC"
        )))
        .fetch_all(self.database.pool())
        .await
        .map(|rows| rows.into_iter().map(dict_type).collect())
        .map_err(StorageError::from)
    }

    pub async fn type_has_data(&self, dict_type: &str) -> StorageResult<bool> {
        query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM sys_dict_data WHERE dict_type=$1)")
            .bind(dict_type)
            .fetch_one(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    pub async fn page_data(&self, filter: DictDataListFilter) -> StorageResult<Page<DictData>> {
        let total = query_scalar::<_, i64>(AssertSqlSafe(data_total_sql()))
            .bind(&filter.dict_type)
            .bind(&filter.dict_label)
            .bind(&filter.status)
            .fetch_one(self.database.pool())
            .await?;
        let rows = query_as::<_, DictDataRecord>(AssertSqlSafe(data_page_sql()))
            .bind(&filter.dict_type)
            .bind(&filter.dict_label)
            .bind(&filter.status)
            .bind(page::limit(filter.page)?)
            .bind(page::offset(filter.page)?)
            .fetch_all(self.database.pool())
            .await?;
        page::page(rows.into_iter().map(dict_data).collect(), total, filter.page)
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
            .map(|row| row.map(dict_data))
            .map_err(StorageError::from)
    }

    pub async fn data_by_type(&self, dict_type: &str) -> StorageResult<Vec<DictData>> {
        query_as::<_, DictDataRecord>(AssertSqlSafe(format!(
            "SELECT {DATA_COLUMNS} FROM sys_dict_data WHERE dict_type=$1 AND status='0' ORDER BY dict_sort ASC"
        )))
        .bind(dict_type)
        .fetch_all(self.database.pool())
        .await
        .map(|rows| rows.into_iter().map(dict_data).collect())
        .map_err(StorageError::from)
    }
}

fn type_predicate() -> &'static str {
    "($1::text IS NULL OR dict_name ILIKE '%' || $1 || '%') AND ($2::text IS NULL OR dict_type ILIKE '%' || $2 || '%') AND ($3::text IS NULL OR status=$3) AND ($4::text IS NULL OR create_time::date >= $4::date) AND ($5::text IS NULL OR create_time::date <= $5::date)"
}
fn type_list_sql() -> String {
    format!("SELECT {TYPE_COLUMNS} FROM sys_dict_type WHERE {} ORDER BY dict_id ASC", type_predicate())
}
fn type_page_sql() -> String {
    format!(
        "SELECT {TYPE_COLUMNS} FROM sys_dict_type WHERE {} ORDER BY dict_id ASC LIMIT $6 OFFSET $7",
        type_predicate()
    )
}
fn type_total_sql() -> String {
    format!("SELECT COUNT(*) FROM sys_dict_type WHERE {}", type_predicate())
}
fn data_predicate() -> &'static str {
    "($1::text IS NULL OR dict_type=$1) AND ($2::text IS NULL OR dict_label ILIKE '%' || $2 || '%') AND ($3::text IS NULL OR status=$3)"
}
fn data_page_sql() -> String {
    format!(
        "SELECT {DATA_COLUMNS} FROM sys_dict_data WHERE {} ORDER BY dict_sort ASC LIMIT $4 OFFSET $5",
        data_predicate()
    )
}
fn data_total_sql() -> String {
    format!("SELECT COUNT(*) FROM sys_dict_data WHERE {}", data_predicate())
}
fn insert_data_sql() -> &'static str {
    "INSERT INTO sys_dict_data (dict_code,dict_sort,dict_label,dict_value,dict_type,css_class,list_class,is_default,status,remark,create_time) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)"
}
fn update_data_sql() -> &'static str {
    "UPDATE sys_dict_data SET dict_sort=$2,dict_label=$3,dict_value=$4,dict_type=$5,css_class=$6,list_class=$7,is_default=$8,status=$9,remark=$10,update_time=CURRENT_TIMESTAMP WHERE dict_code=$1"
}
fn ensure_rows(rows: u64) -> StorageResult<()> {
    if rows == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

fn ensure_batch_rows(rows: u64, expected: usize) -> StorageResult<()> {
    if rows != expected as u64 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{data_page_sql, type_page_sql};

    #[test]
    fn dict_text_filters_use_case_insensitive_search() {
        let type_sql = type_page_sql();
        let data_sql = data_page_sql();

        assert!(type_sql.contains("dict_name ILIKE"));
        assert!(type_sql.contains("dict_type ILIKE"));
        assert!(data_sql.contains("dict_label ILIKE"));
    }
}
