use constants::system_config::{CAPTCHA_CONFIG_KEY, INIT_PASSWORD_KEY};
use kernel::error::LocalizedError;
use kernel::pagination::PageRequest;

use crate::application::{
    ConfigListFilter, DeptListFilter, DictDataListFilter, DictTypeListFilter, PostListFilter, SystemError, SystemRepository, SystemResult,
};
use crate::domain::{ConfigInput, ConfigItem, DeptInput, DictTypeInput, PostInput};

const DATA_SCOPE_FORBIDDEN_KEY: &str = "errors.system.data_scope_forbidden";

pub(super) fn all_configs_filter() -> ConfigListFilter {
    ConfigListFilter {
        page: PageRequest { page: 1, page_size: 100_000 },
        config_name: None,
        config_key: None,
        config_type: None,
        public_read: None,
        begin_time: None,
        end_time: None,
    }
}

pub(super) fn all_dict_types_filter() -> DictTypeListFilter {
    DictTypeListFilter {
        page: PageRequest { page: 1, page_size: 100_000 },
        dict_name: None,
        dict_type: None,
        status: None,
        begin_time: None,
        end_time: None,
    }
}

pub(super) fn all_depts_filter() -> DeptListFilter {
    DeptListFilter {
        page: PageRequest { page: 1, page_size: 100_000 },
        dept_name: None,
        leader: None,
        phone: None,
        email: None,
        status: None,
        begin_time: None,
        end_time: None,
    }
}

pub(super) async fn reject_duplicate_dept<R: SystemRepository>(repository: &R, input: &DeptInput, current_id: Option<&str>) -> SystemResult<()> {
    let depts = repository
        .list_depts(DeptListFilter {
            page: PageRequest { page: 1, page_size: 100_000 },
            dept_name: None,
            leader: None,
            phone: None,
            email: None,
            status: None,
            begin_time: None,
            end_time: None,
        })
        .await?;
    if depts
        .iter()
        .any(|dept| dept.parent_id == input.parent_id && dept.dept_name == input.dept_name && Some(dept.dept_id.as_str()) != current_id)
    {
        return Err(SystemError::Conflict(localized("errors.system.dept_name_exists")));
    }
    Ok(())
}

pub(super) fn reject_invalid_dept_parent(id: &str, input: &DeptInput) -> SystemResult<()> {
    if input.parent_id == id {
        return Err(SystemError::Conflict(localized("errors.system.dept_parent_self")));
    }
    Ok(())
}

pub(super) fn reject_unscoped_dept_ids(requested: &[String], scoped: &[String]) -> SystemResult<()> {
    if requested.iter().all(|id| scoped.contains(id)) {
        return Ok(());
    }
    Err(SystemError::Forbidden(localized(DATA_SCOPE_FORBIDDEN_KEY)))
}

pub(super) async fn reject_duplicate_dict_type<R: SystemRepository>(repository: &R, input: &DictTypeInput, current_id: Option<&str>) -> SystemResult<()> {
    let items = repository
        .page_dict_types(DictTypeListFilter {
            page: PageRequest { page: 1, page_size: 100_000 },
            dict_name: None,
            dict_type: Some(input.dict_type.clone()),
            status: None,
            begin_time: None,
            end_time: None,
        })
        .await?;
    if items
        .items
        .iter()
        .any(|item| item.dict_type == input.dict_type && Some(item.dict_id.as_str()) != current_id)
    {
        return Err(SystemError::Conflict(localized("errors.system.dict_type_exists")));
    }
    Ok(())
}

pub(super) async fn reject_duplicate_config_key<R: SystemRepository>(repository: &R, input: &ConfigInput, current_id: Option<&str>) -> SystemResult<()> {
    let items = repository
        .page_configs(ConfigListFilter {
            page: PageRequest { page: 1, page_size: 100_000 },
            config_name: None,
            config_key: Some(input.config_key.clone()),
            config_type: None,
            public_read: None,
            begin_time: None,
            end_time: None,
        })
        .await?;
    if items
        .items
        .iter()
        .any(|item| item.config_key == input.config_key && Some(item.config_id.as_str()) != current_id)
    {
        return Err(SystemError::Conflict(localized("errors.system.config_key_exists")));
    }
    Ok(())
}

pub(super) fn clean_config_keys(keys: Vec<String>) -> SystemResult<Vec<String>> {
    let keys = keys
        .into_iter()
        .map(|key| key.trim().to_owned())
        .filter(|key| !key.is_empty())
        .collect::<Vec<_>>();
    if keys.is_empty() {
        return Err(SystemError::InvalidInput(localized("errors.system.keys_required")));
    }
    Ok(keys)
}

pub(super) fn reject_builtin_config_delete(item: &ConfigItem) -> SystemResult<()> {
    if item.config_type == "Y" {
        return Err(SystemError::Conflict(localized("errors.system.builtin_config_delete")));
    }
    Ok(())
}

pub(super) fn reject_builtin_config_identity_change(current: &ConfigItem, input: &ConfigInput) -> SystemResult<()> {
    if current.config_type == "Y" && current.config_key != input.config_key {
        return Err(SystemError::Conflict(localized("errors.system.builtin_config_key_change")));
    }
    if current.config_type == "Y" && input.config_type != "Y" {
        return Err(SystemError::Conflict(localized("errors.system.builtin_config_type_change")));
    }
    Ok(())
}

pub(super) fn reject_sensitive_public_config(key: &str, public_read: bool) -> SystemResult<()> {
    if key == INIT_PASSWORD_KEY && public_read {
        return Err(SystemError::Conflict(localized("errors.system.initial_password_public")));
    }
    if key == CAPTCHA_CONFIG_KEY && public_read {
        return Err(SystemError::Conflict(localized("errors.system.captcha_private_public")));
    }
    Ok(())
}

pub(super) async fn reject_dept_delete<R: SystemRepository>(repository: &R, id: &str) -> SystemResult<()> {
    if repository.dept_has_children(id).await? || repository.dept_has_users(id).await? {
        return Err(SystemError::Conflict(localized("errors.system.dept_has_children_or_users")));
    }
    Ok(())
}

pub(super) async fn reject_post_delete<R: SystemRepository>(repository: &R, id: &str) -> SystemResult<()> {
    if repository.post_has_users(id).await? {
        return Err(SystemError::Conflict(localized("errors.system.post_assigned_to_users")));
    }
    Ok(())
}

pub(super) async fn reject_duplicate_post<R: SystemRepository>(repository: &R, input: &PostInput, current_id: Option<&str>) -> SystemResult<()> {
    if repository.post_code_exists(&input.post_code, current_id).await? {
        return Err(SystemError::Conflict(localized("errors.system.post_code_exists")));
    }
    if repository.post_name_exists(&input.post_name, current_id).await? {
        return Err(SystemError::Conflict(localized("errors.system.post_name_exists")));
    }
    Ok(())
}

pub(super) fn validate_page(page: PageRequest) -> SystemResult<()> {
    if page.page == 0 || page.page_size == 0 {
        return Err(SystemError::InvalidInput(localized("errors.validation.page_and_size_positive")));
    }
    Ok(())
}

pub(super) fn sanitize_dept_filter(input: DeptListFilter) -> DeptListFilter {
    DeptListFilter {
        page: input.page,
        dept_name: trim(input.dept_name),
        leader: trim(input.leader),
        phone: trim(input.phone),
        email: trim(input.email),
        status: trim(input.status),
        begin_time: input.begin_time,
        end_time: input.end_time,
    }
}

pub(super) fn sanitize_post_filter(input: PostListFilter) -> PostListFilter {
    PostListFilter {
        page: input.page,
        post_code: trim(input.post_code),
        post_name: trim(input.post_name),
        status: trim(input.status),
        remark: trim(input.remark),
        begin_time: input.begin_time,
        end_time: input.end_time,
    }
}

pub(super) fn sanitize_dict_type_filter(input: DictTypeListFilter) -> DictTypeListFilter {
    DictTypeListFilter {
        page: input.page,
        dict_name: trim(input.dict_name),
        dict_type: trim(input.dict_type),
        status: trim(input.status),
        begin_time: input.begin_time,
        end_time: input.end_time,
    }
}

pub(super) fn sanitize_dict_data_filter(input: DictDataListFilter) -> DictDataListFilter {
    DictDataListFilter {
        page: input.page,
        dict_type: trim(input.dict_type),
        dict_label: trim(input.dict_label),
        status: trim(input.status),
        begin_time: input.begin_time,
        end_time: input.end_time,
    }
}

pub(super) fn sanitize_config_filter(input: ConfigListFilter) -> ConfigListFilter {
    ConfigListFilter {
        page: input.page,
        config_name: trim(input.config_name),
        config_key: trim(input.config_key),
        config_type: trim(input.config_type),
        public_read: input.public_read,
        begin_time: input.begin_time,
        end_time: input.end_time,
    }
}

fn trim(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().into()).filter(|item: &String| !item.is_empty())
}

pub(super) fn localized(key: &'static str) -> LocalizedError {
    LocalizedError::new(key)
}

pub(super) fn localized_param(key: &'static str, param: &'static str, value: impl Into<String>) -> LocalizedError {
    LocalizedError::new(key).with_param(param, value)
}

pub(super) fn reject_empty_ids(ids: &[String]) -> SystemResult<()> {
    if ids.is_empty() {
        return Err(SystemError::InvalidInput(localized("errors.system.ids_required")));
    }
    Ok(())
}
