use kernel::pagination::{Page, PageSliceRequest};
use sqlx::{query, query_as, query_scalar};
use storage::{
    StorageError, StorageResult,
    database::{to_i64, to_u64},
};
use time::OffsetDateTime;
use types::rbac::{MenuItem, MenuSection};

use super::{
    MenuItemRecord, MenuItemRecordInput, MenuSectionRecord, MenuSectionRecordInput, RbacStore,
    repository::{MENU_ITEM_COLUMNS, MENU_SECTION_COLUMNS, ensure_rows_affected, rbac_page},
};

impl RbacStore {
    pub async fn create_menu_section(&self, input: MenuSectionRecordInput) -> StorageResult<MenuSection> {
        let now = OffsetDateTime::now_utc();
        let record = query_as::<_, MenuSectionRecord>(&format!(
            r#"
            INSERT INTO menu_sections (id, code, subheader, sort_order, enabled, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $6)
            RETURNING {MENU_SECTION_COLUMNS}
            "#
        ))
        .bind(self.database.next_id())
        .bind(input.code)
        .bind(input.subheader)
        .bind(input.sort_order)
        .bind(input.enabled)
        .bind(now)
        .fetch_one(self.database.pool())
        .await?;
        Ok(record.into())
    }

    pub async fn replace_menu_section(&self, id: &str, input: MenuSectionRecordInput) -> StorageResult<MenuSection> {
        let now = OffsetDateTime::now_utc();
        let record = query_as::<_, MenuSectionRecord>(&format!(
            r#"
            UPDATE menu_sections
            SET code = $2,
                subheader = $3,
                sort_order = $4,
                enabled = $5,
                updated_at = $6
            WHERE id = $1
            RETURNING {MENU_SECTION_COLUMNS}
            "#
        ))
        .bind(id)
        .bind(input.code)
        .bind(input.subheader)
        .bind(input.sort_order)
        .bind(input.enabled)
        .bind(now)
        .fetch_optional(self.database.pool())
        .await?;
        record.map(MenuSection::from).ok_or(StorageError::NotFound)
    }

    pub async fn delete_menu_section(&self, id: &str) -> StorageResult<()> {
        let result = query("DELETE FROM menu_sections WHERE id = $1").bind(id).execute(self.database.pool()).await?;
        ensure_rows_affected(result.rows_affected())
    }

    pub async fn find_menu_section(&self, id: &str) -> StorageResult<Option<MenuSection>> {
        self.find_menu_section_record(id).await.map(|record| record.map(MenuSection::from))
    }

    pub async fn list_menu_sections(&self) -> StorageResult<Vec<MenuSection>> {
        query_as::<_, MenuSectionRecord>(&format!(
            r#"
            SELECT {MENU_SECTION_COLUMNS}
            FROM menu_sections
            ORDER BY sort_order ASC
            "#
        ))
        .fetch_all(self.database.pool())
        .await
        .map(|records| records.into_iter().map(MenuSection::from).collect())
        .map_err(StorageError::from)
    }

    pub async fn page_menu_sections(&self, request: PageSliceRequest) -> StorageResult<Page<MenuSection>> {
        let total = query_scalar::<_, i64>("SELECT COUNT(*) FROM menu_sections")
            .fetch_one(self.database.pool())
            .await?;
        let items = query_as::<_, MenuSectionRecord>(&format!(
            r#"
            SELECT {MENU_SECTION_COLUMNS}
            FROM menu_sections
            ORDER BY sort_order ASC
            LIMIT $1 OFFSET $2
            "#
        ))
        .bind(to_i64(request.limit)?)
        .bind(to_i64(request.offset)?)
        .fetch_all(self.database.pool())
        .await?;
        Ok(rbac_page(items.into_iter().map(MenuSection::from).collect(), to_u64(total)?, request))
    }

    pub async fn menu_section_has_items(&self, id: &str) -> StorageResult<bool> {
        query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM menu_items WHERE section_id = $1)")
            .bind(id)
            .fetch_one(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    async fn find_menu_section_record(&self, id: &str) -> StorageResult<Option<MenuSectionRecord>> {
        query_as::<_, MenuSectionRecord>(&format!(
            r#"
            SELECT {MENU_SECTION_COLUMNS}
            FROM menu_sections
            WHERE id = $1
            "#
        ))
        .bind(id)
        .fetch_optional(self.database.pool())
        .await
        .map_err(StorageError::from)
    }

    pub async fn create_menu_item(&self, input: MenuItemRecordInput) -> StorageResult<MenuItem> {
        let now = OffsetDateTime::now_utc();
        let record = query_as::<_, MenuItemRecord>(&format!(
            r#"
            INSERT INTO menu_items (
                id, section_id, parent_id, code, title, route_path, icon, caption,
                deep_match, sort_order, enabled, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $12)
            RETURNING {MENU_ITEM_COLUMNS}
            "#
        ))
        .bind(self.database.next_id())
        .bind(input.section_id)
        .bind(input.parent_id)
        .bind(input.code)
        .bind(input.title)
        .bind(input.path)
        .bind(input.icon)
        .bind(input.caption)
        .bind(input.deep_match)
        .bind(input.sort_order)
        .bind(input.enabled)
        .bind(now)
        .fetch_one(self.database.pool())
        .await?;
        Ok(record.into())
    }

    pub async fn replace_menu_item(&self, id: &str, input: MenuItemRecordInput) -> StorageResult<MenuItem> {
        let now = OffsetDateTime::now_utc();
        let record = query_as::<_, MenuItemRecord>(&format!(
            r#"
            UPDATE menu_items
            SET section_id = $2,
                parent_id = $3,
                code = $4,
                title = $5,
                route_path = $6,
                icon = $7,
                caption = $8,
                deep_match = $9,
                sort_order = $10,
                enabled = $11,
                updated_at = $12
            WHERE id = $1
            RETURNING {MENU_ITEM_COLUMNS}
            "#
        ))
        .bind(id)
        .bind(input.section_id)
        .bind(input.parent_id)
        .bind(input.code)
        .bind(input.title)
        .bind(input.path)
        .bind(input.icon)
        .bind(input.caption)
        .bind(input.deep_match)
        .bind(input.sort_order)
        .bind(input.enabled)
        .bind(now)
        .fetch_optional(self.database.pool())
        .await?;
        record.map(MenuItem::from).ok_or(StorageError::NotFound)
    }

    pub async fn delete_menu_item(&self, id: &str) -> StorageResult<()> {
        let result = query("DELETE FROM menu_items WHERE id = $1").bind(id).execute(self.database.pool()).await?;
        ensure_rows_affected(result.rows_affected())
    }

    pub async fn find_menu_item(&self, id: &str) -> StorageResult<Option<MenuItem>> {
        self.find_menu_item_record(id).await.map(|record| record.map(MenuItem::from))
    }

    pub async fn list_menu_items(&self) -> StorageResult<Vec<MenuItem>> {
        query_as::<_, MenuItemRecord>(&format!(
            r#"
            SELECT {MENU_ITEM_COLUMNS}
            FROM menu_items
            ORDER BY sort_order ASC
            "#
        ))
        .fetch_all(self.database.pool())
        .await
        .map(|records| records.into_iter().map(MenuItem::from).collect())
        .map_err(StorageError::from)
    }

    pub async fn page_menu_items(&self, request: PageSliceRequest) -> StorageResult<Page<MenuItem>> {
        let total = query_scalar::<_, i64>("SELECT COUNT(*) FROM menu_items")
            .fetch_one(self.database.pool())
            .await?;
        let items = query_as::<_, MenuItemRecord>(&format!(
            r#"
            SELECT {MENU_ITEM_COLUMNS}
            FROM menu_items
            ORDER BY sort_order ASC
            LIMIT $1 OFFSET $2
            "#
        ))
        .bind(to_i64(request.limit)?)
        .bind(to_i64(request.offset)?)
        .fetch_all(self.database.pool())
        .await?;
        Ok(rbac_page(items.into_iter().map(MenuItem::from).collect(), to_u64(total)?, request))
    }

    pub async fn menu_item_has_children(&self, id: &str) -> StorageResult<bool> {
        query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM menu_items WHERE parent_id = $1)")
            .bind(id)
            .fetch_one(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    async fn find_menu_item_record(&self, id: &str) -> StorageResult<Option<MenuItemRecord>> {
        query_as::<_, MenuItemRecord>(&format!(
            r#"
            SELECT {MENU_ITEM_COLUMNS}
            FROM menu_items
            WHERE id = $1
            "#
        ))
        .bind(id)
        .fetch_optional(self.database.pool())
        .await
        .map_err(StorageError::from)
    }
}
