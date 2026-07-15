use super::*;

#[async_trait]
impl SystemRepository for MemoryRepository {
    async fn export(&self, _request: SystemExportRequest, _sink: &mut dyn SystemExportSink) -> SystemResult<()> {
        Ok(())
    }

    async fn page_depts(&self, filter: DeptListFilter) -> system::application::SystemResult<CursorPage<Dept>> {
        self.state.lock().unwrap().last_dept_filter = Some(filter);
        Ok(empty_page())
    }
    async fn page_depts_scoped(&self, filter: DeptListFilter, _scope: DataScopeFilter) -> system::application::SystemResult<CursorPage<Dept>> {
        self.state.lock().unwrap().last_dept_filter = Some(filter);
        Ok(empty_page())
    }
    async fn list_depts(&self, _filter: DeptListFilter) -> system::application::SystemResult<Vec<Dept>> {
        Ok(vec![])
    }
    async fn list_depts_scoped(&self, _filter: DeptListFilter, scope: DataScopeFilter) -> system::application::SystemResult<Vec<Dept>> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .dept
            .clone()
            .into_iter()
            .filter(|dept| memory_dept_scope_matches(dept, &scope))
            .collect())
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
    async fn page_posts(&self, filter: PostListFilter) -> system::application::SystemResult<CursorPage<Post>> {
        self.state.lock().unwrap().last_post_filter = Some(filter);
        Ok(empty_page())
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
    async fn page_dict_types(&self, _filter: DictTypeListFilter) -> system::application::SystemResult<CursorPage<DictType>> {
        Ok(empty_page())
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
    async fn page_dict_data(&self, _filter: DictDataListFilter) -> system::application::SystemResult<CursorPage<DictData>> {
        Ok(empty_page())
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
    async fn page_configs(&self, filter: ConfigListFilter) -> system::application::SystemResult<CursorPage<ConfigItem>> {
        self.state.lock().unwrap().last_config_filter = Some(filter);
        Ok(empty_page())
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
