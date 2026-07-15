use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;

use crate::{
    application::{SystemRepository, SystemResult},
    domain::{ConfigInput, ConfigItem, Dept, DeptInput, DictData, DictDataInput, DictType, DictTypeInput, Post, PostInput, SortBatchInput},
};

/// Persists a system-management state change and its immutable operation audit
/// record in one PostgreSQL transaction.
#[async_trait]
pub trait AuditedSystemRepository: SystemRepository {
    async fn create_dept_with_audit(&self, input: DeptInput, audit: &AuditOutboxRecord) -> SystemResult<Dept>;
    async fn replace_dept_with_audit(&self, id: &str, input: DeptInput, audit: &AuditOutboxRecord) -> SystemResult<Dept>;
    async fn update_dept_sort_with_audit(&self, id: &str, order_num: i64, audit: &AuditOutboxRecord) -> SystemResult<Dept>;
    async fn update_dept_sorts_with_audit(&self, input: SortBatchInput, audit: &AuditOutboxRecord) -> SystemResult<Vec<Dept>>;
    async fn delete_dept_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> SystemResult<()>;
    async fn create_post_with_audit(&self, input: PostInput, audit: &AuditOutboxRecord) -> SystemResult<Post>;
    async fn replace_post_with_audit(&self, id: &str, input: PostInput, audit: &AuditOutboxRecord) -> SystemResult<Post>;
    async fn delete_post_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> SystemResult<()>;
    async fn delete_posts_with_audit(&self, ids: &[String], audit: &AuditOutboxRecord) -> SystemResult<()>;
    async fn create_dict_type_with_audit(&self, input: DictTypeInput, audit: &AuditOutboxRecord) -> SystemResult<DictType>;
    async fn replace_dict_type_with_audit(&self, id: &str, input: DictTypeInput, audit: &AuditOutboxRecord) -> SystemResult<DictType>;
    async fn delete_dict_type_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> SystemResult<()>;
    async fn delete_dict_types_with_audit(&self, ids: &[String], audit: &AuditOutboxRecord) -> SystemResult<()>;
    async fn create_dict_data_with_audit(&self, input: DictDataInput, audit: &AuditOutboxRecord) -> SystemResult<DictData>;
    async fn replace_dict_data_with_audit(&self, id: &str, input: DictDataInput, audit: &AuditOutboxRecord) -> SystemResult<DictData>;
    async fn delete_dict_data_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> SystemResult<()>;
    async fn delete_dict_data_batch_with_audit(&self, ids: &[String], audit: &AuditOutboxRecord) -> SystemResult<()>;
    async fn create_config_with_audit(&self, input: ConfigInput, audit: &AuditOutboxRecord) -> SystemResult<ConfigItem>;
    async fn replace_config_with_audit(&self, id: &str, input: ConfigInput, audit: &AuditOutboxRecord) -> SystemResult<ConfigItem>;
    async fn delete_config_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> SystemResult<()>;
    async fn delete_configs_with_audit(&self, ids: &[String], audit: &AuditOutboxRecord) -> SystemResult<()>;
}

/// Management write use cases that require a transactional operation-audit
/// record. They return after the database transaction commits; callers must
/// mark that record as persisted before performing external cache work.
#[async_trait]
pub trait SystemAuditedUseCase: Send + Sync + 'static {
    async fn create_dept_with_audit(&self, input: DeptInput, audit: AuditOutboxRecord) -> SystemResult<Dept>;
    async fn replace_dept_with_audit(&self, id: &str, input: DeptInput, audit: AuditOutboxRecord) -> SystemResult<Dept>;
    async fn update_dept_sort_with_audit(&self, id: &str, order_num: i64, audit: AuditOutboxRecord) -> SystemResult<Dept>;
    async fn update_dept_sorts_with_audit(&self, input: SortBatchInput, audit: AuditOutboxRecord) -> SystemResult<Vec<Dept>>;
    async fn delete_dept_with_audit(&self, id: &str, audit: AuditOutboxRecord) -> SystemResult<()>;
    async fn create_post_with_audit(&self, input: PostInput, audit: AuditOutboxRecord) -> SystemResult<Post>;
    async fn replace_post_with_audit(&self, id: &str, input: PostInput, audit: AuditOutboxRecord) -> SystemResult<Post>;
    async fn delete_post_with_audit(&self, id: &str, audit: AuditOutboxRecord) -> SystemResult<()>;
    async fn delete_posts_with_audit(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> SystemResult<()>;
    async fn create_dict_type_with_audit(&self, input: DictTypeInput, audit: AuditOutboxRecord) -> SystemResult<DictType>;
    async fn replace_dict_type_with_audit(&self, id: &str, input: DictTypeInput, audit: AuditOutboxRecord) -> SystemResult<DictType>;
    async fn delete_dict_type_with_audit(&self, id: &str, audit: AuditOutboxRecord) -> SystemResult<()>;
    async fn delete_dict_types_with_audit(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> SystemResult<()>;
    async fn create_dict_data_with_audit(&self, input: DictDataInput, audit: AuditOutboxRecord) -> SystemResult<DictData>;
    async fn replace_dict_data_with_audit(&self, id: &str, input: DictDataInput, audit: AuditOutboxRecord) -> SystemResult<DictData>;
    async fn delete_dict_data_with_audit(&self, id: &str, audit: AuditOutboxRecord) -> SystemResult<()>;
    async fn delete_dict_data_batch_with_audit(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> SystemResult<()>;
    async fn create_config_with_audit(&self, input: ConfigInput, audit: AuditOutboxRecord) -> SystemResult<ConfigItem>;
    async fn replace_config_with_audit(&self, id: &str, input: ConfigInput, audit: AuditOutboxRecord) -> SystemResult<ConfigItem>;
    async fn delete_config_with_audit(&self, id: &str, audit: AuditOutboxRecord) -> SystemResult<()>;
    async fn delete_configs_with_audit(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> SystemResult<()>;
}
