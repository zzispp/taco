use std::collections::BTreeMap;

use audit::{
    application::{AuditCursorQuery, AuditError, AuditRepository},
    domain::{
        AuditLocation, AuditStatus, BusinessType, LoginEventType, LoginLogFilter, LoginSortField, NewLoginLog, NewOperationLog, OperationLogDetail,
        OperationLogFilter, OperationLogSummary, OperationSortField, OperatorType, SortDirection,
    },
    infra::StorageAuditRepository,
};
use kernel::pagination::CursorDirection;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use super::{TestDatabase, up};

#[tokio::test]
async fn audit_repository_combines_filters_paginates_sorts_and_keeps_details_separate() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    let repository = StorageAuditRepository::new(storage::Database::new(database.pool().clone()));
    repository.insert_operation(operation(OperationFixture::user_insert())).await.unwrap();
    repository.insert_operation(operation(OperationFixture::user_delete())).await.unwrap();
    repository.insert_operation(operation(OperationFixture::role_insert())).await.unwrap();

    let filter = OperationLogFilter {
        title: Some("audit.module.user".into()),
        business_types: vec![BusinessType::Insert],
        status: Some(AuditStatus::Success),
        operator_name: Some("ali".into()),
        operation_ip: Some("198.51".into()),
        begin_time: Some(timestamp("2026-07-13T09:00:00Z")),
        end_time: Some(timestamp("2026-07-13T11:00:00Z")),
        sort_field: OperationSortField::CostTime,
        sort_direction: SortDirection::Desc,
        ..OperationLogFilter::default()
    };
    let batch = repository.page_operations(filter, cursor_request(20)).await.unwrap();

    assert_eq!(batch.items.iter().map(|item| item.id.as_str()).collect::<Vec<_>>(), ["oper-a"]);
    let detail = repository.find_operation("oper-a").await.unwrap().unwrap();
    assert_eq!(detail.request_params, r#"{"name":"alice"}"#);
    assert_eq!(detail.response_result, r#"{"ok":true}"#);
    assert_eq!(detail.error_message, "");
    assert_eq!(detail.summary.operation_location, AuditLocation::Resolved("Test".into()));

    database.drop().await;
}

#[tokio::test]
async fn audit_repository_deletes_exact_batches_and_clears_operations() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    let repository = StorageAuditRepository::new(storage::Database::new(database.pool().clone()));
    repository.insert_operation(operation(OperationFixture::user_insert())).await.unwrap();
    repository.insert_operation(operation(OperationFixture::user_delete())).await.unwrap();

    let error = repository.delete_operations(&["oper-a".into(), "missing".into()]).await.unwrap_err();
    assert!(matches!(error, AuditError::NotFound));
    assert!(repository.find_operation("oper-a").await.unwrap().is_some());

    repository.delete_operations(&["oper-a".into(), "oper-b".into()]).await.unwrap();
    assert!(repository.find_operation("oper-a").await.unwrap().is_none());
    repository.insert_operation(operation(OperationFixture::role_insert())).await.unwrap();
    repository.clear_operations().await.unwrap();
    assert_eq!(
        repository
            .page_operations(OperationLogFilter::default(), cursor_request(20))
            .await
            .unwrap()
            .items,
        Vec::new()
    );

    database.drop().await;
}

#[tokio::test]
async fn login_repository_filters_sorts_deletes_and_clears() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    let repository = StorageAuditRepository::new(storage::Database::new(database.pool().clone()));
    repository.insert_login(login(LoginFixture::alice_failure())).await.unwrap();
    repository.insert_login(login(LoginFixture::alice_success())).await.unwrap();
    repository.insert_login(login(LoginFixture::bob_logout())).await.unwrap();

    let filter = LoginLogFilter {
        username: Some("ali".into()),
        ip_address: Some("203.0.113".into()),
        status: Some(AuditStatus::Success),
        begin_time: Some(timestamp("2026-07-13T00:00:00Z")),
        end_time: Some(timestamp("2026-07-13T23:59:59Z")),
        sort_field: LoginSortField::LoginTime,
        sort_direction: SortDirection::Desc,
        ..LoginLogFilter::default()
    };
    let batch = repository.page_logins(filter, cursor_request(1)).await.unwrap();
    assert_eq!(batch.items.len(), 1);
    assert_eq!(batch.items[0].event_type, LoginEventType::LoginSuccess);
    assert_eq!(batch.items[0].login_location, AuditLocation::Resolved("Test".into()));

    let id = batch.items[0].id.clone();
    let error = repository.delete_logins(&[id.clone(), "missing".into()]).await.unwrap_err();
    assert!(matches!(error, AuditError::NotFound));
    repository.delete_logins(&[id]).await.unwrap();
    repository.clear_logins().await.unwrap();
    assert_eq!(
        repository.page_logins(LoginLogFilter::default(), cursor_request(20)).await.unwrap().items,
        Vec::new()
    );

    database.drop().await;
}

#[tokio::test]
async fn login_repository_executes_every_non_default_sort_field() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    let repository = StorageAuditRepository::new(storage::Database::new(database.pool().clone()));
    repository.insert_login(login(LoginFixture::alice_failure())).await.unwrap();
    repository.insert_login(login(LoginFixture::alice_success())).await.unwrap();
    repository.insert_login(login(LoginFixture::bob_logout())).await.unwrap();

    for sort_field in [LoginSortField::Username, LoginSortField::IpAddress, LoginSortField::Status] {
        let filter = LoginLogFilter {
            sort_field,
            sort_direction: SortDirection::Asc,
            ..LoginLogFilter::default()
        };
        let batch = repository.page_logins(filter, cursor_request(20)).await.unwrap();
        assert_eq!(batch.items.len(), 3);
    }

    database.drop().await;
}

struct OperationFixture {
    id: &'static str,
    title: &'static str,
    business_type: BusinessType,
    status: AuditStatus,
    operator: &'static str,
    ip: &'static str,
    time: &'static str,
    cost: i64,
}

impl OperationFixture {
    fn user_insert() -> Self {
        Self {
            id: "oper-a",
            title: "audit.module.user",
            business_type: BusinessType::Insert,
            status: AuditStatus::Success,
            operator: "alice",
            ip: "198.51.100.8",
            time: "2026-07-13T10:00:00Z",
            cost: 30,
        }
    }

    fn user_delete() -> Self {
        Self {
            id: "oper-b",
            title: "audit.module.user",
            business_type: BusinessType::Delete,
            status: AuditStatus::Failure,
            operator: "bob",
            ip: "203.0.113.9",
            time: "2026-07-13T11:00:00Z",
            cost: 50,
        }
    }

    fn role_insert() -> Self {
        Self {
            id: "oper-c",
            title: "audit.module.role",
            business_type: BusinessType::Insert,
            status: AuditStatus::Success,
            operator: "alice",
            ip: "198.51.100.8",
            time: "2026-07-13T12:00:00Z",
            cost: 10,
        }
    }
}

fn operation(input: OperationFixture) -> NewOperationLog {
    let summary = OperationLogSummary {
        id: input.id.into(),
        title_key: input.title.into(),
        business_type: input.business_type,
        handler: "test::handler".into(),
        request_method: "POST".into(),
        operator_type: OperatorType::Manage,
        operator_name: input.operator.into(),
        department_name: "Platform".into(),
        operation_url: "/api/test".into(),
        operation_ip: input.ip.into(),
        operation_location: AuditLocation::Resolved("Test".into()),
        status: input.status,
        operation_time: timestamp(input.time),
        cost_time_ms: input.cost,
    };
    NewOperationLog {
        detail: OperationLogDetail {
            summary,
            request_id: "request-1".into(),
            operator_id: Some("user-1".into()),
            department_id: Some("dept-1".into()),
            request_params: r#"{"name":"alice"}"#.into(),
            response_result: r#"{"ok":true}"#.into(),
            error_message: String::new(),
        },
    }
}

struct LoginFixture {
    username: &'static str,
    status: AuditStatus,
    event_type: LoginEventType,
    time: &'static str,
}

impl LoginFixture {
    fn alice_failure() -> Self {
        Self {
            username: "alice",
            status: AuditStatus::Failure,
            event_type: LoginEventType::LoginFailure,
            time: "2026-07-13T10:00:00Z",
        }
    }

    fn alice_success() -> Self {
        Self {
            username: "alice",
            status: AuditStatus::Success,
            event_type: LoginEventType::LoginSuccess,
            time: "2026-07-13T11:00:00Z",
        }
    }

    fn bob_logout() -> Self {
        Self {
            username: "bob",
            status: AuditStatus::Success,
            event_type: LoginEventType::LogoutSuccess,
            time: "2026-07-13T12:00:00Z",
        }
    }
}

fn login(input: LoginFixture) -> NewLoginLog {
    NewLoginLog {
        request_id: "request-login".into(),
        route: "/api/auth/sign-in".into(),
        user_id: Some("user-1".into()),
        username: input.username.into(),
        ip_address: "203.0.113.8".into(),
        login_location: AuditLocation::Resolved("Test".into()),
        browser: "Chrome".into(),
        os: "macOS".into(),
        status: input.status,
        event_type: input.event_type,
        message_key: "messages.user.login_success".into(),
        message_params: BTreeMap::new(),
        login_time: timestamp(input.time),
    }
}

fn cursor_request<B>(limit: u64) -> AuditCursorQuery<B> {
    AuditCursorQuery {
        limit,
        direction: CursorDirection::Next,
        boundary: None,
        snapshot: None,
    }
}

fn timestamp(value: &str) -> OffsetDateTime {
    OffsetDateTime::parse(value, &Rfc3339).unwrap()
}
