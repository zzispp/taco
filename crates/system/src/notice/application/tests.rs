use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use constants::system::STATUS_NORMAL;
use kernel::pagination::{CursorPage, CursorPageRequest};

use super::{NoticeRepository, NoticeService, NoticeUseCase, is_safe_url, validate_input, validate_markdown};
use crate::{
    application::{SystemError, SystemResult},
    notice::domain::{
        NOTICE_STATUS_CLOSED, NOTICE_TOP_LIMIT, NOTICE_TYPE_ANNOUNCEMENT, NOTICE_TYPE_NOTICE, Notice, NoticeInput, NoticeListFilter, NoticeReader,
        NoticeReaderFilter, NoticeSummary, NoticeTopResponse,
    },
};

#[derive(Clone, Default)]
struct TestRepository {
    notice: Option<Notice>,
    deleted_ids: Arc<Mutex<Vec<String>>>,
    top_limit: Arc<Mutex<Option<u64>>>,
}

#[async_trait]
impl NoticeRepository for TestRepository {
    async fn page_notices(&self, _filter: NoticeListFilter) -> SystemResult<CursorPage<NoticeSummary>> {
        Ok(page(self.notice.clone().into_iter().map(summary).collect()))
    }

    async fn find_notice(&self, _id: &str) -> SystemResult<Option<Notice>> {
        Ok(self.notice.clone())
    }

    async fn create_notice(&self, input: NoticeInput, operator: &str) -> SystemResult<Notice> {
        Ok(notice_from_input(input, operator))
    }

    async fn replace_notice(&self, _id: &str, input: NoticeInput, operator: &str) -> SystemResult<Notice> {
        Ok(notice_from_input(input, operator))
    }

    async fn delete_notice(&self, id: &str) -> SystemResult<()> {
        self.deleted_ids.lock().expect("deleted ids lock").push(id.into());
        Ok(())
    }

    async fn delete_notices(&self, ids: &[String]) -> SystemResult<()> {
        self.deleted_ids.lock().expect("deleted ids lock").extend_from_slice(ids);
        Ok(())
    }

    async fn top_notices(&self, _user_id: &str, limit: u64) -> SystemResult<NoticeTopResponse> {
        *self.top_limit.lock().expect("top limit lock") = Some(limit);
        Ok(NoticeTopResponse {
            items: Vec::new(),
            unread_count: 0,
        })
    }

    async fn mark_read(&self, _notice_id: &str, _user_id: &str) -> SystemResult<()> {
        Ok(())
    }

    async fn mark_all_read(&self, _user_id: &str) -> SystemResult<()> {
        Ok(())
    }

    async fn page_readers(&self, _notice_id: &str, _filter: NoticeReaderFilter) -> SystemResult<CursorPage<NoticeReader>> {
        Ok(page(Vec::new()))
    }
}

#[test]
fn trims_title_and_accepts_markdown() {
    let result = validate_input(input("  title  ", "# Hello\n\n[docs](https://example.com)"));
    assert_eq!(result.expect("valid input").notice_title, "title");
}

#[test]
fn rejects_html_and_unsafe_urls() {
    assert!(validate_markdown("<script>alert(1)</script>").is_err());
    assert!(!is_safe_url("javascript:alert(1)"));
    assert!(validate_markdown("[x](javascript:alert(1))").is_err());
}

#[test]
fn recognizes_only_rfc_schemes_and_allows_colons_in_relative_links() {
    for url in ["/docs/topic:one", "./docs/topic:one", "docs/topic:one", "?next=topic:one", "#topic:one"] {
        assert!(is_safe_url(url), "relative URL should be allowed: {url}");
    }
    for url in ["//example.com", "javascript:alert(1)", "data:text/html,boom", "custom:value"] {
        assert!(!is_safe_url(url), "unsafe URL should be rejected: {url}");
    }
    for url in ["http://example.com", "https://example.com", "mailto:user@example.com"] {
        assert!(is_safe_url(url), "supported URL should be allowed: {url}");
    }
}

#[test]
fn cursor_validation_rejects_limits_outside_the_public_range() {
    for limit in [0, kernel::pagination::MAX_CURSOR_LIMIT + 1] {
        let request = CursorPageRequest { limit, cursor: None };
        assert!(matches!(
            crate::application::validate_cursor_request(&request),
            Err(SystemError::InvalidInput(message)) if message.key() == "errors.validation.cursor_limit_range"
        ));
    }
}

#[test]
fn rejects_empty_or_long_titles() {
    assert!(validate_input(input(" ", "")).is_err());
    assert!(validate_input(input(&"a".repeat(51), "")).is_err());
}

#[test]
fn validates_type_status_remark_and_empty_markdown() {
    assert!(validate_input(input("title", "")).is_ok());
    assert!(
        validate_input(NoticeInput {
            notice_type: NOTICE_TYPE_ANNOUNCEMENT.into(),
            ..input("title", "")
        })
        .is_ok()
    );
    assert!(
        validate_input(NoticeInput {
            notice_type: "3".into(),
            ..input("title", "")
        })
        .is_err()
    );
    assert!(
        validate_input(NoticeInput {
            status: "2".into(),
            ..input("title", "")
        })
        .is_err()
    );
    assert!(
        validate_input(NoticeInput {
            remark: Some("a".repeat(501)),
            ..input("title", "")
        })
        .is_err()
    );
}

#[tokio::test]
async fn normal_user_cannot_read_closed_notice() {
    let service = NoticeService::new(TestRepository {
        notice: Some(notice(NOTICE_STATUS_CLOSED)),
        ..TestRepository::default()
    });
    assert!(matches!(service.get_notice("notice-1", false).await, Err(SystemError::NotFound)));
}

#[tokio::test]
async fn batch_delete_cleans_and_deduplicates_ids() {
    let repository = TestRepository::default();
    let deleted_ids = repository.deleted_ids.clone();
    let service = NoticeService::new(repository);
    service
        .delete_notices(vec![" notice-2 ".into(), "notice-1".into(), "notice-2".into()])
        .await
        .expect("delete notices");
    assert_eq!(*deleted_ids.lock().expect("deleted ids lock"), vec!["notice-1", "notice-2"]);
}

#[tokio::test]
async fn top_notice_limit_is_fixed_to_domain_constant() {
    let repository = TestRepository::default();
    let top_limit = repository.top_limit.clone();
    NoticeService::new(repository).top_notices("user-1").await.expect("top notices");
    assert_eq!(*top_limit.lock().expect("top limit lock"), Some(NOTICE_TOP_LIMIT));
}

fn input(title: &str, content: &str) -> NoticeInput {
    NoticeInput {
        notice_title: title.into(),
        notice_type: NOTICE_TYPE_NOTICE.into(),
        notice_content: content.into(),
        status: STATUS_NORMAL.into(),
        remark: None,
    }
}

fn notice(status: &str) -> Notice {
    Notice {
        notice_id: "notice-1".into(),
        notice_title: "title".into(),
        notice_type: NOTICE_TYPE_NOTICE.into(),
        notice_content: "content".into(),
        status: status.into(),
        create_by: "admin".into(),
        create_time: "2026-07-13T00:00:00Z".into(),
        update_by: None,
        update_time: None,
        remark: None,
    }
}

fn notice_from_input(input: NoticeInput, operator: &str) -> Notice {
    Notice {
        notice_title: input.notice_title,
        notice_type: input.notice_type,
        notice_content: input.notice_content,
        status: input.status,
        create_by: operator.into(),
        remark: input.remark,
        ..notice(STATUS_NORMAL)
    }
}

fn summary(value: Notice) -> NoticeSummary {
    NoticeSummary {
        notice_id: value.notice_id,
        notice_title: value.notice_title,
        notice_type: value.notice_type,
        status: value.status,
        create_by: value.create_by,
        create_time: value.create_time,
    }
}

fn page<T>(items: Vec<T>) -> CursorPage<T> {
    CursorPage::new(items, None, None)
}
