use super::*;

type RoleUsersRequest = (
    State<RbacApiState>,
    Extension<CurrentUser>,
    Extension<DataScopeFilter>,
    Path<String>,
    Query<RoleUsersQuery>,
);
type RoleUserReplaceRequest = (
    State<RbacApiState>,
    Extension<CurrentUser>,
    Extension<DataScopeFilter>,
    Path<String>,
    RequestJson<RoleUserBindingInput>,
);
type DeleteRoleUserRequest = (State<RbacApiState>, Extension<CurrentUser>, Extension<DataScopeFilter>, Path<(String, String)>);
type DeleteRoleUsersRequest = (
    State<RbacApiState>,
    Extension<CurrentUser>,
    Extension<DataScopeFilter>,
    Path<String>,
    RequestJson<BatchIdsInput>,
);

#[require_perms("system:role:list")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn role_users(request: RoleUsersRequest) -> ApiResult<ApiJson<Page<RoleUser>>> {
    let (State(state), Extension(current_user), Extension(data_scope), Path(id), Query(query)) = request;
    let scope = (!current_user.admin).then_some(data_scope);
    Ok(ok(state.rbac_admin.page_role_users(role_user_filter(id, query), scope).await?))
}

#[require_perms("system:role:edit")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn replace_role_users(request: RoleUserReplaceRequest) -> ApiResult<ApiJson<()>> {
    let (State(state), Extension(current_user), Extension(data_scope), Path(id), RequestJson(payload)) = request;
    RoleUserScopeGuard::new(&state, &current_user, data_scope)
        .ensure_many(payload.user_ids.clone())
        .await?;
    state.rbac_admin.replace_role_users(&id, payload).await?;
    Ok(ok(()))
}

#[require_perms("system:role:remove")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn delete_role_user(request: DeleteRoleUserRequest) -> ApiResult<ApiJson<()>> {
    let (State(state), Extension(current_user), Extension(data_scope), Path((id, user_id))) = request;
    RoleUserScopeGuard::new(&state, &current_user, data_scope).ensure_one(&user_id).await?;
    state.rbac_admin.delete_role_user(&id, &user_id).await?;
    Ok(ok(()))
}

#[require_perms("system:role:remove")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn delete_role_users(request: DeleteRoleUsersRequest) -> ApiResult<ApiJson<()>> {
    let (State(state), Extension(current_user), Extension(data_scope), Path(id), RequestJson(payload)) = request;
    RoleUserScopeGuard::new(&state, &current_user, data_scope)
        .ensure_many(payload.ids.clone())
        .await?;
    state.rbac_admin.delete_role_users(&id, payload.ids).await?;
    Ok(ok(()))
}

struct RoleUserScopeGuard<'a> {
    state: &'a RbacApiState,
    current_user: &'a CurrentUser,
    data_scope: DataScopeFilter,
}

impl<'a> RoleUserScopeGuard<'a> {
    const fn new(state: &'a RbacApiState, current_user: &'a CurrentUser, data_scope: DataScopeFilter) -> Self {
        Self {
            state,
            current_user,
            data_scope,
        }
    }

    async fn ensure_one(&self, user_id: &str) -> ApiResult<()> {
        self.ensure_many(vec![user_id.into()]).await
    }

    async fn ensure_many(&self, user_ids: Vec<String>) -> ApiResult<()> {
        if self.current_user.admin {
            return Ok(());
        }
        let user_ids = clean_user_ids(user_ids);
        if user_ids.is_empty() {
            return Ok(());
        }
        self.state.rbac_admin.ensure_user_ids_scoped(user_ids, self.data_scope.clone()).await?;
        Ok(())
    }
}

fn clean_user_ids(user_ids: Vec<String>) -> Vec<String> {
    user_ids.into_iter().map(|id| id.trim().into()).filter(|id: &String| !id.is_empty()).collect()
}
