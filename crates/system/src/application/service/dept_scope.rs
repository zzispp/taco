use rbac::domain::DataScopeFilter;

use crate::{
    application::{DeptListFilter, SystemRepository, SystemResult},
    domain::TreeSelectNode,
};

use super::{
    tree::dept_tree,
    validation::{all_depts_filter, reject_unscoped_dept_ids, sanitize_dept_filter},
};

pub(super) async fn scoped_dept_tree<R: SystemRepository>(
    repository: &R,
    filter: DeptListFilter,
    scope: Option<DataScopeFilter>,
) -> SystemResult<Vec<TreeSelectNode>> {
    let filter = sanitize_dept_filter(filter);
    let depts = match scope {
        Some(scope) => repository.list_depts_scoped(filter, scope).await?,
        None => repository.list_depts(filter).await?,
    };
    Ok(dept_tree(depts))
}

pub(super) async fn ensure_dept_ids_scoped<R: SystemRepository>(repository: &R, ids: Vec<String>, scope: DataScopeFilter) -> SystemResult<()> {
    let scoped = repository.list_depts_scoped(all_depts_filter(), scope).await?;
    let scoped_ids = scoped.into_iter().map(|dept| dept.dept_id).collect::<Vec<_>>();
    reject_unscoped_dept_ids(&ids, &scoped_ids)
}
