use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use kernel::error::LocalizedError;
use kernel::pagination::{Page, PageRequest};
use system::{
    application::{ConfigListFilter, DeptListFilter, DictDataListFilter, DictTypeListFilter, PostListFilter, SystemError, SystemRepository},
    domain::{ConfigInput, ConfigItem, Dept, DeptInput, DictData, DictDataInput, DictType, DictTypeInput, Post, PostInput},
};
use types::rbac::DataScopeFilter;

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
    last_post_filter: Option<PostListFilter>,
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
        self.state.lock().unwrap().configs.insert(key.into(), config_item(key, value, "N", false));
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
    pub(crate) fn last_post_filter(&self) -> Option<PostListFilter> {
        self.state.lock().unwrap().last_post_filter.clone()
    }
}

#[async_trait]
impl SystemRepository for MemoryRepository {
    async fn page_depts(&self, filter: DeptListFilter) -> system::application::SystemResult<Page<Dept>> {
        Ok(empty_page(filter.page))
    }
    async fn page_depts_scoped(&self, filter: DeptListFilter, _scope: DataScopeFilter) -> system::application::SystemResult<Page<Dept>> {
        Ok(empty_page(filter.page))
    }
    async fn list_depts(&self, _filter: DeptListFilter) -> system::application::SystemResult<Vec<Dept>> {
        Ok(vec![])
    }
    async fn list_depts_scoped(&self, _filter: DeptListFilter, _scope: DataScopeFilter) -> system::application::SystemResult<Vec<Dept>> {
        Ok(vec![])
    }
    async fn list_depts_excluding(&self, _id: &str) -> system::application::SystemResult<Vec<Dept>> {
        Ok(vec![])
    }
    async fn find_dept(&self, _id: &str) -> system::application::SystemResult<Option<Dept>> {
        Ok(self.state.lock().unwrap().dept.clone())
    }
    async fn create_dept(&self, _input: DeptInput) -> system::application::SystemResult<Dept> {
        unimplemented!("create_dept")
    }
    async fn replace_dept(&self, _id: &str, _input: DeptInput) -> system::application::SystemResult<Dept> {
        unimplemented!("replace_dept")
    }
    async fn update_dept_sort(&self, id: &str, order_num: i64) -> system::application::SystemResult<Dept> {
        self.update_dept(id, order_num)
    }
    async fn delete_dept(&self, _id: &str) -> system::application::SystemResult<()> {
        Ok(())
    }
    async fn dept_has_children(&self, _id: &str) -> system::application::SystemResult<bool> {
        Ok(self.state.lock().unwrap().dept_has_children)
    }
    async fn dept_has_users(&self, _id: &str) -> system::application::SystemResult<bool> {
        Ok(self.state.lock().unwrap().dept_has_users)
    }
    async fn dept_has_normal_children(&self, _id: &str) -> system::application::SystemResult<bool> {
        Ok(false)
    }
    async fn page_posts(&self, filter: PostListFilter) -> system::application::SystemResult<Page<Post>> {
        let page = filter.page;
        self.state.lock().unwrap().last_post_filter = Some(filter);
        Ok(empty_page(page))
    }
    async fn find_post(&self, _id: &str) -> system::application::SystemResult<Option<Post>> {
        Ok(None)
    }
    async fn post_options(&self) -> system::application::SystemResult<Vec<Post>> {
        Ok(vec![])
    }
    async fn post_code_exists(&self, _code: &str, _current_id: Option<&str>) -> system::application::SystemResult<bool> {
        Ok(self.state.lock().unwrap().duplicate_post_code)
    }
    async fn post_name_exists(&self, _name: &str, _current_id: Option<&str>) -> system::application::SystemResult<bool> {
        Ok(self.state.lock().unwrap().duplicate_post_name)
    }
    async fn create_post(&self, _input: PostInput) -> system::application::SystemResult<Post> {
        Ok(post("1", "ceo", "董事长"))
    }
    async fn replace_post(&self, _id: &str, _input: PostInput) -> system::application::SystemResult<Post> {
        Ok(post("1", "ceo", "董事长"))
    }
    async fn delete_post(&self, _id: &str) -> system::application::SystemResult<()> {
        Ok(())
    }
    async fn delete_posts(&self, ids: &[String]) -> system::application::SystemResult<()> {
        for id in ids {
            if self.post_has_users(id).await? {
                return Err(SystemError::Conflict(LocalizedError::new("errors.system.post_assigned_to_users")));
            }
        }
        Ok(())
    }
    async fn post_has_users(&self, _id: &str) -> system::application::SystemResult<bool> {
        Ok(false)
    }
    async fn page_dict_types(&self, filter: DictTypeListFilter) -> system::application::SystemResult<Page<DictType>> {
        Ok(empty_page(filter.page))
    }
    async fn list_dict_types(&self, _filter: DictTypeListFilter) -> system::application::SystemResult<Vec<DictType>> {
        Ok(self.state.lock().unwrap().dict_type.clone().into_iter().collect())
    }
    async fn find_dict_type(&self, _id: &str) -> system::application::SystemResult<Option<DictType>> {
        Ok(self.state.lock().unwrap().dict_type.clone())
    }
    async fn dict_type_options(&self) -> system::application::SystemResult<Vec<DictType>> {
        Ok(vec![])
    }
    async fn dict_type_has_data(&self, _dict_type: &str) -> system::application::SystemResult<bool> {
        Ok(self.state.lock().unwrap().dict_type_has_data)
    }
    async fn create_dict_type(&self, _input: DictTypeInput) -> system::application::SystemResult<DictType> {
        unimplemented!("create_dict_type")
    }
    async fn replace_dict_type(&self, _id: &str, _input: DictTypeInput) -> system::application::SystemResult<DictType> {
        unimplemented!("replace_dict_type")
    }
    async fn delete_dict_type(&self, id: &str) -> system::application::SystemResult<()> {
        self.state.lock().unwrap().deleted_dict_types.push(id.into());
        Ok(())
    }
    async fn delete_dict_types(&self, ids: &[String]) -> system::application::SystemResult<()> {
        self.state.lock().unwrap().deleted_dict_types.extend(ids.iter().cloned());
        Ok(())
    }
    async fn page_dict_data(&self, filter: DictDataListFilter) -> system::application::SystemResult<Page<DictData>> {
        Ok(empty_page(filter.page))
    }
    async fn find_dict_data(&self, _id: &str) -> system::application::SystemResult<Option<DictData>> {
        Ok(None)
    }
    async fn dict_data_by_type(&self, _dict_type: &str) -> system::application::SystemResult<Vec<DictData>> {
        Ok(vec![])
    }
    async fn create_dict_data(&self, _input: DictDataInput) -> system::application::SystemResult<DictData> {
        unimplemented!("create_dict_data")
    }
    async fn replace_dict_data(&self, _id: &str, _input: DictDataInput) -> system::application::SystemResult<DictData> {
        unimplemented!("replace_dict_data")
    }
    async fn delete_dict_data(&self, _id: &str) -> system::application::SystemResult<()> {
        Ok(())
    }
    async fn delete_dict_data_batch(&self, _ids: &[String]) -> system::application::SystemResult<()> {
        Ok(())
    }
    async fn page_configs(&self, filter: ConfigListFilter) -> system::application::SystemResult<Page<ConfigItem>> {
        Ok(empty_page(filter.page))
    }
    async fn list_configs(&self, _filter: ConfigListFilter) -> system::application::SystemResult<Vec<ConfigItem>> {
        Ok(self.state.lock().unwrap().configs.values().cloned().collect())
    }
    async fn find_config(&self, id: &str) -> system::application::SystemResult<Option<ConfigItem>> {
        Ok(self.state.lock().unwrap().configs.values().find(|item| item.config_id == id).cloned())
    }
    async fn find_config_by_key(&self, key: &str) -> system::application::SystemResult<Option<ConfigItem>> {
        Ok(self.state.lock().unwrap().configs.get(key).cloned())
    }
    async fn config_by_key(&self, key: &str) -> system::application::SystemResult<Option<String>> {
        Ok(self.state.lock().unwrap().configs.get(key).map(|item| item.config_value.clone()))
    }
    async fn create_config(&self, input: ConfigInput) -> system::application::SystemResult<ConfigItem> {
        let item = config_from_input("created", input);
        self.state.lock().unwrap().configs.insert(item.config_key.clone(), item.clone());
        Ok(item)
    }
    async fn replace_config(&self, id: &str, input: ConfigInput) -> system::application::SystemResult<ConfigItem> {
        let item = config_from_input(id, input);
        self.state.lock().unwrap().configs.insert(item.config_key.clone(), item.clone());
        Ok(item)
    }
    async fn delete_config(&self, id: &str) -> system::application::SystemResult<()> {
        self.state.lock().unwrap().configs.retain(|_, item| item.config_id != id);
        Ok(())
    }
    async fn delete_configs(&self, ids: &[String]) -> system::application::SystemResult<()> {
        self.state.lock().unwrap().configs.retain(|_, item| !ids.contains(&item.config_id));
        Ok(())
    }
}

impl MemoryRepository {
    fn update_dept(&self, id: &str, order_num: i64) -> system::application::SystemResult<Dept> {
        let mut state = self.state.lock().unwrap();
        let mut dept = state.dept.clone().ok_or(SystemError::NotFound)?;
        dept.order_num = order_num;
        state.updated_dept_sorts.push((id.into(), order_num));
        Ok(dept)
    }
}

fn empty_page<T>(page: PageRequest) -> Page<T> {
    Page {
        items: vec![],
        total: 0,
        page: page.page,
        page_size: page.page_size,
    }
}
pub(crate) fn page() -> PageRequest {
    PageRequest { page: 1, page_size: 10 }
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
pub(crate) fn config_item(key: &str, value: &str, config_type: &str, public_read: bool) -> ConfigItem {
    ConfigItem {
        config_id: key.into(),
        config_name: key.into(),
        config_key: key.into(),
        config_value: value.into(),
        config_type: config_type.into(),
        public_read,
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
