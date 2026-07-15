use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;

use crate::{
    application::{AuditedSystemRepository, SystemAuditedUseCase, SystemCache, SystemResult},
    domain::{ConfigInput, ConfigItem, Dept, DeptInput, DictData, DictDataInput, DictType, DictTypeInput, Post, PostInput, SortBatchInput},
};

use super::SystemService;

#[async_trait]
impl<R: AuditedSystemRepository, C: SystemCache> SystemAuditedUseCase for SystemService<R, C> {
    async fn create_dept_with_audit(&self, input: DeptInput, audit: AuditOutboxRecord) -> SystemResult<Dept> {
        self.create_dept_with_audit_command(input, audit).await
    }

    async fn replace_dept_with_audit(&self, id: &str, input: DeptInput, audit: AuditOutboxRecord) -> SystemResult<Dept> {
        self.replace_dept_with_audit_command(id, input, audit).await
    }

    async fn update_dept_sort_with_audit(&self, id: &str, order_num: i64, audit: AuditOutboxRecord) -> SystemResult<Dept> {
        self.update_dept_sort_with_audit_command(id, order_num, audit).await
    }

    async fn update_dept_sorts_with_audit(&self, input: SortBatchInput, audit: AuditOutboxRecord) -> SystemResult<Vec<Dept>> {
        self.update_dept_sorts_with_audit_command(input, audit).await
    }

    async fn delete_dept_with_audit(&self, id: &str, audit: AuditOutboxRecord) -> SystemResult<()> {
        self.delete_dept_with_audit_command(id, audit).await
    }

    async fn create_post_with_audit(&self, input: PostInput, audit: AuditOutboxRecord) -> SystemResult<Post> {
        self.create_post_with_audit_command(input, audit).await
    }

    async fn replace_post_with_audit(&self, id: &str, input: PostInput, audit: AuditOutboxRecord) -> SystemResult<Post> {
        self.replace_post_with_audit_command(id, input, audit).await
    }

    async fn delete_post_with_audit(&self, id: &str, audit: AuditOutboxRecord) -> SystemResult<()> {
        self.delete_post_with_audit_command(id, audit).await
    }

    async fn delete_posts_with_audit(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> SystemResult<()> {
        self.delete_posts_with_audit_command(ids, audit).await
    }

    async fn create_dict_type_with_audit(&self, input: DictTypeInput, audit: AuditOutboxRecord) -> SystemResult<DictType> {
        self.create_dict_type_with_audit_command(input, audit).await
    }

    async fn replace_dict_type_with_audit(&self, id: &str, input: DictTypeInput, audit: AuditOutboxRecord) -> SystemResult<DictType> {
        self.replace_dict_type_with_audit_command(id, input, audit).await
    }

    async fn delete_dict_type_with_audit(&self, id: &str, audit: AuditOutboxRecord) -> SystemResult<()> {
        self.delete_dict_type_with_audit_command(id, audit).await
    }

    async fn delete_dict_types_with_audit(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> SystemResult<()> {
        self.delete_dict_types_with_audit_command(ids, audit).await
    }

    async fn create_dict_data_with_audit(&self, input: DictDataInput, audit: AuditOutboxRecord) -> SystemResult<DictData> {
        self.create_dict_data_with_audit_command(input, audit).await
    }

    async fn replace_dict_data_with_audit(&self, id: &str, input: DictDataInput, audit: AuditOutboxRecord) -> SystemResult<DictData> {
        self.replace_dict_data_with_audit_command(id, input, audit).await
    }

    async fn delete_dict_data_with_audit(&self, id: &str, audit: AuditOutboxRecord) -> SystemResult<()> {
        self.delete_dict_data_with_audit_command(id, audit).await
    }

    async fn delete_dict_data_batch_with_audit(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> SystemResult<()> {
        self.delete_dict_data_batch_with_audit_command(ids, audit).await
    }

    async fn create_config_with_audit(&self, input: ConfigInput, audit: AuditOutboxRecord) -> SystemResult<ConfigItem> {
        self.create_config_with_audit_command(input, audit).await
    }

    async fn replace_config_with_audit(&self, id: &str, input: ConfigInput, audit: AuditOutboxRecord) -> SystemResult<ConfigItem> {
        self.replace_config_with_audit_command(id, input, audit).await
    }

    async fn delete_config_with_audit(&self, id: &str, audit: AuditOutboxRecord) -> SystemResult<()> {
        self.delete_config_with_audit_command(id, audit).await
    }

    async fn delete_configs_with_audit(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> SystemResult<()> {
        self.delete_configs_with_audit_command(ids, audit).await
    }
}
