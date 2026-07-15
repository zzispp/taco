use audit_contract::AuditOutboxRecord;

use crate::{
    application::{AuditedSystemRepository, SystemCache, SystemError, SystemResult},
    domain::{DictData, DictDataInput, DictType, DictTypeInput},
};

use super::{SystemService, validation::*};

impl<R: AuditedSystemRepository, C: SystemCache> SystemService<R, C> {
    pub(super) async fn create_dict_type_with_audit_command(&self, input: DictTypeInput, audit: AuditOutboxRecord) -> SystemResult<DictType> {
        reject_duplicate_dict_type(&self.repository, &input, None).await?;
        self.repository.create_dict_type_with_audit(input, &audit).await
    }

    pub(super) async fn replace_dict_type_with_audit_command(&self, id: &str, input: DictTypeInput, audit: AuditOutboxRecord) -> SystemResult<DictType> {
        reject_duplicate_dict_type(&self.repository, &input, Some(id)).await?;
        self.repository.replace_dict_type_with_audit(id, input, &audit).await
    }

    pub(super) async fn delete_dict_type_with_audit_command(&self, id: &str, audit: AuditOutboxRecord) -> SystemResult<()> {
        let item = self.repository.find_dict_type(id).await?.ok_or(SystemError::NotFound)?;
        if self.repository.dict_type_has_data(&item.dict_type).await? {
            return Err(SystemError::Conflict(localized("errors.system.dict_type_has_data")));
        }
        self.repository.delete_dict_type_with_audit(id, &audit).await
    }

    pub(super) async fn delete_dict_types_with_audit_command(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> SystemResult<()> {
        reject_empty_ids(&ids)?;
        for id in &ids {
            let item = self.repository.find_dict_type(id).await?.ok_or(SystemError::NotFound)?;
            if self.repository.dict_type_has_data(&item.dict_type).await? {
                return Err(SystemError::Conflict(localized("errors.system.dict_type_has_data")));
            }
        }
        self.repository.delete_dict_types_with_audit(&ids, &audit).await
    }

    pub(super) async fn create_dict_data_with_audit_command(&self, input: DictDataInput, audit: AuditOutboxRecord) -> SystemResult<DictData> {
        self.repository.create_dict_data_with_audit(input, &audit).await
    }

    pub(super) async fn replace_dict_data_with_audit_command(&self, id: &str, input: DictDataInput, audit: AuditOutboxRecord) -> SystemResult<DictData> {
        self.repository.replace_dict_data_with_audit(id, input, &audit).await
    }

    pub(super) async fn delete_dict_data_with_audit_command(&self, id: &str, audit: AuditOutboxRecord) -> SystemResult<()> {
        self.repository.delete_dict_data_with_audit(id, &audit).await
    }

    pub(super) async fn delete_dict_data_batch_with_audit_command(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> SystemResult<()> {
        reject_empty_ids(&ids)?;
        self.repository.delete_dict_data_batch_with_audit(&ids, &audit).await
    }
}
