use audit_contract::{ActorSnapshot, AuditOutboxEvent, AuditOutboxRecord, AuditStatus, BusinessType, OperationAuditEvent, OperatorType};
use time::OffsetDateTime;

use super::*;
use crate::{
    application::{AuditedPasswordChange, UserImportInput, UserImportRow, UserService, UserUseCase},
    domain::UserId,
    test_support::{MemoryUserRepository, TestPasswordHasher, VALID_PASSWORD, new_user, replace_user, stored_user, user_id},
};

#[tokio::test]
async fn audited_user_write_commands_append_exactly_one_record_per_successful_write() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository.clone(), TestPasswordHasher);
    let created = service.create_user_with_audit(new_user("alice"), audit("create")).await.unwrap();

    apply_audited_user_updates(&service, &created.id).await;
    delete_audited_user(&service, created.id).await;

    assert_eq!(
        repository.audit_records().into_iter().map(|record| record.id).collect::<Vec<_>>(),
        vec![
            "create",
            "profile",
            "password",
            "avatar",
            "replace",
            "reset",
            "status",
            "roles",
            "delete",
            "delete-many"
        ]
    );
}

async fn apply_audited_user_updates(service: &UserService<MemoryUserRepository, TestPasswordHasher>, user_id: &UserId) {
    service
        .update_profile_with_audit(user_id.clone(), profile_update("alice-new@example.com", Some("13900000000")), audit("profile"))
        .await
        .unwrap();
    service
        .change_password_with_audit(AuditedPasswordChange {
            user_id: user_id.clone(),
            old_password: VALID_PASSWORD.into(),
            new_password: "newsecret123".into(),
            audit: audit("password"),
        })
        .await
        .unwrap();
    service
        .update_avatar_with_audit(user_id.clone(), "/uploads/avatars/alice.png".into(), audit("avatar"))
        .await
        .unwrap();
    service
        .replace_user_with_audit(user_id.clone(), replace_user("alice", false), audit("replace"))
        .await
        .unwrap();
    service
        .reset_password_with_audit(user_id.clone(), VALID_PASSWORD.into(), audit("reset"))
        .await
        .unwrap();
    service.update_status_with_audit(user_id.clone(), "0".into(), audit("status")).await.unwrap();
    service
        .replace_roles_with_audit(user_id.clone(), vec!["1".into()], audit("roles"))
        .await
        .unwrap();
}

async fn delete_audited_user(service: &UserService<MemoryUserRepository, TestPasswordHasher>, user_id: UserId) {
    service.delete_user_with_audit(user_id.clone(), audit("delete")).await.unwrap();
    service.delete_users_with_audit(vec![user_id], audit("delete-many")).await.unwrap();
}

#[tokio::test]
async fn audited_user_import_commits_the_batch_and_one_record_together() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let report = service
        .import_users_with_audit(
            UserImportInput {
                rows: vec![import_row("bob"), import_row("carol")],
                update_support: false,
            },
            audit("import"),
        )
        .await
        .unwrap();

    assert_eq!(report.success_count, 2);
    assert_eq!(repository.created_records().len(), 2);
    assert!(repository.created_records().iter().all(|record| record.role_ids.is_empty()));
    assert_eq!(repository.created_records()[0].password_hash.as_deref(), Some("hashed:secret123"));
    assert_eq!(
        repository.audit_records().into_iter().map(|record| record.id).collect::<Vec<_>>(),
        vec!["import"]
    );
}

#[tokio::test]
async fn audited_user_import_reports_missing_and_invalid_passwords_before_any_write() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let result = service
        .import_users_with_audit(
            UserImportInput {
                rows: vec![
                    import_row("bob"),
                    import_row_with_password("carol", "short"),
                    import_row_with_password("dave", ""),
                ],
                update_support: false,
            },
            audit("import"),
        )
        .await;

    let Err(AppError::ImportValidation(errors)) = result else {
        panic!("invalid row password must reject the entire import batch");
    };
    assert_eq!(errors.len(), 2);
    assert!(errors.iter().all(|error| error.key() == "errors.validation.length_between"));
    assert!(errors.iter().all(|error| error.params()[0].value() == "password"));
    assert_eq!(repository.created_records(), Vec::new());
    assert_eq!(repository.audit_records(), Vec::new());
}

#[tokio::test]
async fn audited_user_import_does_not_write_when_its_outbox_append_fails() {
    let repository = MemoryUserRepository::default();
    repository.fail_audit_with("outbox unavailable");
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let result = service
        .import_users_with_audit(
            UserImportInput {
                rows: vec![import_row("bob")],
                update_support: false,
            },
            audit("import"),
        )
        .await;

    assert!(matches!(result, Err(AppError::Infrastructure(message)) if message == "outbox unavailable"));
    assert_eq!(repository.created_records(), Vec::new());
    assert_eq!(repository.audit_records(), Vec::new());
}

#[tokio::test]
async fn audited_user_import_propagates_repository_lookup_failures() {
    let repository = MemoryUserRepository::default();
    repository.fail_auth_lookup_with("user lookup unavailable");
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service
        .import_users_with_audit(
            UserImportInput {
                rows: vec![import_row("bob")],
                update_support: false,
            },
            audit("import"),
        )
        .await;

    assert!(matches!(result, Err(AppError::Infrastructure(message)) if message == "user lookup unavailable"));
}

#[tokio::test]
async fn audited_user_import_cannot_replace_the_installation_owner() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "owner", "hashed:secret123"));
    repository.mark_installation_owner(user_id(1));
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service
        .import_users_with_audit(
            UserImportInput {
                rows: vec![import_row("owner")],
                update_support: true,
            },
            audit("import-owner"),
        )
        .await;

    assert!(matches!(result, Err(AppError::Forbidden(error)) if error.key() == "errors.user.installation_owner_protected"));
}

fn import_row(username: &str) -> UserImportRow {
    import_row_with_password(username, VALID_PASSWORD)
}

fn import_row_with_password(username: &str, password: &str) -> UserImportRow {
    UserImportRow {
        dept_id: Some("103".into()),
        username: username.into(),
        password: password.into(),
        nick_name: username.into(),
        email: format!("{username}@example.com"),
        phonenumber: None,
        sex: "2".into(),
        status: "0".into(),
    }
}

fn audit(id: &str) -> AuditOutboxRecord {
    AuditOutboxRecord {
        id: id.into(),
        occurred_at: OffsetDateTime::UNIX_EPOCH,
        event: AuditOutboxEvent::Operation(OperationAuditEvent {
            title_key: "audit.module.user".into(),
            business_type: BusinessType::Update,
            handler: "user::test".into(),
            request_method: "PUT".into(),
            operator_type: OperatorType::Manage,
            actor: ActorSnapshot {
                user_id: Some("1".into()),
                username: "admin".into(),
                department_id: None,
                department_name: String::new(),
            },
            operation_url: "/api/system/users/1".into(),
            operation_ip: "127.0.0.1".into(),
            status: AuditStatus::Success,
            request_id: format!("request-{id}"),
            request_params: "{}".into(),
            response_result: String::new(),
            error_message: String::new(),
            cost_time_ms: 1,
        }),
    }
}
