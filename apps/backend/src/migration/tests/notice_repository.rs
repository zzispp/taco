use std::time::Duration;

use kernel::pagination::PageRequest;
use sqlx::{query, query_scalar};
use storage::Database;
use system::notice::{Notice, NoticeInput, NoticeListFilter, NoticeReaderFilter, NoticeService, NoticeUseCase, StorageNoticeRepository};

use super::{TestDatabase, fresh};

const NORMAL_STATUS: &str = "0";
const CLOSED_STATUS: &str = "1";
const NOTICE_TYPE: &str = "1";
const ANNOUNCEMENT_TYPE: &str = "2";
const ADMIN_USER_ID: &str = "1";
const TACO_USER_ID: &str = "2";
const PAGE_SIZE: u64 = 20;
const LOCK_SETTLE_DELAY: Duration = Duration::from_millis(50);

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn notice_repository_supports_crud_filters_and_cascade_delete() {
    let database = TestDatabase::create().await;
    fresh(database.pool()).await.unwrap();
    let service = notice_service(database.pool());

    let notice = service.create_notice(input("Alpha", NOTICE_TYPE, NORMAL_STATUS), "admin".into()).await.unwrap();
    let announcement = service
        .create_notice(input("Beta", ANNOUNCEMENT_TYPE, NORMAL_STATUS), "taco".into())
        .await
        .unwrap();
    let third = service.create_notice(input("Gamma", NOTICE_TYPE, NORMAL_STATUS), "admin".into()).await.unwrap();
    let page = service
        .page_notices(list_filter(Some("Alpha"), Some("admin"), Some(NOTICE_TYPE)))
        .await
        .unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.items[0].notice_id, notice.notice_id);

    let updated = service
        .replace_notice(&notice.notice_id, input("Alpha updated", ANNOUNCEMENT_TYPE, NORMAL_STATUS), "taco".into())
        .await
        .unwrap();
    assert_eq!(updated.notice_title, "Alpha updated");
    assert_eq!(updated.update_by.as_deref(), Some("taco"));

    service.mark_read(&notice.notice_id, TACO_USER_ID).await.unwrap();
    assert_eq!(read_count(database.pool(), &notice.notice_id).await, 1);
    service.delete_notice(&notice.notice_id).await.unwrap();
    assert_eq!(read_count(database.pool(), &notice.notice_id).await, 0);
    assert!(service.delete_notices(vec![announcement.notice_id.clone(), "missing".into()]).await.is_err());
    assert!(service.get_notice(&announcement.notice_id, true).await.is_ok());
    service.delete_notices(vec![announcement.notice_id.clone(), third.notice_id]).await.unwrap();
    assert!(service.get_notice(&announcement.notice_id, true).await.is_err());

    database.drop().await;
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn notice_repository_tracks_top_unread_and_readers() {
    let database = TestDatabase::create().await;
    fresh(database.pool()).await.unwrap();
    let service = notice_service(database.pool());
    let notices = create_normal_notices(&service, 7).await;
    let closed = service
        .create_notice(input("Closed", NOTICE_TYPE, CLOSED_STATUS), "admin".into())
        .await
        .unwrap();
    assert!(service.get_notice(&closed.notice_id, false).await.is_err());
    assert_eq!(service.get_notice(&closed.notice_id, true).await.unwrap().notice_id, closed.notice_id);

    let latest_id = &notices.last().unwrap().notice_id;
    let initial = service.top_notices(TACO_USER_ID).await.unwrap();
    assert_eq!(initial.items.len(), 5);
    assert_eq!(initial.unread_count, 7);
    assert!(initial.items.iter().all(|notice| notice.notice_id != closed.notice_id));

    service.mark_read(latest_id, TACO_USER_ID).await.unwrap();
    service.mark_read(latest_id, TACO_USER_ID).await.unwrap();
    service.mark_read(latest_id, ADMIN_USER_ID).await.unwrap();
    assert_eq!(read_count(database.pool(), latest_id).await, 2);
    let readers = service.page_readers(latest_id, reader_filter("admin")).await.unwrap();
    assert_eq!(readers.total, 1);
    assert_eq!(readers.items[0].user_name, "admin");

    service.mark_all_read(TACO_USER_ID).await.unwrap();
    let completed = service.top_notices(TACO_USER_ID).await.unwrap();
    assert_eq!(completed.unread_count, 0);
    assert!(completed.items.iter().all(|notice| notice.is_read));

    database.drop().await;
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn mark_read_rechecks_status_after_concurrent_close() {
    let database = TestDatabase::create().await;
    fresh(database.pool()).await.unwrap();
    let notice = notice_service(database.pool())
        .create_notice(input("Race", NOTICE_TYPE, NORMAL_STATUS), "admin".into())
        .await
        .unwrap();
    let mut transaction = database.pool().begin().await.unwrap();
    query("SELECT notice_id FROM sys_notice WHERE notice_id=$1 FOR UPDATE")
        .bind(&notice.notice_id)
        .fetch_one(&mut *transaction)
        .await
        .unwrap();

    let pool = database.pool().clone();
    let notice_id = notice.notice_id.clone();
    let read = tokio::spawn(async move { notice_service(&pool).mark_read(&notice_id, TACO_USER_ID).await });
    tokio::time::sleep(LOCK_SETTLE_DELAY).await;
    assert!(!read.is_finished(), "mark read should wait for the notice row lock");
    query("UPDATE sys_notice SET status=$2 WHERE notice_id=$1")
        .bind(&notice.notice_id)
        .bind(CLOSED_STATUS)
        .execute(&mut *transaction)
        .await
        .unwrap();
    transaction.commit().await.unwrap();

    assert!(read.await.unwrap().is_err());
    assert_eq!(read_count(database.pool(), &notice.notice_id).await, 0);
    database.drop().await;
}

fn notice_service(pool: &sqlx::PgPool) -> NoticeService<StorageNoticeRepository> {
    NoticeService::new(StorageNoticeRepository::new(Database::new(pool.clone())))
}

async fn create_normal_notices(service: &impl NoticeUseCase, count: usize) -> Vec<Notice> {
    let mut notices = Vec::with_capacity(count);
    for index in 0..count {
        notices.push(
            service
                .create_notice(input(&format!("Notice {index}"), NOTICE_TYPE, NORMAL_STATUS), "admin".into())
                .await
                .unwrap(),
        );
    }
    notices
}

fn input(title: &str, notice_type: &str, status: &str) -> NoticeInput {
    NoticeInput {
        notice_title: title.into(),
        notice_type: notice_type.into(),
        notice_content: "# Content".into(),
        status: status.into(),
        remark: None,
    }
}

fn list_filter(title: Option<&str>, creator: Option<&str>, notice_type: Option<&str>) -> NoticeListFilter {
    NoticeListFilter {
        page: PageRequest { page: 1, page_size: PAGE_SIZE },
        notice_title: title.map(str::to_owned),
        create_by: creator.map(str::to_owned),
        notice_type: notice_type.map(str::to_owned),
    }
}

fn reader_filter(search: &str) -> NoticeReaderFilter {
    NoticeReaderFilter {
        page: PageRequest { page: 1, page_size: PAGE_SIZE },
        search_value: Some(search.into()),
    }
}

async fn read_count(pool: &sqlx::PgPool, notice_id: &str) -> i64 {
    query_scalar("SELECT COUNT(*) FROM sys_notice_read WHERE notice_id=$1")
        .bind(notice_id)
        .fetch_one(pool)
        .await
        .unwrap()
}
