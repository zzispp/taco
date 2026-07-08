use kernel::pagination::Page;
use sqlx::{AssertSqlSafe, query, query_as, query_scalar};
use storage::{
    Database, StorageError, StorageResult,
    database::{to_i64, to_u64},
};
use time::OffsetDateTime;

use crate::{
    application::MenuListFilter,
    domain::{MENU_TYPE_BUTTON, Menu, MenuInput},
};
use types::system::SortBatchInput;

use super::{
    mapping::menu,
    records::{MenuRecord, RoleMenuRecord},
};

const MENU_COLUMNS: &str = r#"
    menu_id, menu_name, parent_id, order_num, path, component, query, route_name,
    is_frame, is_cache, menu_type, visible, status, perms, icon, remark
"#;

#[derive(Clone)]
pub struct MenuQueries {
    database: Database,
}

impl MenuQueries {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create(&self, input: MenuInput) -> StorageResult<Menu> {
        let id = self.database.next_id();
        query(insert_menu_sql())
            .bind(&id)
            .bind(input.menu_name)
            .bind(input.parent_id)
            .bind(input.order_num)
            .bind(input.path)
            .bind(input.component)
            .bind(input.query)
            .bind(input.route_name)
            .bind(input.is_frame)
            .bind(input.is_cache)
            .bind(input.menu_type)
            .bind(input.visible)
            .bind(input.status)
            .bind(input.perms)
            .bind(input.icon)
            .bind(input.remark)
            .bind(OffsetDateTime::now_utc())
            .execute(self.database.pool())
            .await?;
        self.find(&id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn replace(&self, id: &str, input: MenuInput) -> StorageResult<Menu> {
        let result = query(update_menu_sql())
            .bind(id)
            .bind(input.menu_name)
            .bind(input.parent_id)
            .bind(input.order_num)
            .bind(input.path)
            .bind(input.component)
            .bind(input.query)
            .bind(input.route_name)
            .bind(input.is_frame)
            .bind(input.is_cache)
            .bind(input.menu_type)
            .bind(input.visible)
            .bind(input.status)
            .bind(input.perms)
            .bind(input.icon)
            .bind(input.remark)
            .execute(self.database.pool())
            .await?;
        ensure_rows_affected(result.rows_affected())?;
        self.find(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn update_sort(&self, id: &str, order_num: i64) -> StorageResult<Menu> {
        let result = query("UPDATE sys_menu SET order_num=$2, update_time=CURRENT_TIMESTAMP WHERE menu_id=$1")
            .bind(id)
            .bind(order_num)
            .execute(self.database.pool())
            .await?;
        ensure_rows_affected(result.rows_affected())?;
        self.find(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn update_sorts(&self, input: SortBatchInput) -> StorageResult<Vec<Menu>> {
        let mut tx = self.database.pool().begin().await?;
        for item in &input.items {
            let result = query("UPDATE sys_menu SET order_num=$2, update_time=CURRENT_TIMESTAMP WHERE menu_id=$1")
                .bind(&item.id)
                .bind(item.order_num)
                .execute(&mut *tx)
                .await?;
            ensure_rows_affected(result.rows_affected())?;
        }
        tx.commit().await.map_err(StorageError::from)?;
        self.find_many(input.items.into_iter().map(|item| item.id).collect()).await
    }

    pub async fn delete(&self, id: &str) -> StorageResult<()> {
        let result = query("DELETE FROM sys_menu WHERE menu_id = $1").bind(id).execute(self.database.pool()).await?;
        ensure_rows_affected(result.rows_affected())
    }

    pub async fn find(&self, id: &str) -> StorageResult<Option<Menu>> {
        query_as::<_, MenuRecord>(AssertSqlSafe(format!("SELECT {MENU_COLUMNS} FROM sys_menu WHERE menu_id = $1")))
            .bind(id)
            .fetch_optional(self.database.pool())
            .await
            .map(|record| record.map(menu))
            .map_err(StorageError::from)
    }

    pub async fn has_children(&self, id: &str) -> StorageResult<bool> {
        query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM sys_menu WHERE parent_id = $1)")
            .bind(id)
            .fetch_one(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    pub async fn has_role_bindings(&self, id: &str) -> StorageResult<bool> {
        query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM sys_role_menu WHERE menu_id = $1)")
            .bind(id)
            .fetch_one(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    pub async fn list(&self) -> StorageResult<Vec<Menu>> {
        query_as::<_, MenuRecord>(AssertSqlSafe(format!(
            "SELECT {MENU_COLUMNS} FROM sys_menu ORDER BY parent_id ASC, order_num ASC"
        )))
        .fetch_all(self.database.pool())
        .await
        .map(|records| records.into_iter().map(menu).collect())
        .map_err(StorageError::from)
    }

    pub async fn page(&self, filter: MenuListFilter) -> StorageResult<Page<Menu>> {
        let total = query_scalar::<_, i64>(AssertSqlSafe(menu_total_sql()))
            .bind(&filter.menu_name)
            .bind(&filter.status)
            .fetch_one(self.database.pool())
            .await?;
        let items = query_as::<_, MenuRecord>(AssertSqlSafe(menu_page_sql()))
            .bind(&filter.menu_name)
            .bind(&filter.status)
            .bind(to_i64(filter.page.page_size)?)
            .bind(to_i64((filter.page.page - 1) * filter.page.page_size)?)
            .fetch_all(self.database.pool())
            .await?;
        Ok(Page {
            items: items.into_iter().map(menu).collect(),
            total: to_u64(total)?,
            page: filter.page.page,
            page_size: filter.page.page_size,
        })
    }

    pub async fn role_menu_rows(&self) -> StorageResult<Vec<RoleMenuRecord>> {
        query_as::<_, RoleMenuRecord>(role_menu_query())
            .bind(MENU_TYPE_BUTTON)
            .fetch_all(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    async fn find_many(&self, ids: Vec<String>) -> StorageResult<Vec<Menu>> {
        let mut menus = Vec::with_capacity(ids.len());
        for id in ids {
            menus.push(self.find(&id).await?.ok_or(StorageError::NotFound)?);
        }
        Ok(menus)
    }
}

fn insert_menu_sql() -> &'static str {
    "INSERT INTO sys_menu (menu_id, menu_name, parent_id, order_num, path, component, query, route_name, is_frame, is_cache, menu_type, visible, status, perms, icon, remark, create_time) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)"
}

fn update_menu_sql() -> &'static str {
    "UPDATE sys_menu SET menu_name=$2,parent_id=$3,order_num=$4,path=$5,component=$6,query=$7,route_name=$8,is_frame=$9,is_cache=$10,menu_type=$11,visible=$12,status=$13,perms=$14,icon=$15,remark=$16,update_time=CURRENT_TIMESTAMP WHERE menu_id=$1"
}

fn role_menu_query() -> &'static str {
    r#"
    SELECT r.role_key, m.menu_id, m.menu_name, m.parent_id, m.path, m.menu_type, m.icon, m.order_num
    FROM sys_role r
    CROSS JOIN sys_menu m
    WHERE r.role_key = 'admin' AND r.del_flag = '0' AND r.status = '0' AND m.status = '0' AND m.visible = '0' AND m.menu_type <> $1
    UNION
    SELECT r.role_key, m.menu_id, m.menu_name, m.parent_id, m.path, m.menu_type, m.icon, m.order_num
    FROM sys_role r
    INNER JOIN sys_role_menu rm ON rm.role_id = r.role_id
    INNER JOIN sys_menu m ON m.menu_id = rm.menu_id
    WHERE r.role_key <> 'admin' AND r.del_flag = '0' AND r.status = '0' AND m.status = '0' AND m.visible = '0' AND m.menu_type <> $1
    ORDER BY role_key ASC, parent_id ASC, order_num ASC
    "#
}

fn ensure_rows_affected(rows_affected: u64) -> StorageResult<()> {
    if rows_affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

fn menu_where() -> &'static str {
    "($1::text IS NULL OR menu_name ILIKE '%' || $1 || '%') AND ($2::text IS NULL OR status=$2)"
}

fn menu_page_sql() -> String {
    format!(
        "SELECT {MENU_COLUMNS} FROM sys_menu WHERE {} ORDER BY parent_id ASC, order_num ASC LIMIT $3 OFFSET $4",
        menu_where()
    )
}

fn menu_total_sql() -> String {
    format!("SELECT COUNT(*) FROM sys_menu WHERE {}", menu_where())
}
