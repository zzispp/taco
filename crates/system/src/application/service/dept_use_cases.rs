use kernel::pagination::CursorPage;
use rbac::domain::DataScopeFilter;

use crate::{
    application::{DeptListFilter, SystemCache, SystemCursorCodec, SystemError, SystemRepository, SystemResult},
    domain::{Dept, DeptInput, SortBatchInput, TreeSelectNode},
};

use super::{SystemService, dept_scope, tree::dept_tree, validation::*};

impl<R: SystemRepository, C: SystemCache> SystemService<R, C> {
    pub(super) async fn page_depts_impl(&self, filter: DeptListFilter) -> SystemResult<CursorPage<Dept>> {
        let filter = sanitize_dept_filter(filter);
        validate_page(filter.page.clone())?;
        SystemCursorCodec::dept(&filter, None)?.decode(&filter.page)?;
        self.repository.page_depts(filter).await
    }

    pub(super) async fn page_depts_scoped_impl(&self, filter: DeptListFilter, scope: DataScopeFilter) -> SystemResult<CursorPage<Dept>> {
        let filter = sanitize_dept_filter(filter);
        validate_page(filter.page.clone())?;
        SystemCursorCodec::dept(&filter, Some(&scope))?.decode(&filter.page)?;
        self.repository.page_depts_scoped(filter, scope).await
    }

    pub(super) async fn get_dept_impl(&self, id: &str) -> SystemResult<Dept> {
        self.repository.find_dept(id).await?.ok_or(SystemError::NotFound)
    }

    pub(super) async fn dept_tree_impl(&self, filter: DeptListFilter, scope: Option<DataScopeFilter>) -> SystemResult<Vec<TreeSelectNode>> {
        dept_scope::scoped_dept_tree(&self.repository, filter, scope).await
    }

    pub(super) async fn ensure_dept_ids_scoped_impl(&self, ids: Vec<String>, scope: DataScopeFilter) -> SystemResult<()> {
        dept_scope::ensure_dept_ids_scoped(&self.repository, ids, scope).await
    }

    pub(super) async fn exclude_dept_tree_impl(&self, id: &str) -> SystemResult<Vec<TreeSelectNode>> {
        self.get_dept_impl(id).await?;
        Ok(dept_tree(self.repository.list_depts_excluding(id).await?))
    }

    pub(super) async fn create_dept_impl(&self, input: DeptInput) -> SystemResult<Dept> {
        reject_duplicate_dept(&self.repository, &input, None).await?;
        self.repository.create_dept(input).await
    }

    pub(super) async fn replace_dept_impl(&self, id: &str, input: DeptInput) -> SystemResult<Dept> {
        reject_invalid_dept_parent(id, &input)?;
        reject_duplicate_dept(&self.repository, &input, Some(id)).await?;
        if input.status != constants::system::STATUS_NORMAL && self.repository.dept_has_normal_children(id).await? {
            return Err(SystemError::Conflict(localized("errors.system.dept_has_active_children")));
        }
        self.repository.replace_dept(id, input).await
    }

    pub(super) async fn update_dept_sort_impl(&self, id: &str, order_num: i64) -> SystemResult<Dept> {
        self.repository.update_dept_sort(id, order_num).await
    }

    pub(super) async fn update_dept_sorts_impl(&self, input: SortBatchInput) -> SystemResult<Vec<Dept>> {
        let mut items = Vec::with_capacity(input.items.len());
        for item in input.items {
            items.push(self.update_dept_sort_impl(&item.id, item.order_num).await?);
        }
        Ok(items)
    }

    pub(super) async fn delete_dept_impl(&self, id: &str) -> SystemResult<()> {
        reject_dept_delete(&self.repository, id).await?;
        self.repository.delete_dept(id).await
    }
}
