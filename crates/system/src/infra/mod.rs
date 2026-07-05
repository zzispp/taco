mod config;
mod dept;
mod dict;
mod mapping;
mod page;
mod post;
mod record;
mod redis_cache;

pub use redis_cache::RedisSystemCache;

use async_trait::async_trait;
use kernel::pagination::Page;
use storage::Database;

use crate::{
    application::{ConfigListFilter, DeptListFilter, DictDataListFilter, DictTypeListFilter, PostListFilter, SystemRepository, SystemResult},
    domain::{ConfigInput, ConfigItem, Dept, DeptInput, DictData, DictDataInput, DictType, DictTypeInput, Post, PostInput},
};
use types::rbac::DataScopeFilter;

use self::{config::ConfigQueries, dept::DeptQueries, dict::DictQueries, mapping::storage_error, post::PostQueries};

#[derive(Clone)]
pub struct StorageSystemRepository {
    depts: DeptQueries,
    posts: PostQueries,
    dicts: DictQueries,
    configs: ConfigQueries,
}

impl StorageSystemRepository {
    pub fn new(database: Database) -> Self {
        Self {
            depts: DeptQueries::new(database.clone()),
            posts: PostQueries::new(database.clone()),
            dicts: DictQueries::new(database.clone()),
            configs: ConfigQueries::new(database),
        }
    }
}

#[async_trait]
impl SystemRepository for StorageSystemRepository {
    async fn page_depts(&self, filter: DeptListFilter) -> SystemResult<Page<Dept>> {
        self.depts.page(filter).await.map_err(storage_error)
    }

    async fn page_depts_scoped(&self, filter: DeptListFilter, scope: DataScopeFilter) -> SystemResult<Page<Dept>> {
        self.depts.page_scoped(filter, scope).await.map_err(storage_error)
    }

    async fn list_depts(&self, filter: DeptListFilter) -> SystemResult<Vec<Dept>> {
        self.depts.list(filter).await.map_err(storage_error)
    }

    async fn list_depts_scoped(&self, filter: DeptListFilter, scope: DataScopeFilter) -> SystemResult<Vec<Dept>> {
        self.depts.list_scoped(filter, scope).await.map_err(storage_error)
    }

    async fn list_depts_excluding(&self, id: &str) -> SystemResult<Vec<Dept>> {
        self.depts.list_excluding(id).await.map_err(storage_error)
    }

    async fn find_dept(&self, id: &str) -> SystemResult<Option<Dept>> {
        self.depts.find(id).await.map_err(storage_error)
    }

    async fn create_dept(&self, input: DeptInput) -> SystemResult<Dept> {
        self.depts.create(input).await.map_err(storage_error)
    }

    async fn replace_dept(&self, id: &str, input: DeptInput) -> SystemResult<Dept> {
        self.depts.replace(id, input).await.map_err(storage_error)
    }

    async fn update_dept_sort(&self, id: &str, order_num: i64) -> SystemResult<Dept> {
        self.depts.update_sort(id, order_num).await.map_err(storage_error)
    }

    async fn delete_dept(&self, id: &str) -> SystemResult<()> {
        self.depts.delete(id).await.map_err(storage_error)
    }

    async fn dept_has_children(&self, id: &str) -> SystemResult<bool> {
        self.depts.has_children(id).await.map_err(storage_error)
    }

    async fn dept_has_users(&self, id: &str) -> SystemResult<bool> {
        self.depts.has_users(id).await.map_err(storage_error)
    }

    async fn dept_has_normal_children(&self, id: &str) -> SystemResult<bool> {
        self.depts.has_normal_children(id).await.map_err(storage_error)
    }

    async fn page_posts(&self, filter: PostListFilter) -> SystemResult<Page<Post>> {
        self.posts.page(filter).await.map_err(storage_error)
    }

    async fn find_post(&self, id: &str) -> SystemResult<Option<Post>> {
        self.posts.find(id).await.map_err(storage_error)
    }

    async fn post_options(&self) -> SystemResult<Vec<Post>> {
        self.posts.options().await.map_err(storage_error)
    }

    async fn post_code_exists(&self, code: &str, current_id: Option<&str>) -> SystemResult<bool> {
        self.posts.code_exists(code, current_id).await.map_err(storage_error)
    }

    async fn post_name_exists(&self, name: &str, current_id: Option<&str>) -> SystemResult<bool> {
        self.posts.name_exists(name, current_id).await.map_err(storage_error)
    }

    async fn create_post(&self, input: PostInput) -> SystemResult<Post> {
        self.posts.create(input).await.map_err(storage_error)
    }

    async fn replace_post(&self, id: &str, input: PostInput) -> SystemResult<Post> {
        self.posts.replace(id, input).await.map_err(storage_error)
    }

    async fn delete_post(&self, id: &str) -> SystemResult<()> {
        self.posts.delete(id).await.map_err(storage_error)
    }

    async fn delete_posts(&self, ids: &[String]) -> SystemResult<()> {
        self.posts.delete_many(ids).await.map_err(storage_error)
    }

    async fn post_has_users(&self, id: &str) -> SystemResult<bool> {
        self.posts.has_users(id).await.map_err(storage_error)
    }

    async fn page_dict_types(&self, filter: DictTypeListFilter) -> SystemResult<Page<DictType>> {
        self.dicts.page_types(filter).await.map_err(storage_error)
    }

    async fn list_dict_types(&self, filter: DictTypeListFilter) -> SystemResult<Vec<DictType>> {
        self.dicts.list_types(filter).await.map_err(storage_error)
    }

    async fn find_dict_type(&self, id: &str) -> SystemResult<Option<DictType>> {
        self.dicts.find_type(id).await.map_err(storage_error)
    }

    async fn dict_type_options(&self) -> SystemResult<Vec<DictType>> {
        self.dicts.type_options().await.map_err(storage_error)
    }

    async fn dict_type_has_data(&self, dict_type: &str) -> SystemResult<bool> {
        self.dicts.type_has_data(dict_type).await.map_err(storage_error)
    }

    async fn create_dict_type(&self, input: DictTypeInput) -> SystemResult<DictType> {
        self.dicts.create_type(input).await.map_err(storage_error)
    }

    async fn replace_dict_type(&self, id: &str, input: DictTypeInput) -> SystemResult<DictType> {
        self.dicts.replace_type(id, input).await.map_err(storage_error)
    }

    async fn delete_dict_type(&self, id: &str) -> SystemResult<()> {
        self.dicts.delete_type(id).await.map_err(storage_error)
    }

    async fn delete_dict_types(&self, ids: &[String]) -> SystemResult<()> {
        self.dicts.delete_types_many(ids).await.map_err(storage_error)
    }

    async fn page_dict_data(&self, filter: DictDataListFilter) -> SystemResult<Page<DictData>> {
        self.dicts.page_data(filter).await.map_err(storage_error)
    }

    async fn find_dict_data(&self, id: &str) -> SystemResult<Option<DictData>> {
        self.dicts.find_data(id).await.map_err(storage_error)
    }

    async fn dict_data_by_type(&self, dict_type: &str) -> SystemResult<Vec<DictData>> {
        self.dicts.data_by_type(dict_type).await.map_err(storage_error)
    }

    async fn create_dict_data(&self, input: DictDataInput) -> SystemResult<DictData> {
        self.dicts.create_data(input).await.map_err(storage_error)
    }

    async fn replace_dict_data(&self, id: &str, input: DictDataInput) -> SystemResult<DictData> {
        self.dicts.replace_data(id, input).await.map_err(storage_error)
    }

    async fn delete_dict_data(&self, id: &str) -> SystemResult<()> {
        self.dicts.delete_data(id).await.map_err(storage_error)
    }

    async fn delete_dict_data_batch(&self, ids: &[String]) -> SystemResult<()> {
        self.dicts.delete_data_many(ids).await.map_err(storage_error)
    }

    async fn page_configs(&self, filter: ConfigListFilter) -> SystemResult<Page<ConfigItem>> {
        self.configs.page(filter).await.map_err(storage_error)
    }

    async fn list_configs(&self, filter: ConfigListFilter) -> SystemResult<Vec<ConfigItem>> {
        self.configs.list(filter).await.map_err(storage_error)
    }

    async fn find_config(&self, id: &str) -> SystemResult<Option<ConfigItem>> {
        self.configs.find(id).await.map_err(storage_error)
    }

    async fn config_by_key(&self, key: &str) -> SystemResult<Option<String>> {
        self.configs.value_by_key(key).await.map_err(storage_error)
    }

    async fn create_config(&self, input: ConfigInput) -> SystemResult<ConfigItem> {
        self.configs.create(input).await.map_err(storage_error)
    }

    async fn replace_config(&self, id: &str, input: ConfigInput) -> SystemResult<ConfigItem> {
        self.configs.replace(id, input).await.map_err(storage_error)
    }

    async fn delete_config(&self, id: &str) -> SystemResult<()> {
        self.configs.delete(id).await.map_err(storage_error)
    }

    async fn delete_configs(&self, ids: &[String]) -> SystemResult<()> {
        self.configs.delete_many(ids).await.map_err(storage_error)
    }
}
