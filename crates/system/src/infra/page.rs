use kernel::pagination::{Page, PageRequest};
use storage::{
    StorageResult,
    database::{to_i64, to_u64},
};

pub fn offset(page: PageRequest) -> StorageResult<i64> {
    to_i64((page.page - 1) * page.page_size)
}

pub fn limit(page: PageRequest) -> StorageResult<i64> {
    to_i64(page.page_size)
}

pub fn page<T>(items: Vec<T>, total: i64, request: PageRequest) -> StorageResult<Page<T>> {
    Ok(Page {
        items,
        total: to_u64(total)?,
        page: request.page,
        page_size: request.page_size,
    })
}
