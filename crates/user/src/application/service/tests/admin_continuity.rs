use super::*;

#[tokio::test]
async fn user_management_cannot_remove_the_last_enabled_admin() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "renamed-admin", "hashed:secret123").with_role_ids(vec!["admin-role"]));
    let admin_id = user_id(1);
    let service = UserService::new(repository, TestPasswordHasher);

    assert_last_admin_required(service.replace_user(admin_id.clone(), replace_user("renamed-admin", false)).await);
    assert_last_admin_required(service.delete_user(admin_id.clone()).await);
    assert_last_admin_required(service.delete_users(vec![admin_id.clone()]).await);
    assert_last_admin_required(service.update_status(admin_id.clone(), "1".into()).await);
    assert_last_admin_required(service.replace_roles(admin_id, Vec::new()).await);
}

#[tokio::test]
async fn audited_user_management_cannot_remove_the_last_enabled_admin() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "renamed-admin", "hashed:secret123").with_role_ids(vec!["admin-role"]));
    let admin_id = user_id(1);
    let service = UserService::new(repository, TestPasswordHasher);

    assert_last_admin_required(
        service
            .replace_user_with_audit(admin_id.clone(), replace_user("renamed-admin", false), audit("replace"))
            .await,
    );
    assert_last_admin_required(service.delete_user_with_audit(admin_id.clone(), audit("delete")).await);
    assert_last_admin_required(service.delete_users_with_audit(vec![admin_id.clone()], audit("delete-many")).await);
    assert_last_admin_required(service.update_status_with_audit(admin_id.clone(), "1".into(), audit("status")).await);
    assert_last_admin_required(service.replace_roles_with_audit(admin_id, Vec::new(), audit("roles")).await);
}

#[tokio::test]
async fn user_management_allows_removing_an_admin_when_another_enabled_admin_remains() {
    let repository = MemoryUserRepository::with_users(vec![
        stored_user(1, "renamed-admin", "hashed:secret123").with_role_ids(vec!["admin-role"]),
        stored_user(2, "other-admin", "hashed:secret123").with_role_ids(vec!["admin-role"]),
    ]);
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    service.delete_user(user_id(1)).await.unwrap();

    assert_eq!(repository.deleted_records(), vec![user_id(1)]);
}

fn assert_last_admin_required<T>(result: AppResult<T>) {
    assert!(matches!(result, Err(AppError::Conflict(error)) if error.key() == "errors.user.last_enabled_admin_required"));
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
