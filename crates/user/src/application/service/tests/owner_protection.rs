use super::*;

#[tokio::test]
async fn ordinary_user_management_cannot_mutate_the_installation_owner() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "owner", "hashed:secret123"));
    let owner_id = user_id(1);
    repository.mark_installation_owner(owner_id.clone());
    let service = UserService::new(repository, TestPasswordHasher);

    assert_owner_protected(service.replace_user(owner_id.clone(), replace_user("owner", false)).await);
    assert_owner_protected(service.delete_user(owner_id.clone()).await);
    assert_owner_protected(service.delete_users(vec![owner_id.clone()]).await);
    assert_owner_protected(service.reset_password(owner_id.clone(), VALID_PASSWORD.into()).await);
    assert_owner_protected(service.update_status(owner_id.clone(), "1".into()).await);
    assert_owner_protected(service.replace_roles(owner_id, Vec::new()).await);
}

#[tokio::test]
async fn audited_user_management_cannot_mutate_the_installation_owner() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "owner", "hashed:secret123"));
    let owner_id = user_id(1);
    repository.mark_installation_owner(owner_id.clone());
    let service = UserService::new(repository, TestPasswordHasher);

    assert_owner_protected(
        service
            .replace_user_with_audit(owner_id.clone(), replace_user("owner", false), audit("replace"))
            .await,
    );
    assert_owner_protected(service.delete_user_with_audit(owner_id.clone(), audit("delete")).await);
    assert_owner_protected(service.delete_users_with_audit(vec![owner_id.clone()], audit("delete-many")).await);
    assert_owner_protected(service.reset_password_with_audit(owner_id.clone(), VALID_PASSWORD.into(), audit("reset")).await);
    assert_owner_protected(service.update_status_with_audit(owner_id.clone(), "1".into(), audit("status")).await);
    assert_owner_protected(service.replace_roles_with_audit(owner_id, Vec::new(), audit("roles")).await);
}

fn assert_owner_protected<T>(result: AppResult<T>) {
    assert!(matches!(result, Err(AppError::Forbidden(error)) if error.key() == "errors.user.installation_owner_protected"));
}

fn audit(id: &str) -> audit_contract::AuditOutboxRecord {
    audit_contract::AuditOutboxRecord {
        id: id.into(),
        occurred_at: time::OffsetDateTime::UNIX_EPOCH,
        event: audit_contract::AuditOutboxEvent::Operation(audit_contract::OperationAuditEvent {
            title_key: "audit.module.user".into(),
            business_type: audit_contract::BusinessType::Update,
            handler: "user::test".into(),
            request_method: "PUT".into(),
            operator_type: audit_contract::OperatorType::Manage,
            actor: audit_contract::ActorSnapshot::default(),
            operation_url: "/api/system/users".into(),
            operation_ip: "127.0.0.1".into(),
            status: audit_contract::AuditStatus::Success,
            request_id: format!("request-{id}"),
            request_params: "{}".into(),
            response_result: String::new(),
            error_message: String::new(),
            cost_time_ms: 1,
        }),
    }
}
