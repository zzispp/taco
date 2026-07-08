use std::collections::BTreeMap;

use async_trait::async_trait;
use kernel::pagination::Page;
use types::rbac::DataScopeFilter;

use crate::domain::{
    ConfigInput, ConfigItem, Dept, DeptInput, DictData, DictDataInput, DictType, DictTypeInput, Post, PostInput, SortBatchInput, TreeSelectNode,
};

use super::{ConfigListFilter, DeptListFilter, DictDataListFilter, DictTypeListFilter, PostListFilter, SystemResult};

#[async_trait]
pub trait SystemUseCase: Send + Sync + 'static {
    async fn page_depts(&self, filter: DeptListFilter) -> SystemResult<Page<Dept>>;
    async fn page_depts_scoped(&self, filter: DeptListFilter, scope: DataScopeFilter) -> SystemResult<Page<Dept>>;
    async fn get_dept(&self, id: &str) -> SystemResult<Dept>;
    async fn dept_tree(&self, filter: DeptListFilter, scope: Option<DataScopeFilter>) -> SystemResult<Vec<TreeSelectNode>>;
    async fn ensure_dept_ids_scoped(&self, ids: Vec<String>, scope: DataScopeFilter) -> SystemResult<()>;
    async fn exclude_dept_tree(&self, id: &str) -> SystemResult<Vec<TreeSelectNode>>;
    async fn create_dept(&self, input: DeptInput) -> SystemResult<Dept>;
    async fn replace_dept(&self, id: &str, input: DeptInput) -> SystemResult<Dept>;
    async fn update_dept_sort(&self, id: &str, order_num: i64) -> SystemResult<Dept>;
    async fn update_dept_sorts(&self, input: SortBatchInput) -> SystemResult<Vec<Dept>>;
    async fn delete_dept(&self, id: &str) -> SystemResult<()>;
    async fn page_posts(&self, filter: PostListFilter) -> SystemResult<Page<Post>>;
    async fn get_post(&self, id: &str) -> SystemResult<Post>;
    async fn post_options(&self) -> SystemResult<Vec<Post>>;
    async fn create_post(&self, input: PostInput) -> SystemResult<Post>;
    async fn replace_post(&self, id: &str, input: PostInput) -> SystemResult<Post>;
    async fn delete_post(&self, id: &str) -> SystemResult<()>;
    async fn delete_posts(&self, ids: Vec<String>) -> SystemResult<()>;
    async fn page_dict_types(&self, filter: DictTypeListFilter) -> SystemResult<Page<DictType>>;
    async fn get_dict_type(&self, id: &str) -> SystemResult<DictType>;
    async fn dict_type_options(&self) -> SystemResult<Vec<DictType>>;
    async fn create_dict_type(&self, input: DictTypeInput) -> SystemResult<DictType>;
    async fn replace_dict_type(&self, id: &str, input: DictTypeInput) -> SystemResult<DictType>;
    async fn delete_dict_type(&self, id: &str) -> SystemResult<()>;
    async fn delete_dict_types(&self, ids: Vec<String>) -> SystemResult<()>;
    async fn refresh_dict_cache(&self) -> SystemResult<()>;
    async fn page_dict_data(&self, filter: DictDataListFilter) -> SystemResult<Page<DictData>>;
    async fn get_dict_data(&self, id: &str) -> SystemResult<DictData>;
    async fn dict_data_by_type(&self, dict_type: &str) -> SystemResult<Vec<DictData>>;
    async fn create_dict_data(&self, input: DictDataInput) -> SystemResult<DictData>;
    async fn replace_dict_data(&self, id: &str, input: DictDataInput) -> SystemResult<DictData>;
    async fn delete_dict_data(&self, id: &str) -> SystemResult<()>;
    async fn delete_dict_data_batch(&self, ids: Vec<String>) -> SystemResult<()>;
    async fn page_configs(&self, filter: ConfigListFilter) -> SystemResult<Page<ConfigItem>>;
    async fn get_config(&self, id: &str) -> SystemResult<ConfigItem>;
    async fn config_by_key(&self, key: &str) -> SystemResult<String>;
    async fn public_configs(&self, keys: Vec<String>) -> SystemResult<BTreeMap<String, String>>;
    async fn create_config(&self, input: ConfigInput) -> SystemResult<ConfigItem>;
    async fn replace_config(&self, id: &str, input: ConfigInput) -> SystemResult<ConfigItem>;
    async fn delete_config(&self, id: &str) -> SystemResult<()>;
    async fn delete_configs(&self, ids: Vec<String>) -> SystemResult<()>;
    async fn refresh_config_cache(&self) -> SystemResult<()>;
}

#[async_trait]
pub trait SystemRepository: Send + Sync + 'static {
    async fn page_depts(&self, filter: DeptListFilter) -> SystemResult<Page<Dept>>;
    async fn page_depts_scoped(&self, filter: DeptListFilter, scope: DataScopeFilter) -> SystemResult<Page<Dept>>;
    async fn list_depts(&self, filter: DeptListFilter) -> SystemResult<Vec<Dept>>;
    async fn list_depts_scoped(&self, filter: DeptListFilter, scope: DataScopeFilter) -> SystemResult<Vec<Dept>>;
    async fn list_depts_excluding(&self, id: &str) -> SystemResult<Vec<Dept>>;
    async fn find_dept(&self, id: &str) -> SystemResult<Option<Dept>>;
    async fn create_dept(&self, input: DeptInput) -> SystemResult<Dept>;
    async fn replace_dept(&self, id: &str, input: DeptInput) -> SystemResult<Dept>;
    async fn update_dept_sort(&self, id: &str, order_num: i64) -> SystemResult<Dept>;
    async fn delete_dept(&self, id: &str) -> SystemResult<()>;
    async fn dept_has_children(&self, id: &str) -> SystemResult<bool>;
    async fn dept_has_users(&self, id: &str) -> SystemResult<bool>;
    async fn dept_has_normal_children(&self, id: &str) -> SystemResult<bool>;
    async fn page_posts(&self, filter: PostListFilter) -> SystemResult<Page<Post>>;
    async fn find_post(&self, id: &str) -> SystemResult<Option<Post>>;
    async fn post_options(&self) -> SystemResult<Vec<Post>>;
    async fn post_code_exists(&self, code: &str, current_id: Option<&str>) -> SystemResult<bool>;
    async fn post_name_exists(&self, name: &str, current_id: Option<&str>) -> SystemResult<bool>;
    async fn create_post(&self, input: PostInput) -> SystemResult<Post>;
    async fn replace_post(&self, id: &str, input: PostInput) -> SystemResult<Post>;
    async fn delete_post(&self, id: &str) -> SystemResult<()>;
    async fn delete_posts(&self, ids: &[String]) -> SystemResult<()>;
    async fn post_has_users(&self, id: &str) -> SystemResult<bool>;
    async fn page_dict_types(&self, filter: DictTypeListFilter) -> SystemResult<Page<DictType>>;
    async fn list_dict_types(&self, filter: DictTypeListFilter) -> SystemResult<Vec<DictType>>;
    async fn find_dict_type(&self, id: &str) -> SystemResult<Option<DictType>>;
    async fn dict_type_options(&self) -> SystemResult<Vec<DictType>>;
    async fn dict_type_has_data(&self, dict_type: &str) -> SystemResult<bool>;
    async fn create_dict_type(&self, input: DictTypeInput) -> SystemResult<DictType>;
    async fn replace_dict_type(&self, id: &str, input: DictTypeInput) -> SystemResult<DictType>;
    async fn delete_dict_type(&self, id: &str) -> SystemResult<()>;
    async fn delete_dict_types(&self, ids: &[String]) -> SystemResult<()>;
    async fn page_dict_data(&self, filter: DictDataListFilter) -> SystemResult<Page<DictData>>;
    async fn find_dict_data(&self, id: &str) -> SystemResult<Option<DictData>>;
    async fn dict_data_by_type(&self, dict_type: &str) -> SystemResult<Vec<DictData>>;
    async fn create_dict_data(&self, input: DictDataInput) -> SystemResult<DictData>;
    async fn replace_dict_data(&self, id: &str, input: DictDataInput) -> SystemResult<DictData>;
    async fn delete_dict_data(&self, id: &str) -> SystemResult<()>;
    async fn delete_dict_data_batch(&self, ids: &[String]) -> SystemResult<()>;
    async fn page_configs(&self, filter: ConfigListFilter) -> SystemResult<Page<ConfigItem>>;
    async fn list_configs(&self, filter: ConfigListFilter) -> SystemResult<Vec<ConfigItem>>;
    async fn find_config(&self, id: &str) -> SystemResult<Option<ConfigItem>>;
    async fn find_config_by_key(&self, key: &str) -> SystemResult<Option<ConfigItem>>;
    async fn config_by_key(&self, key: &str) -> SystemResult<Option<String>>;
    async fn create_config(&self, input: ConfigInput) -> SystemResult<ConfigItem>;
    async fn replace_config(&self, id: &str, input: ConfigInput) -> SystemResult<ConfigItem>;
    async fn delete_config(&self, id: &str) -> SystemResult<()>;
    async fn delete_configs(&self, ids: &[String]) -> SystemResult<()>;
}

#[async_trait]
pub trait SystemCache: Send + Sync + 'static {
    async fn read_config(&self, key: &str) -> SystemResult<Option<String>>;
    async fn write_config(&self, item: &ConfigItem) -> SystemResult<()>;
    async fn clear_configs(&self) -> SystemResult<()>;
    async fn read_dict_data(&self, dict_type: &str) -> SystemResult<Option<Vec<DictData>>>;
    async fn write_dict_data(&self, dict_type: &str, items: &[DictData]) -> SystemResult<()>;
    async fn clear_dicts(&self) -> SystemResult<()>;
}
