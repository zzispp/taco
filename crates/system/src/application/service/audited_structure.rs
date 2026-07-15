use audit_contract::AuditOutboxRecord;

use crate::{
    application::{AuditedSystemRepository, SystemCache, SystemError, SystemResult},
    domain::{Dept, DeptInput, Post, PostInput, SortBatchInput},
};

use super::{SystemService, validation::*};

impl<R: AuditedSystemRepository, C: SystemCache> SystemService<R, C> {
    pub(super) async fn create_dept_with_audit_command(&self, input: DeptInput, audit: AuditOutboxRecord) -> SystemResult<Dept> {
        reject_duplicate_dept(&self.repository, &input, None).await?;
        self.repository.create_dept_with_audit(input, &audit).await
    }

    pub(super) async fn replace_dept_with_audit_command(&self, id: &str, input: DeptInput, audit: AuditOutboxRecord) -> SystemResult<Dept> {
        reject_invalid_dept_parent(id, &input)?;
        reject_duplicate_dept(&self.repository, &input, Some(id)).await?;
        if input.status != constants::system::STATUS_NORMAL && self.repository.dept_has_normal_children(id).await? {
            return Err(SystemError::Conflict(localized("errors.system.dept_has_active_children")));
        }
        self.repository.replace_dept_with_audit(id, input, &audit).await
    }

    pub(super) async fn update_dept_sort_with_audit_command(&self, id: &str, order_num: i64, audit: AuditOutboxRecord) -> SystemResult<Dept> {
        self.repository.update_dept_sort_with_audit(id, order_num, &audit).await
    }

    pub(super) async fn update_dept_sorts_with_audit_command(&self, input: SortBatchInput, audit: AuditOutboxRecord) -> SystemResult<Vec<Dept>> {
        self.repository.update_dept_sorts_with_audit(input, &audit).await
    }

    pub(super) async fn delete_dept_with_audit_command(&self, id: &str, audit: AuditOutboxRecord) -> SystemResult<()> {
        reject_dept_delete(&self.repository, id).await?;
        self.repository.delete_dept_with_audit(id, &audit).await
    }

    pub(super) async fn create_post_with_audit_command(&self, input: PostInput, audit: AuditOutboxRecord) -> SystemResult<Post> {
        reject_duplicate_post(&self.repository, &input, None).await?;
        self.repository.create_post_with_audit(input, &audit).await
    }

    pub(super) async fn replace_post_with_audit_command(&self, id: &str, input: PostInput, audit: AuditOutboxRecord) -> SystemResult<Post> {
        reject_duplicate_post(&self.repository, &input, Some(id)).await?;
        self.repository.replace_post_with_audit(id, input, &audit).await
    }

    pub(super) async fn delete_post_with_audit_command(&self, id: &str, audit: AuditOutboxRecord) -> SystemResult<()> {
        reject_post_delete(&self.repository, id).await?;
        self.repository.delete_post_with_audit(id, &audit).await
    }

    pub(super) async fn delete_posts_with_audit_command(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> SystemResult<()> {
        reject_empty_ids(&ids)?;
        for id in &ids {
            reject_post_delete(&self.repository, id).await?;
        }
        self.repository.delete_posts_with_audit(&ids, &audit).await
    }
}
