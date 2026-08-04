use std::collections::BTreeMap;

use async_trait::async_trait;
use kernel::pagination::CursorPage;
use rbac::domain::DataScopeFilter;

use crate::{
    application::{
        ConfigListFilter, DeptListFilter, DictDataListFilter, DictTypeListFilter, PostListFilter, SystemCache, SystemExportRequest, SystemExportSink,
        SystemRepository, SystemResult, SystemUseCase,
    },
    domain::{ConfigInput, ConfigItem, Dept, DeptInput, DictData, DictDataInput, DictType, DictTypeInput, Post, PostInput, SortBatchInput, TreeSelectNode},
};

use super::{SystemService, export};

#[async_trait]
impl<R: SystemRepository, C: SystemCache> SystemUseCase for SystemService<R, C> {
    async fn export(&self, request: SystemExportRequest, sink: &mut dyn SystemExportSink) -> SystemResult<()> {
        export::export(&self.repository, request, sink).await
    }

    async fn page_depts(&self, filter: DeptListFilter) -> SystemResult<CursorPage<Dept>> {
        self.page_depts_impl(filter).await
    }

    async fn page_depts_scoped(&self, filter: DeptListFilter, scope: DataScopeFilter) -> SystemResult<CursorPage<Dept>> {
        self.page_depts_scoped_impl(filter, scope).await
    }

    async fn get_dept(&self, id: &str) -> SystemResult<Dept> {
        self.get_dept_impl(id).await
    }

    async fn dept_tree(&self, filter: DeptListFilter, scope: Option<DataScopeFilter>) -> SystemResult<Vec<TreeSelectNode>> {
        self.dept_tree_impl(filter, scope).await
    }

    async fn ensure_dept_ids_scoped(&self, ids: Vec<String>, scope: DataScopeFilter) -> SystemResult<()> {
        self.ensure_dept_ids_scoped_impl(ids, scope).await
    }

    async fn exclude_dept_tree(&self, id: &str) -> SystemResult<Vec<TreeSelectNode>> {
        self.exclude_dept_tree_impl(id).await
    }

    async fn create_dept(&self, input: DeptInput) -> SystemResult<Dept> {
        self.create_dept_impl(input).await
    }

    async fn replace_dept(&self, id: &str, input: DeptInput) -> SystemResult<Dept> {
        self.replace_dept_impl(id, input).await
    }

    async fn update_dept_sort(&self, id: &str, order_num: i64) -> SystemResult<Dept> {
        self.update_dept_sort_impl(id, order_num).await
    }

    async fn update_dept_sorts(&self, input: SortBatchInput) -> SystemResult<Vec<Dept>> {
        self.update_dept_sorts_impl(input).await
    }

    async fn delete_dept(&self, id: &str) -> SystemResult<()> {
        self.delete_dept_impl(id).await
    }

    async fn page_posts(&self, filter: PostListFilter) -> SystemResult<CursorPage<Post>> {
        self.page_posts_impl(filter).await
    }

    async fn get_post(&self, id: &str) -> SystemResult<Post> {
        self.get_post_impl(id).await
    }

    async fn post_options(&self) -> SystemResult<Vec<Post>> {
        self.post_options_impl().await
    }

    async fn create_post(&self, input: PostInput) -> SystemResult<Post> {
        self.create_post_impl(input).await
    }

    async fn replace_post(&self, id: &str, input: PostInput) -> SystemResult<Post> {
        self.replace_post_impl(id, input).await
    }

    async fn delete_post(&self, id: &str) -> SystemResult<()> {
        self.delete_post_impl(id).await
    }

    async fn delete_posts(&self, ids: Vec<String>) -> SystemResult<()> {
        self.delete_posts_impl(ids).await
    }

    async fn page_dict_types(&self, filter: DictTypeListFilter) -> SystemResult<CursorPage<DictType>> {
        self.page_dict_types_impl(filter).await
    }

    async fn get_dict_type(&self, id: &str) -> SystemResult<DictType> {
        self.get_dict_type_impl(id).await
    }

    async fn dict_type_options(&self) -> SystemResult<Vec<DictType>> {
        self.dict_type_options_impl().await
    }

    async fn create_dict_type(&self, input: DictTypeInput) -> SystemResult<DictType> {
        self.create_dict_type_impl(input).await
    }

    async fn replace_dict_type(&self, id: &str, input: DictTypeInput) -> SystemResult<DictType> {
        self.replace_dict_type_impl(id, input).await
    }

    async fn delete_dict_type(&self, id: &str) -> SystemResult<()> {
        self.delete_dict_type_impl(id).await
    }

    async fn delete_dict_types(&self, ids: Vec<String>) -> SystemResult<()> {
        self.delete_dict_types_impl(ids).await
    }

    async fn refresh_dict_cache(&self) -> SystemResult<()> {
        self.refresh_dict_cache_impl().await
    }

    async fn page_dict_data(&self, filter: DictDataListFilter) -> SystemResult<CursorPage<DictData>> {
        self.page_dict_data_impl(filter).await
    }

    async fn get_dict_data(&self, id: &str) -> SystemResult<DictData> {
        self.get_dict_data_impl(id).await
    }

    async fn dict_data_by_type(&self, dict_type: &str) -> SystemResult<Vec<DictData>> {
        self.dict_data_by_type_impl(dict_type).await
    }

    async fn create_dict_data(&self, input: DictDataInput) -> SystemResult<DictData> {
        self.create_dict_data_impl(input).await
    }

    async fn replace_dict_data(&self, id: &str, input: DictDataInput) -> SystemResult<DictData> {
        self.replace_dict_data_impl(id, input).await
    }

    async fn delete_dict_data(&self, id: &str) -> SystemResult<()> {
        self.delete_dict_data_impl(id).await
    }

    async fn delete_dict_data_batch(&self, ids: Vec<String>) -> SystemResult<()> {
        self.delete_dict_data_batch_impl(ids).await
    }

    async fn page_configs(&self, filter: ConfigListFilter) -> SystemResult<CursorPage<ConfigItem>> {
        self.page_configs_impl(filter).await
    }

    async fn get_config(&self, id: &str) -> SystemResult<ConfigItem> {
        self.get_config_impl(id).await
    }

    async fn config_by_key(&self, key: &str) -> SystemResult<String> {
        self.config_by_key_impl(key).await
    }

    async fn public_configs(&self, keys: Vec<String>) -> SystemResult<BTreeMap<String, String>> {
        self.public_configs_impl(keys).await
    }

    async fn create_config(&self, input: ConfigInput) -> SystemResult<ConfigItem> {
        self.create_config_impl(input).await
    }

    async fn replace_config(&self, id: &str, input: ConfigInput) -> SystemResult<ConfigItem> {
        self.replace_config_impl(id, input).await
    }

    async fn delete_config(&self, id: &str) -> SystemResult<()> {
        self.delete_config_impl(id).await
    }

    async fn delete_configs(&self, ids: Vec<String>) -> SystemResult<()> {
        self.delete_configs_impl(ids).await
    }

    async fn refresh_config_cache(&self) -> SystemResult<()> {
        self.refresh_config_cache_impl().await
    }
}
