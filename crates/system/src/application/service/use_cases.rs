use std::collections::BTreeMap;

use async_trait::async_trait;
use kernel::pagination::CursorPage;
use rbac::domain::DataScopeFilter;

use crate::domain::{
    ConfigInput, ConfigItem, Dept, DeptInput, DictData, DictDataInput, DictType, DictTypeInput, Post, PostInput, SortBatchInput, TreeSelectNode,
};

use crate::application::{
    ConfigListFilter, DeptListFilter, DictDataListFilter, DictTypeListFilter, PostListFilter, SystemCache, SystemCursorCodec, SystemError, SystemExportRequest,
    SystemExportSink, SystemRepository, SystemResult, SystemUseCase,
};

use super::{SystemService, dept_scope, export, tree::dept_tree, validation::*};

#[async_trait]
impl<R: SystemRepository, C: SystemCache> SystemUseCase for SystemService<R, C> {
    async fn export(&self, request: SystemExportRequest, sink: &mut dyn SystemExportSink) -> SystemResult<()> {
        export::export(&self.repository, request, sink).await
    }
    async fn page_depts(&self, filter: DeptListFilter) -> SystemResult<CursorPage<Dept>> {
        let filter = sanitize_dept_filter(filter);
        validate_page(filter.page.clone())?;
        SystemCursorCodec::dept(&filter, None)?.decode(&filter.page)?;
        self.repository.page_depts(filter).await
    }

    async fn page_depts_scoped(&self, filter: DeptListFilter, scope: DataScopeFilter) -> SystemResult<CursorPage<Dept>> {
        let filter = sanitize_dept_filter(filter);
        validate_page(filter.page.clone())?;
        SystemCursorCodec::dept(&filter, Some(&scope))?.decode(&filter.page)?;
        self.repository.page_depts_scoped(filter, scope).await
    }

    async fn get_dept(&self, id: &str) -> SystemResult<Dept> {
        self.repository.find_dept(id).await?.ok_or(SystemError::NotFound)
    }

    async fn dept_tree(&self, filter: DeptListFilter, scope: Option<DataScopeFilter>) -> SystemResult<Vec<TreeSelectNode>> {
        dept_scope::scoped_dept_tree(&self.repository, filter, scope).await
    }

    async fn ensure_dept_ids_scoped(&self, ids: Vec<String>, scope: DataScopeFilter) -> SystemResult<()> {
        dept_scope::ensure_dept_ids_scoped(&self.repository, ids, scope).await
    }

    async fn exclude_dept_tree(&self, id: &str) -> SystemResult<Vec<TreeSelectNode>> {
        self.get_dept(id).await?;
        Ok(dept_tree(self.repository.list_depts_excluding(id).await?))
    }

    async fn create_dept(&self, input: DeptInput) -> SystemResult<Dept> {
        reject_duplicate_dept(&self.repository, &input, None).await?;
        self.repository.create_dept(input).await
    }

    async fn replace_dept(&self, id: &str, input: DeptInput) -> SystemResult<Dept> {
        reject_invalid_dept_parent(id, &input)?;
        reject_duplicate_dept(&self.repository, &input, Some(id)).await?;
        if input.status != constants::system::STATUS_NORMAL && self.repository.dept_has_normal_children(id).await? {
            return Err(SystemError::Conflict(localized("errors.system.dept_has_active_children")));
        }
        self.repository.replace_dept(id, input).await
    }

    async fn update_dept_sort(&self, id: &str, order_num: i64) -> SystemResult<Dept> {
        self.repository.update_dept_sort(id, order_num).await
    }

    async fn update_dept_sorts(&self, input: SortBatchInput) -> SystemResult<Vec<Dept>> {
        let mut items = Vec::with_capacity(input.items.len());
        for item in input.items {
            items.push(self.update_dept_sort(&item.id, item.order_num).await?);
        }
        Ok(items)
    }

    async fn delete_dept(&self, id: &str) -> SystemResult<()> {
        reject_dept_delete(&self.repository, id).await?;
        self.repository.delete_dept(id).await
    }

    async fn page_posts(&self, filter: PostListFilter) -> SystemResult<CursorPage<Post>> {
        let filter = sanitize_post_filter(filter);
        validate_page(filter.page.clone())?;
        SystemCursorCodec::post(&filter)?.decode(&filter.page)?;
        self.repository.page_posts(filter).await
    }

    async fn get_post(&self, id: &str) -> SystemResult<Post> {
        self.repository.find_post(id).await?.ok_or(SystemError::NotFound)
    }

    async fn post_options(&self) -> SystemResult<Vec<Post>> {
        self.repository.post_options().await
    }

    async fn create_post(&self, input: PostInput) -> SystemResult<Post> {
        reject_duplicate_post(&self.repository, &input, None).await?;
        self.repository.create_post(input).await
    }

    async fn replace_post(&self, id: &str, input: PostInput) -> SystemResult<Post> {
        reject_duplicate_post(&self.repository, &input, Some(id)).await?;
        self.repository.replace_post(id, input).await
    }

    async fn delete_post(&self, id: &str) -> SystemResult<()> {
        reject_post_delete(&self.repository, id).await?;
        self.repository.delete_post(id).await
    }

    async fn delete_posts(&self, ids: Vec<String>) -> SystemResult<()> {
        reject_empty_ids(&ids)?;
        for id in &ids {
            reject_post_delete(&self.repository, id).await?;
        }
        self.repository.delete_posts(&ids).await
    }

    async fn page_dict_types(&self, filter: DictTypeListFilter) -> SystemResult<CursorPage<DictType>> {
        let filter = sanitize_dict_type_filter(filter);
        validate_page(filter.page.clone())?;
        SystemCursorCodec::dict_type(&filter)?.decode(&filter.page)?;
        self.repository.page_dict_types(filter).await
    }

    async fn get_dict_type(&self, id: &str) -> SystemResult<DictType> {
        self.repository.find_dict_type(id).await?.ok_or(SystemError::NotFound)
    }

    async fn dict_type_options(&self) -> SystemResult<Vec<DictType>> {
        self.repository.dict_type_options().await
    }

    async fn create_dict_type(&self, input: DictTypeInput) -> SystemResult<DictType> {
        reject_duplicate_dict_type(&self.repository, &input, None).await?;
        let item = self.repository.create_dict_type(input).await?;
        self.refresh_dict_cache().await?;
        Ok(item)
    }

    async fn replace_dict_type(&self, id: &str, input: DictTypeInput) -> SystemResult<DictType> {
        reject_duplicate_dict_type(&self.repository, &input, Some(id)).await?;
        let item = self.repository.replace_dict_type(id, input).await?;
        self.refresh_dict_cache().await?;
        Ok(item)
    }

    async fn delete_dict_type(&self, id: &str) -> SystemResult<()> {
        let item = self.get_dict_type(id).await?;
        if self.repository.dict_type_has_data(&item.dict_type).await? {
            return Err(SystemError::Conflict(localized("errors.system.dict_type_has_data")));
        }
        self.repository.delete_dict_type(id).await?;
        self.refresh_dict_cache().await
    }

    async fn delete_dict_types(&self, ids: Vec<String>) -> SystemResult<()> {
        reject_empty_ids(&ids)?;
        for id in &ids {
            let item = self.get_dict_type(id).await?;
            if self.repository.dict_type_has_data(&item.dict_type).await? {
                return Err(SystemError::Conflict(localized("errors.system.dict_type_has_data")));
            }
        }
        self.repository.delete_dict_types(&ids).await?;
        self.refresh_dict_cache().await
    }

    async fn refresh_dict_cache(&self) -> SystemResult<()> {
        self.refresh_dict_cache_after_write().await
    }

    async fn page_dict_data(&self, filter: DictDataListFilter) -> SystemResult<CursorPage<DictData>> {
        let filter = sanitize_dict_data_filter(filter);
        validate_page(filter.page.clone())?;
        SystemCursorCodec::dict_data(&filter)?.decode(&filter.page)?;
        self.repository.page_dict_data(filter).await
    }

    async fn get_dict_data(&self, id: &str) -> SystemResult<DictData> {
        self.repository.find_dict_data(id).await?.ok_or(SystemError::NotFound)
    }

    async fn dict_data_by_type(&self, dict_type: &str) -> SystemResult<Vec<DictData>> {
        if let Some(items) = self.cache.read_dict_data(dict_type).await? {
            return Ok(items);
        }
        let items = self.repository.dict_data_by_type(dict_type).await?;
        self.cache.write_dict_data(dict_type, &items).await?;
        Ok(items)
    }

    async fn create_dict_data(&self, input: DictDataInput) -> SystemResult<DictData> {
        let item = self.repository.create_dict_data(input).await?;
        self.refresh_dict_cache().await?;
        Ok(item)
    }

    async fn replace_dict_data(&self, id: &str, input: DictDataInput) -> SystemResult<DictData> {
        let item = self.repository.replace_dict_data(id, input).await?;
        self.refresh_dict_cache().await?;
        Ok(item)
    }

    async fn delete_dict_data(&self, id: &str) -> SystemResult<()> {
        self.repository.delete_dict_data(id).await?;
        self.refresh_dict_cache().await
    }

    async fn delete_dict_data_batch(&self, ids: Vec<String>) -> SystemResult<()> {
        reject_empty_ids(&ids)?;
        self.repository.delete_dict_data_batch(&ids).await?;
        self.refresh_dict_cache().await
    }

    async fn page_configs(&self, filter: ConfigListFilter) -> SystemResult<CursorPage<ConfigItem>> {
        let filter = sanitize_config_filter(filter);
        validate_page(filter.page.clone())?;
        SystemCursorCodec::config(&filter)?.decode(&filter.page)?;
        self.repository.page_configs(filter).await
    }

    async fn get_config(&self, id: &str) -> SystemResult<ConfigItem> {
        self.repository.find_config(id).await?.ok_or(SystemError::NotFound)
    }

    async fn config_by_key(&self, key: &str) -> SystemResult<String> {
        if let Some(value) = self.cache.read_config(key).await? {
            return Ok(value);
        }
        let value = self.repository.config_by_key(key).await?.ok_or(SystemError::NotFound)?;
        self.cache
            .write_config(&ConfigItem {
                config_id: String::new(),
                config_name: String::new(),
                config_key: key.into(),
                config_value: value.clone(),
                config_type: String::new(),
                public_read: false,
                remark: None,
                create_time: String::new(),
            })
            .await?;
        Ok(value)
    }

    async fn public_configs(&self, keys: Vec<String>) -> SystemResult<BTreeMap<String, String>> {
        let keys = clean_config_keys(keys)?;
        let mut values = BTreeMap::new();
        for key in keys {
            let item = self.repository.find_config_by_key(&key).await?.ok_or(SystemError::NotFound)?;
            if !item.public_read {
                return Err(SystemError::Forbidden(localized_param("errors.system.config_not_public", "key", key)));
            }
            values.insert(item.config_key, item.config_value);
        }
        Ok(values)
    }

    async fn create_config(&self, input: ConfigInput) -> SystemResult<ConfigItem> {
        validate_runtime_config(&input)?;
        reject_sensitive_public_config(&input.config_key, input.public_read)?;
        reject_duplicate_config_key(&self.repository, &input, None).await?;
        let item = self.repository.create_config(input).await?;
        self.cache.write_config(&item).await?;
        Ok(item)
    }

    async fn replace_config(&self, id: &str, input: ConfigInput) -> SystemResult<ConfigItem> {
        let current = self.get_config(id).await?;
        reject_builtin_config_identity_change(&current, &input)?;
        validate_runtime_config(&input)?;
        reject_sensitive_public_config(&input.config_key, input.public_read)?;
        reject_duplicate_config_key(&self.repository, &input, Some(id)).await?;
        let item = self.repository.replace_config(id, input).await?;
        self.refresh_config_cache().await?;
        Ok(item)
    }

    async fn delete_config(&self, id: &str) -> SystemResult<()> {
        reject_builtin_config_delete(&self.get_config(id).await?)?;
        self.repository.delete_config(id).await?;
        self.refresh_config_cache().await
    }

    async fn delete_configs(&self, ids: Vec<String>) -> SystemResult<()> {
        reject_empty_ids(&ids)?;
        for id in &ids {
            reject_builtin_config_delete(&self.get_config(id).await?)?;
        }
        self.repository.delete_configs(&ids).await?;
        self.refresh_config_cache().await
    }

    async fn refresh_config_cache(&self) -> SystemResult<()> {
        self.refresh_config_cache_after_write().await
    }
}
