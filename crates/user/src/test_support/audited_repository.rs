use async_trait::async_trait;
use audit_contract::{AuditOutboxEvent, AuditOutboxRecord};

use crate::{
    application::{AppResult, AuditedUserRepository, ReplaceUserRecord, UserImportWrite, UserRepository},
    domain::{ProfileUpdate, User, UserId},
};

use super::MemoryUserRepository;

#[async_trait]
impl AuditedUserRepository for MemoryUserRepository {
    async fn create_with_audit(&self, user: ReplaceUserRecord, audit: &AuditOutboxRecord) -> AppResult<User> {
        self.ensure_audit_available()?;
        let result = <Self as UserRepository>::create(self, user).await?;
        self.record_created_user_audit(audit, &result);
        Ok(result)
    }

    async fn import_with_audit(&self, writes: Vec<UserImportWrite>, audit: &AuditOutboxRecord) -> AppResult<()> {
        self.ensure_audit_available()?;
        let mut state = self.state.lock().unwrap();
        let mut transaction = state.clone();
        for write in writes {
            match write {
                UserImportWrite::Create(record) => create_imported_user(&mut transaction, record),
                UserImportWrite::Replace { id, user } => replace_imported_user(&mut transaction, id, user)?,
            }
        }
        transaction.audits.push(audit.clone());
        *state = transaction;
        Ok(())
    }

    async fn replace_with_audit(&self, id: UserId, user: ReplaceUserRecord, audit: &AuditOutboxRecord) -> AppResult<User> {
        self.ensure_audit_available()?;
        let result = <Self as UserRepository>::replace(self, id, user).await?;
        self.record_audit(audit);
        Ok(result)
    }

    async fn delete_with_audit(&self, id: UserId, audit: &AuditOutboxRecord) -> AppResult<()> {
        self.ensure_audit_available()?;
        <Self as UserRepository>::delete(self, id).await?;
        self.record_audit(audit);
        Ok(())
    }

    async fn delete_many_with_audit(&self, ids: Vec<UserId>, audit: &AuditOutboxRecord) -> AppResult<()> {
        self.ensure_audit_available()?;
        <Self as UserRepository>::delete_many(self, ids).await?;
        self.record_audit(audit);
        Ok(())
    }

    async fn record_login_with_audit(&self, id: UserId, ipaddr: String, audit: &AuditOutboxRecord) -> AppResult<()> {
        self.ensure_audit_available()?;
        <Self as UserRepository>::record_login(self, id, ipaddr).await?;
        self.record_audit(audit);
        Ok(())
    }

    async fn update_password_with_audit(&self, id: UserId, password_hash: String, audit: &AuditOutboxRecord) -> AppResult<()> {
        self.ensure_audit_available()?;
        <Self as UserRepository>::update_password(self, id, password_hash).await?;
        self.record_audit(audit);
        Ok(())
    }

    async fn update_profile_with_audit(&self, id: UserId, profile: ProfileUpdate, audit: &AuditOutboxRecord) -> AppResult<User> {
        self.ensure_audit_available()?;
        let result = <Self as UserRepository>::update_profile(self, id, profile).await?;
        self.record_audit(audit);
        Ok(result)
    }

    async fn update_avatar_with_audit(&self, id: UserId, avatar: String, audit: &AuditOutboxRecord) -> AppResult<User> {
        self.ensure_audit_available()?;
        let result = <Self as UserRepository>::update_avatar(self, id, avatar).await?;
        self.record_audit(audit);
        Ok(result)
    }

    async fn update_status_with_audit(&self, id: UserId, status: String, audit: &AuditOutboxRecord) -> AppResult<User> {
        self.ensure_audit_available()?;
        let result = <Self as UserRepository>::update_status(self, id, status).await?;
        self.record_audit(audit);
        Ok(result)
    }

    async fn replace_roles_with_audit(&self, id: UserId, role_ids: Vec<String>, audit: &AuditOutboxRecord) -> AppResult<User> {
        self.ensure_audit_available()?;
        let result = <Self as UserRepository>::replace_roles(self, id, role_ids).await?;
        self.record_audit(audit);
        Ok(result)
    }
}

impl MemoryUserRepository {
    fn record_audit(&self, audit: &AuditOutboxRecord) {
        self.state.lock().unwrap().audits.push(audit.clone());
    }

    fn record_created_user_audit(&self, audit: &AuditOutboxRecord, user: &User) {
        let mut audit = audit.clone();
        if let AuditOutboxEvent::Security(event) = &mut audit.event {
            event.user_id = Some(user.id.0.clone());
        }
        self.record_audit(&audit);
    }
}

fn create_imported_user(state: &mut super::RepositoryState, record: ReplaceUserRecord) {
    let id = super::next_user_id(state);
    let user = super::user_from_record(id, &record);
    state.users.push(super::StoredUser {
        user,
        password_hash: record.password_hash.clone().unwrap_or_default(),
    });
    state.created.push(record);
}

fn replace_imported_user(state: &mut super::RepositoryState, id: UserId, record: ReplaceUserRecord) -> AppResult<()> {
    super::replace_stored_user(state, &id, &record)?;
    state.replaced.push((id, record));
    Ok(())
}

#[cfg(test)]
mod tests {
    use audit_contract::{ActorSnapshot, AuditOutboxEvent, AuditStatus, BusinessType, OperationAuditEvent, OperatorType};
    use time::OffsetDateTime;

    use super::*;

    #[tokio::test]
    async fn audited_write_preserves_the_record_after_the_business_write_succeeds() {
        let repository = MemoryUserRepository::default();
        let audit = operation_record();

        let user = repository.create_with_audit(user_record(), &audit).await.unwrap();

        assert_eq!(user.username, "audited-user");
        assert_eq!(repository.audit_records(), vec![audit]);
    }

    fn user_record() -> ReplaceUserRecord {
        ReplaceUserRecord {
            username: "audited-user".into(),
            password_hash: Some("hashed:secret123".into()),
            nick_name: "Audited User".into(),
            dept_id: Some("103".into()),
            email: "audited-user@example.com".into(),
            phonenumber: None,
            sex: "2".into(),
            status: "0".into(),
            remark: None,
            role_ids: vec!["1".into()],
            post_ids: vec!["1".into()],
        }
    }

    fn operation_record() -> AuditOutboxRecord {
        AuditOutboxRecord {
            id: "audit-user-create".into(),
            occurred_at: OffsetDateTime::UNIX_EPOCH,
            event: AuditOutboxEvent::Operation(OperationAuditEvent {
                title_key: "audit.module.user".into(),
                business_type: BusinessType::Insert,
                handler: "user::create_user".into(),
                request_method: "POST".into(),
                operator_type: OperatorType::Manage,
                actor: ActorSnapshot {
                    user_id: Some("1".into()),
                    username: "admin".into(),
                    department_id: Some("103".into()),
                    department_name: "Engineering".into(),
                },
                operation_url: "/api/system/users".into(),
                operation_ip: "127.0.0.1".into(),
                status: AuditStatus::Success,
                request_id: "request-user-create".into(),
                request_params: "{}".into(),
                response_result: "{}".into(),
                error_message: String::new(),
                cost_time_ms: 1,
            }),
        }
    }
}
