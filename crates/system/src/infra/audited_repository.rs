use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;

use crate::{
    application::{AuditedSystemRepository, SystemResult},
    domain::{ConfigInput, ConfigItem, Dept, DeptInput, DictData, DictDataInput, DictType, DictTypeInput, Post, PostInput, SortBatchInput},
};

use super::{StorageSystemRepository, mapping::storage_error};

#[async_trait]
impl AuditedSystemRepository for StorageSystemRepository {
    async fn create_dept_with_audit(&self, input: DeptInput, audit: &AuditOutboxRecord) -> SystemResult<Dept> {
        self.depts.create_with_audit(input, audit).await.map_err(storage_error)
    }

    async fn replace_dept_with_audit(&self, id: &str, input: DeptInput, audit: &AuditOutboxRecord) -> SystemResult<Dept> {
        self.depts.replace_with_audit(id, input, audit).await.map_err(storage_error)
    }

    async fn update_dept_sort_with_audit(&self, id: &str, order_num: i64, audit: &AuditOutboxRecord) -> SystemResult<Dept> {
        self.depts.update_sort_with_audit(id, order_num, audit).await.map_err(storage_error)
    }

    async fn update_dept_sorts_with_audit(&self, input: SortBatchInput, audit: &AuditOutboxRecord) -> SystemResult<Vec<Dept>> {
        self.depts.update_sorts_with_audit(input, audit).await.map_err(storage_error)
    }

    async fn delete_dept_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> SystemResult<()> {
        self.depts.delete_with_audit(id, audit).await.map_err(storage_error)
    }

    async fn create_post_with_audit(&self, input: PostInput, audit: &AuditOutboxRecord) -> SystemResult<Post> {
        self.posts.create_with_audit(input, audit).await.map_err(storage_error)
    }

    async fn replace_post_with_audit(&self, id: &str, input: PostInput, audit: &AuditOutboxRecord) -> SystemResult<Post> {
        self.posts.replace_with_audit(id, input, audit).await.map_err(storage_error)
    }

    async fn delete_post_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> SystemResult<()> {
        self.posts.delete_with_audit(id, audit).await.map_err(storage_error)
    }

    async fn delete_posts_with_audit(&self, ids: &[String], audit: &AuditOutboxRecord) -> SystemResult<()> {
        self.posts.delete_many_with_audit(ids, audit).await.map_err(storage_error)
    }

    async fn create_dict_type_with_audit(&self, input: DictTypeInput, audit: &AuditOutboxRecord) -> SystemResult<DictType> {
        self.dicts.create_type_with_audit(input, audit).await.map_err(storage_error)
    }

    async fn replace_dict_type_with_audit(&self, id: &str, input: DictTypeInput, audit: &AuditOutboxRecord) -> SystemResult<DictType> {
        self.dicts.replace_type_with_audit(id, input, audit).await.map_err(storage_error)
    }

    async fn delete_dict_type_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> SystemResult<()> {
        self.dicts.delete_type_with_audit(id, audit).await.map_err(storage_error)
    }

    async fn delete_dict_types_with_audit(&self, ids: &[String], audit: &AuditOutboxRecord) -> SystemResult<()> {
        self.dicts.delete_types_many_with_audit(ids, audit).await.map_err(storage_error)
    }

    async fn create_dict_data_with_audit(&self, input: DictDataInput, audit: &AuditOutboxRecord) -> SystemResult<DictData> {
        self.dicts.create_data_with_audit(input, audit).await.map_err(storage_error)
    }

    async fn replace_dict_data_with_audit(&self, id: &str, input: DictDataInput, audit: &AuditOutboxRecord) -> SystemResult<DictData> {
        self.dicts.replace_data_with_audit(id, input, audit).await.map_err(storage_error)
    }

    async fn delete_dict_data_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> SystemResult<()> {
        self.dicts.delete_data_with_audit(id, audit).await.map_err(storage_error)
    }

    async fn delete_dict_data_batch_with_audit(&self, ids: &[String], audit: &AuditOutboxRecord) -> SystemResult<()> {
        self.dicts.delete_data_many_with_audit(ids, audit).await.map_err(storage_error)
    }

    async fn create_config_with_audit(&self, input: ConfigInput, audit: &AuditOutboxRecord) -> SystemResult<ConfigItem> {
        self.configs.create_with_audit(input, audit).await.map_err(storage_error)
    }

    async fn replace_config_with_audit(&self, id: &str, input: ConfigInput, audit: &AuditOutboxRecord) -> SystemResult<ConfigItem> {
        self.configs.replace_with_audit(id, input, audit).await.map_err(storage_error)
    }

    async fn delete_config_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> SystemResult<()> {
        self.configs.delete_with_audit(id, audit).await.map_err(storage_error)
    }

    async fn delete_configs_with_audit(&self, ids: &[String], audit: &AuditOutboxRecord) -> SystemResult<()> {
        self.configs.delete_many_with_audit(ids, audit).await.map_err(storage_error)
    }
}
