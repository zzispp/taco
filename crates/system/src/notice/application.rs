use async_trait::async_trait;
use constants::system::STATUS_NORMAL;
use kernel::{error::LocalizedError, pagination::CursorPage};
use pulldown_cmark::{Event, Parser, Tag};

use crate::application::{SystemError, SystemResult, validate_cursor_request};

use super::domain::{
    NOTICE_STATUS_CLOSED, NOTICE_TOP_LIMIT, NOTICE_TYPE_ANNOUNCEMENT, NOTICE_TYPE_NOTICE, Notice, NoticeInput, NoticeListFilter, NoticeReader,
    NoticeReaderFilter, NoticeSummary, NoticeTopResponse,
};

const TITLE_MAX_LENGTH: usize = 50;
const REMARK_MAX_LENGTH: usize = 500;
const INVALID_NOTICE_KEY: &str = "errors.system.notice_invalid";
const INVALID_REMARK_KEY: &str = "errors.system.notice_invalid_remark";
const INVALID_MARKDOWN_KEY: &str = "errors.system.notice_invalid_markdown";
pub(super) const EMPTY_IDS_KEY: &str = "errors.system.ids_required";

#[async_trait]
pub trait NoticeUseCase: Send + Sync + 'static {
    async fn page_notices(&self, filter: NoticeListFilter) -> SystemResult<CursorPage<NoticeSummary>>;
    async fn get_notice(&self, id: &str, can_view_closed: bool) -> SystemResult<Notice>;
    async fn create_notice(&self, input: NoticeInput, operator: String) -> SystemResult<Notice>;
    async fn replace_notice(&self, id: &str, input: NoticeInput, operator: String) -> SystemResult<Notice>;
    async fn delete_notice(&self, id: &str) -> SystemResult<()>;
    async fn delete_notices(&self, ids: Vec<String>) -> SystemResult<()>;
    async fn top_notices(&self, user_id: &str) -> SystemResult<NoticeTopResponse>;
    async fn mark_read(&self, notice_id: &str, user_id: &str) -> SystemResult<()>;
    async fn mark_all_read(&self, user_id: &str) -> SystemResult<()>;
    async fn page_readers(&self, notice_id: &str, filter: NoticeReaderFilter) -> SystemResult<CursorPage<NoticeReader>>;
}

#[async_trait]
pub trait NoticeRepository: Send + Sync + 'static {
    async fn page_notices(&self, filter: NoticeListFilter) -> SystemResult<CursorPage<NoticeSummary>>;
    async fn find_notice(&self, id: &str) -> SystemResult<Option<Notice>>;
    async fn create_notice(&self, input: NoticeInput, operator: &str) -> SystemResult<Notice>;
    async fn replace_notice(&self, id: &str, input: NoticeInput, operator: &str) -> SystemResult<Notice>;
    async fn delete_notice(&self, id: &str) -> SystemResult<()>;
    async fn delete_notices(&self, ids: &[String]) -> SystemResult<()>;
    async fn top_notices(&self, user_id: &str, limit: u64) -> SystemResult<NoticeTopResponse>;
    async fn mark_read(&self, notice_id: &str, user_id: &str) -> SystemResult<()>;
    async fn mark_all_read(&self, user_id: &str) -> SystemResult<()>;
    async fn page_readers(&self, notice_id: &str, filter: NoticeReaderFilter) -> SystemResult<CursorPage<NoticeReader>>;
}

pub struct NoticeService<R> {
    pub(super) repository: R,
}

impl<R: NoticeRepository> NoticeService<R> {
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: NoticeRepository> NoticeUseCase for NoticeService<R> {
    async fn page_notices(&self, filter: NoticeListFilter) -> SystemResult<CursorPage<NoticeSummary>> {
        let filter = sanitize_list_filter(filter)?;
        self.repository.page_notices(filter).await
    }

    async fn get_notice(&self, id: &str, can_view_closed: bool) -> SystemResult<Notice> {
        let notice = self.repository.find_notice(id).await?.ok_or(SystemError::NotFound)?;
        if !can_view_closed && notice.status != STATUS_NORMAL {
            return Err(SystemError::NotFound);
        }
        Ok(notice)
    }

    async fn create_notice(&self, input: NoticeInput, operator: String) -> SystemResult<Notice> {
        let input = validate_input(input)?;
        self.repository.create_notice(input, &operator).await
    }

    async fn replace_notice(&self, id: &str, input: NoticeInput, operator: String) -> SystemResult<Notice> {
        let input = validate_input(input)?;
        self.repository.replace_notice(id, input, &operator).await
    }

    async fn delete_notice(&self, id: &str) -> SystemResult<()> {
        self.repository.delete_notice(id).await
    }

    async fn delete_notices(&self, ids: Vec<String>) -> SystemResult<()> {
        let ids = clean_ids(ids);
        if ids.is_empty() {
            return Err(SystemError::InvalidInput(LocalizedError::new(EMPTY_IDS_KEY)));
        }
        self.repository.delete_notices(&ids).await
    }

    async fn top_notices(&self, user_id: &str) -> SystemResult<NoticeTopResponse> {
        self.repository.top_notices(user_id, NOTICE_TOP_LIMIT).await
    }

    async fn mark_read(&self, notice_id: &str, user_id: &str) -> SystemResult<()> {
        let notice = self.repository.find_notice(notice_id).await?.ok_or(SystemError::NotFound)?;
        if notice.status != STATUS_NORMAL {
            return Err(SystemError::NotFound);
        }
        self.repository.mark_read(notice_id, user_id).await
    }

    async fn mark_all_read(&self, user_id: &str) -> SystemResult<()> {
        self.repository.mark_all_read(user_id).await
    }

    async fn page_readers(&self, notice_id: &str, filter: NoticeReaderFilter) -> SystemResult<CursorPage<NoticeReader>> {
        let filter = sanitize_reader_filter(filter)?;
        self.repository.find_notice(notice_id).await?.ok_or(SystemError::NotFound)?;
        self.repository.page_readers(notice_id, filter).await
    }
}

pub(super) fn validate_input(input: NoticeInput) -> SystemResult<NoticeInput> {
    let title = input.notice_title.trim().to_owned();
    if title.is_empty() || title.chars().count() > TITLE_MAX_LENGTH {
        return Err(SystemError::InvalidInput(LocalizedError::new(INVALID_NOTICE_KEY)));
    }
    if !matches!(input.notice_type.as_str(), NOTICE_TYPE_NOTICE | NOTICE_TYPE_ANNOUNCEMENT)
        || !matches!(input.status.as_str(), STATUS_NORMAL | NOTICE_STATUS_CLOSED)
    {
        return Err(SystemError::InvalidInput(LocalizedError::new(INVALID_NOTICE_KEY)));
    }
    if input.remark.as_ref().is_some_and(|value| value.chars().count() > REMARK_MAX_LENGTH) {
        return Err(SystemError::InvalidInput(
            LocalizedError::new(INVALID_REMARK_KEY).with_param("max", REMARK_MAX_LENGTH.to_string()),
        ));
    }
    validate_markdown(&input.notice_content)?;
    Ok(NoticeInput {
        notice_title: title,
        remark: trim_optional(input.remark),
        ..input
    })
}

fn sanitize_list_filter(filter: NoticeListFilter) -> SystemResult<NoticeListFilter> {
    validate_cursor_request(&filter.page)?;
    let notice_type = trim_optional(filter.notice_type);
    if notice_type
        .as_deref()
        .is_some_and(|value| !matches!(value, NOTICE_TYPE_NOTICE | NOTICE_TYPE_ANNOUNCEMENT))
    {
        return Err(SystemError::InvalidInput(LocalizedError::new(INVALID_NOTICE_KEY)));
    }
    Ok(NoticeListFilter {
        page: filter.page,
        notice_title: trim_optional(filter.notice_title),
        create_by: trim_optional(filter.create_by),
        notice_type,
    })
}

fn sanitize_reader_filter(filter: NoticeReaderFilter) -> SystemResult<NoticeReaderFilter> {
    validate_cursor_request(&filter.page)?;
    Ok(NoticeReaderFilter {
        page: filter.page,
        search_value: trim_optional(filter.search_value),
    })
}

fn validate_markdown(value: &str) -> SystemResult<()> {
    for event in Parser::new(value) {
        let destination = match event {
            Event::Start(Tag::Link { dest_url, .. }) | Event::Start(Tag::Image { dest_url, .. }) => Some(dest_url),
            Event::Html(_) | Event::InlineHtml(_) => return Err(SystemError::InvalidInput(LocalizedError::new(INVALID_MARKDOWN_KEY))),
            _ => None,
        };
        if destination.is_some_and(|url| !is_safe_url(&url)) {
            return Err(SystemError::InvalidInput(LocalizedError::new(INVALID_MARKDOWN_KEY)));
        }
    }
    Ok(())
}

fn is_safe_url(url: &str) -> bool {
    let normalized = url.trim().to_ascii_lowercase();
    if normalized.starts_with("//") {
        return false;
    }
    url_scheme(&normalized).is_none_or(|scheme| matches!(scheme, "http" | "https" | "mailto"))
}

fn url_scheme(url: &str) -> Option<&str> {
    let separator = url.find(':')?;
    let candidate = &url[..separator];
    let mut characters = candidate.chars();
    if !characters.next().is_some_and(|character| character.is_ascii_alphabetic()) {
        return None;
    }
    characters
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.'))
        .then_some(candidate)
}

pub(super) fn clean_ids(ids: Vec<String>) -> Vec<String> {
    let mut cleaned = ids.into_iter().map(|id| id.trim().to_owned()).filter(|id| !id.is_empty()).collect::<Vec<_>>();
    cleaned.sort_unstable();
    cleaned.dedup();
    cleaned
}

fn trim_optional(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().to_owned()).filter(|item| !item.is_empty())
}

#[cfg(test)]
mod tests;
