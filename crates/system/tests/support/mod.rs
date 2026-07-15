use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;
use kernel::error::LocalizedError;
use kernel::pagination::{CursorPage, CursorPageRequest};
use rbac::domain::{DataScope, DataScopeFilter};
use system::{
    application::{
        ConfigListFilter, DeptListFilter, DictDataListFilter, DictTypeListFilter, PostListFilter, SystemError, SystemExportRequest, SystemExportSink,
        SystemRepository, SystemResult,
    },
    domain::{ConfigInput, ConfigItem, Dept, DeptInput, DictData, DictDataInput, DictType, DictTypeInput, Post, PostInput},
};

#[derive(Clone, Default)]
pub(crate) struct MemoryRepository {
    state: Arc<Mutex<State>>,
}

#[derive(Default)]
struct State {
    dept: Option<Dept>,
    dict_type: Option<DictType>,
    dict_type_has_data: bool,
    configs: HashMap<String, ConfigItem>,
    duplicate_post_code: bool,
    duplicate_post_name: bool,
    dept_has_children: bool,
    dept_has_users: bool,
    deleted_dict_types: Vec<String>,
    updated_dept_sorts: Vec<(String, i64)>,
    audit_records: Vec<AuditOutboxRecord>,
    last_dept_filter: Option<DeptListFilter>,
    last_post_filter: Option<PostListFilter>,
    last_config_filter: Option<ConfigListFilter>,
}

impl MemoryRepository {
    pub(crate) fn with_dept(self, dept: Dept) -> Self {
        self.state.lock().unwrap().dept = Some(dept);
        self
    }
    pub(crate) fn with_dict_type(self, dict_type: DictType) -> Self {
        self.state.lock().unwrap().dict_type = Some(dict_type);
        self
    }
    pub(crate) fn with_dict_data(self, exists: bool) -> Self {
        self.state.lock().unwrap().dict_type_has_data = exists;
        self
    }
    pub(crate) fn with_config(self, key: &str, value: &str) -> Self {
        self.state.lock().unwrap().configs.insert(key.into(), private_config_item(key, value));
        self
    }

    pub(crate) fn with_config_item(self, item: ConfigItem) -> Self {
        self.state.lock().unwrap().configs.insert(item.config_key.clone(), item);
        self
    }
    pub(crate) fn with_duplicate_post_code(self, exists: bool) -> Self {
        self.state.lock().unwrap().duplicate_post_code = exists;
        self
    }
    pub(crate) fn with_duplicate_post_name(self, exists: bool) -> Self {
        self.state.lock().unwrap().duplicate_post_name = exists;
        self
    }
    pub(crate) fn with_dept_children(self, exists: bool) -> Self {
        self.state.lock().unwrap().dept_has_children = exists;
        self
    }
    pub(crate) fn with_dept_users(self, exists: bool) -> Self {
        self.state.lock().unwrap().dept_has_users = exists;
        self
    }
    pub(crate) fn deleted_dict_types(&self) -> Vec<String> {
        self.state.lock().unwrap().deleted_dict_types.clone()
    }
    pub(crate) fn updated_dept_sorts(&self) -> Vec<(String, i64)> {
        self.state.lock().unwrap().updated_dept_sorts.clone()
    }
    pub(crate) fn last_dept_filter(&self) -> Option<DeptListFilter> {
        self.state.lock().unwrap().last_dept_filter.clone()
    }
    pub(crate) fn last_post_filter(&self) -> Option<PostListFilter> {
        self.state.lock().unwrap().last_post_filter.clone()
    }

    pub(crate) fn last_config_filter(&self) -> Option<ConfigListFilter> {
        self.state.lock().unwrap().last_config_filter.clone()
    }

    pub(crate) fn audit_records(&self) -> Vec<AuditOutboxRecord> {
        self.state.lock().unwrap().audit_records.clone()
    }

    fn record_audit(&self, audit: &AuditOutboxRecord) {
        self.state.lock().unwrap().audit_records.push(audit.clone());
    }
}

mod audited_repository;
mod repository;

impl MemoryRepository {
    fn update_dept(&self, id: &str, order_num: i64) -> system::application::SystemResult<Dept> {
        let mut state = self.state.lock().unwrap();
        let mut dept = state.dept.clone().ok_or(SystemError::NotFound)?;
        dept.order_num = order_num;
        state.updated_dept_sorts.push((id.into(), order_num));
        Ok(dept)
    }
}

fn empty_page<T>() -> CursorPage<T> {
    CursorPage::new(vec![], None, None)
}
pub(crate) fn page() -> CursorPageRequest {
    CursorPageRequest { limit: 10, cursor: None }
}
pub(crate) fn post_input(code: &str, name: &str) -> PostInput {
    PostInput {
        post_code: code.into(),
        post_name: name.into(),
        post_sort: 1,
        status: "0".into(),
        remark: None,
    }
}
pub(crate) fn dict_type(id: &str, kind: &str) -> DictType {
    DictType {
        dict_id: id.into(),
        dict_name: "用户性别".into(),
        dict_type: kind.into(),
        status: "0".into(),
        remark: None,
        create_time: "2026-01-01 00:00:00".into(),
    }
}
fn post(id: &str, code: &str, name: &str) -> Post {
    Post {
        post_id: id.into(),
        post_code: code.into(),
        post_name: name.into(),
        post_sort: 1,
        status: "0".into(),
        remark: None,
        create_time: "2026-01-01 00:00:00".into(),
    }
}
struct ConfigItemSeed<'a> {
    key: &'a str,
    value: &'a str,
    config_type: &'a str,
    public_read: bool,
}

pub(crate) fn public_config_item(key: &str, value: &str) -> ConfigItem {
    config_item(ConfigItemSeed {
        key,
        value,
        config_type: "Y",
        public_read: true,
    })
}

fn private_config_item(key: &str, value: &str) -> ConfigItem {
    config_item(ConfigItemSeed {
        key,
        value,
        config_type: "N",
        public_read: false,
    })
}

pub(crate) struct ConfigInputSeed<'a> {
    key: &'a str,
    value: &'a str,
    config_type: &'a str,
    public_read: bool,
}

impl<'a> ConfigInputSeed<'a> {
    pub(crate) const fn public(key: &'a str, value: &'a str) -> Self {
        Self {
            key,
            value,
            config_type: "Y",
            public_read: true,
        }
    }
}

pub(crate) fn config_input(seed: ConfigInputSeed<'_>) -> ConfigInput {
    ConfigInput {
        config_name: seed.key.into(),
        config_key: seed.key.into(),
        config_value: seed.value.into(),
        config_type: seed.config_type.into(),
        public_read: seed.public_read,
        remark: None,
    }
}

fn config_item(seed: ConfigItemSeed<'_>) -> ConfigItem {
    ConfigItem {
        config_id: seed.key.into(),
        config_name: seed.key.into(),
        config_key: seed.key.into(),
        config_value: seed.value.into(),
        config_type: seed.config_type.into(),
        public_read: seed.public_read,
        remark: None,
        create_time: "2026-01-01 00:00:00".into(),
    }
}
fn config_from_input(id: &str, input: ConfigInput) -> ConfigItem {
    ConfigItem {
        config_id: id.into(),
        config_name: input.config_name,
        config_key: input.config_key,
        config_value: input.config_value,
        config_type: input.config_type,
        public_read: input.public_read,
        remark: input.remark,
        create_time: "2026-01-01 00:00:00".into(),
    }
}

fn memory_dept_scope_matches(dept: &Dept, scope: &DataScopeFilter) -> bool {
    match scope.data_scope {
        DataScope::All => true,
        DataScope::Custom => scope.dept_ids.contains(&dept.dept_id),
        DataScope::Department | DataScope::SelfOnly => scope.dept_id.as_deref() == Some(dept.dept_id.as_str()),
        DataScope::DepartmentAndChildren => dept_in_scope_tree(dept, scope.dept_id.as_deref()),
    }
}

fn dept_in_scope_tree(dept: &Dept, root: Option<&str>) -> bool {
    let Some(root) = root else {
        return false;
    };
    dept.dept_id == root || dept.ancestors.split(',').any(|id| id == root)
}
pub(crate) fn dept(id: &str, parent_id: &str, name: &str) -> Dept {
    Dept {
        dept_id: id.into(),
        parent_id: parent_id.into(),
        ancestors: "0,100".into(),
        dept_name: name.into(),
        order_num: 1,
        leader: None,
        phone: None,
        email: None,
        status: "0".into(),
        create_time: "2026-01-01 00:00:00".into(),
    }
}
