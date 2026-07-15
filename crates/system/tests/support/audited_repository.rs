use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;
use system::{
    application::{AuditedSystemRepository, SystemRepository},
    domain::{ConfigInput, ConfigItem, Dept, DeptInput, DictData, DictDataInput, DictType, DictTypeInput, Post, PostInput, SortBatchInput},
};

use super::MemoryRepository;

#[async_trait]
impl AuditedSystemRepository for MemoryRepository {
    async fn create_dept_with_audit(&self, input: DeptInput, audit: &AuditOutboxRecord) -> system::application::SystemResult<Dept> {
        let item = self.create_dept(input).await?;
        self.record_audit(audit);
        Ok(item)
    }

    async fn replace_dept_with_audit(&self, id: &str, input: DeptInput, audit: &AuditOutboxRecord) -> system::application::SystemResult<Dept> {
        let item = self.replace_dept(id, input).await?;
        self.record_audit(audit);
        Ok(item)
    }

    async fn update_dept_sort_with_audit(&self, id: &str, order_num: i64, audit: &AuditOutboxRecord) -> system::application::SystemResult<Dept> {
        let item = self.update_dept_sort(id, order_num).await?;
        self.record_audit(audit);
        Ok(item)
    }

    async fn update_dept_sorts_with_audit(&self, input: SortBatchInput, audit: &AuditOutboxRecord) -> system::application::SystemResult<Vec<Dept>> {
        let mut departments = Vec::with_capacity(input.items.len());
        for item in input.items {
            departments.push(self.update_dept_sort(&item.id, item.order_num).await?);
        }
        self.record_audit(audit);
        Ok(departments)
    }

    async fn delete_dept_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> system::application::SystemResult<()> {
        self.delete_dept(id).await?;
        self.record_audit(audit);
        Ok(())
    }

    async fn create_post_with_audit(&self, input: PostInput, audit: &AuditOutboxRecord) -> system::application::SystemResult<Post> {
        let item = self.create_post(input).await?;
        self.record_audit(audit);
        Ok(item)
    }

    async fn replace_post_with_audit(&self, id: &str, input: PostInput, audit: &AuditOutboxRecord) -> system::application::SystemResult<Post> {
        let item = self.replace_post(id, input).await?;
        self.record_audit(audit);
        Ok(item)
    }

    async fn delete_post_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> system::application::SystemResult<()> {
        self.delete_post(id).await?;
        self.record_audit(audit);
        Ok(())
    }

    async fn delete_posts_with_audit(&self, ids: &[String], audit: &AuditOutboxRecord) -> system::application::SystemResult<()> {
        self.delete_posts(ids).await?;
        self.record_audit(audit);
        Ok(())
    }

    async fn create_dict_type_with_audit(&self, input: DictTypeInput, audit: &AuditOutboxRecord) -> system::application::SystemResult<DictType> {
        let item = self.create_dict_type(input).await?;
        self.record_audit(audit);
        Ok(item)
    }

    async fn replace_dict_type_with_audit(&self, id: &str, input: DictTypeInput, audit: &AuditOutboxRecord) -> system::application::SystemResult<DictType> {
        let item = self.replace_dict_type(id, input).await?;
        self.record_audit(audit);
        Ok(item)
    }

    async fn delete_dict_type_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> system::application::SystemResult<()> {
        self.delete_dict_type(id).await?;
        self.record_audit(audit);
        Ok(())
    }

    async fn delete_dict_types_with_audit(&self, ids: &[String], audit: &AuditOutboxRecord) -> system::application::SystemResult<()> {
        self.delete_dict_types(ids).await?;
        self.record_audit(audit);
        Ok(())
    }

    async fn create_dict_data_with_audit(&self, input: DictDataInput, audit: &AuditOutboxRecord) -> system::application::SystemResult<DictData> {
        let item = self.create_dict_data(input).await?;
        self.record_audit(audit);
        Ok(item)
    }

    async fn replace_dict_data_with_audit(&self, id: &str, input: DictDataInput, audit: &AuditOutboxRecord) -> system::application::SystemResult<DictData> {
        let item = self.replace_dict_data(id, input).await?;
        self.record_audit(audit);
        Ok(item)
    }

    async fn delete_dict_data_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> system::application::SystemResult<()> {
        self.delete_dict_data(id).await?;
        self.record_audit(audit);
        Ok(())
    }

    async fn delete_dict_data_batch_with_audit(&self, ids: &[String], audit: &AuditOutboxRecord) -> system::application::SystemResult<()> {
        self.delete_dict_data_batch(ids).await?;
        self.record_audit(audit);
        Ok(())
    }

    async fn create_config_with_audit(&self, input: ConfigInput, audit: &AuditOutboxRecord) -> system::application::SystemResult<ConfigItem> {
        let item = self.create_config(input).await?;
        self.record_audit(audit);
        Ok(item)
    }

    async fn replace_config_with_audit(&self, id: &str, input: ConfigInput, audit: &AuditOutboxRecord) -> system::application::SystemResult<ConfigItem> {
        let item = self.replace_config(id, input).await?;
        self.record_audit(audit);
        Ok(item)
    }

    async fn delete_config_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> system::application::SystemResult<()> {
        self.delete_config(id).await?;
        self.record_audit(audit);
        Ok(())
    }

    async fn delete_configs_with_audit(&self, ids: &[String], audit: &AuditOutboxRecord) -> system::application::SystemResult<()> {
        self.delete_configs(ids).await?;
        self.record_audit(audit);
        Ok(())
    }
}
