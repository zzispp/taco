use audit_contract::AuditOutboxRecord;

use crate::{
    application::{AuditedSystemRepository, SystemCache, SystemError, SystemResult},
    domain::{ConfigInput, ConfigItem},
};

use super::{SystemService, validation::*};

impl<R: AuditedSystemRepository, C: SystemCache> SystemService<R, C> {
    pub(super) async fn create_config_with_audit_command(&self, input: ConfigInput, audit: AuditOutboxRecord) -> SystemResult<ConfigItem> {
        reject_sensitive_public_config(&input.config_key, input.public_read)?;
        reject_duplicate_config_key(&self.repository, &input, None).await?;
        self.repository.create_config_with_audit(input, &audit).await
    }

    pub(super) async fn replace_config_with_audit_command(&self, id: &str, input: ConfigInput, audit: AuditOutboxRecord) -> SystemResult<ConfigItem> {
        let current = self.repository.find_config(id).await?.ok_or(SystemError::NotFound)?;
        reject_builtin_config_identity_change(&current, &input)?;
        reject_sensitive_public_config(&input.config_key, input.public_read)?;
        reject_duplicate_config_key(&self.repository, &input, Some(id)).await?;
        self.repository.replace_config_with_audit(id, input, &audit).await
    }

    pub(super) async fn delete_config_with_audit_command(&self, id: &str, audit: AuditOutboxRecord) -> SystemResult<()> {
        let current = self.repository.find_config(id).await?.ok_or(SystemError::NotFound)?;
        reject_builtin_config_delete(&current)?;
        self.repository.delete_config_with_audit(id, &audit).await
    }

    pub(super) async fn delete_configs_with_audit_command(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> SystemResult<()> {
        reject_empty_ids(&ids)?;
        for id in &ids {
            let current = self.repository.find_config(id).await?.ok_or(SystemError::NotFound)?;
            reject_builtin_config_delete(&current)?;
        }
        self.repository.delete_configs_with_audit(&ids, &audit).await
    }
}
