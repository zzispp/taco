#[allow(dead_code)]
mod support;

use audit_contract::{ActorSnapshot, AuditOutboxEvent, AuditOutboxRecord, AuditStatus, BusinessType, OperationAuditEvent, OperatorType};
use system::application::{SystemAuditedUseCase, SystemService};
use time::OffsetDateTime;

use support::{ConfigInputSeed, MemoryRepository, config_input, dept, dict_type, post_input};

#[tokio::test]
async fn audited_config_creation_records_the_same_successful_command() {
    let repository = MemoryRepository::default();
    let service = SystemService::new(repository.clone());
    let audit = audit_record();

    let item = service
        .create_config_with_audit(config_input(ConfigInputSeed::public("sys.index.modeTheme", "dark")), audit.clone())
        .await
        .unwrap();

    assert_eq!(item.config_key, "sys.index.modeTheme");
    assert_eq!(repository.audit_records(), vec![audit]);
}

#[tokio::test]
async fn audited_dept_sort_records_the_committed_structure_change() {
    let repository = MemoryRepository::default().with_dept(dept("dept-1", "0", "Engineering"));
    let service = SystemService::new(repository.clone());
    let audit = audit_record();

    let updated = service.update_dept_sort_with_audit("dept-1", 9, audit.clone()).await.unwrap();

    assert_eq!(updated.order_num, 9);
    assert_eq!(repository.updated_dept_sorts(), vec![("dept-1".into(), 9)]);
    assert_eq!(repository.audit_records(), vec![audit]);
}

#[tokio::test]
async fn audited_post_creation_records_the_committed_post_change() {
    let repository = MemoryRepository::default();
    let service = SystemService::new(repository.clone());
    let audit = audit_record();

    let post = service.create_post_with_audit(post_input("ceo", "董事长"), audit.clone()).await.unwrap();

    assert_eq!(post.post_id, "1");
    assert_eq!(repository.audit_records(), vec![audit]);
}

#[tokio::test]
async fn audited_dictionary_deletion_records_the_committed_data_change() {
    let repository = MemoryRepository::default().with_dict_type(dict_type("type-1", "sys_user_sex"));
    let service = SystemService::new(repository.clone());
    let audit = audit_record();

    service.delete_dict_type_with_audit("type-1", audit.clone()).await.unwrap();

    assert_eq!(repository.deleted_dict_types(), vec!["type-1"]);
    assert_eq!(repository.audit_records(), vec![audit]);
}

fn audit_record() -> AuditOutboxRecord {
    AuditOutboxRecord {
        id: "019f5f96-f723-72a0-81dd-2502fbba6658".into(),
        occurred_at: OffsetDateTime::UNIX_EPOCH,
        event: AuditOutboxEvent::Operation(OperationAuditEvent {
            title_key: "audit.module.config".into(),
            business_type: BusinessType::Insert,
            handler: "system::create_config".into(),
            request_method: "POST".into(),
            operator_type: OperatorType::Manage,
            actor: ActorSnapshot::default(),
            operation_url: "/api/system/configs".into(),
            operation_ip: "198.51.100.10".into(),
            status: AuditStatus::Success,
            request_id: "request-1".into(),
            request_params: String::new(),
            response_result: String::new(),
            error_message: String::new(),
            cost_time_ms: 0,
        }),
    }
}
