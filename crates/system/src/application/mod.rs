use async_trait::async_trait;
use kernel::pagination::{Page, PageRequest};
use std::collections::BTreeMap;
use thiserror::Error;

use crate::domain::{
    ConfigInput, ConfigItem, Dept, DeptInput, DictData, DictDataInput, DictType, DictTypeInput, Post, PostInput, SortBatchInput, TreeSelectNode,
};
use types::rbac::DataScopeFilter;

pub type SystemResult<T> = Result<T, SystemError>;

#[derive(Debug, Error)]
pub enum SystemError {
    #[error("resource not found")]
    NotFound,
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("resource conflict: {0}")]
    Conflict(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeptListFilter {
    pub page: PageRequest,
    pub dept_name: Option<String>,
    pub status: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostListFilter {
    pub page: PageRequest,
    pub post_code: Option<String>,
    pub post_name: Option<String>,
    pub status: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DictTypeListFilter {
    pub page: PageRequest,
    pub dict_name: Option<String>,
    pub dict_type: Option<String>,
    pub status: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DictDataListFilter {
    pub page: PageRequest,
    pub dict_type: Option<String>,
    pub dict_label: Option<String>,
    pub status: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigListFilter {
    pub page: PageRequest,
    pub config_name: Option<String>,
    pub config_key: Option<String>,
    pub config_type: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[async_trait]
pub trait SystemUseCase: Send + Sync + 'static {
    async fn page_depts(&self, filter: DeptListFilter) -> SystemResult<Page<Dept>>;
    async fn page_depts_scoped(&self, filter: DeptListFilter, scope: DataScopeFilter) -> SystemResult<Page<Dept>>;
    async fn get_dept(&self, id: &str) -> SystemResult<Dept>;
    async fn dept_tree(&self, filter: DeptListFilter, scope: Option<DataScopeFilter>) -> SystemResult<Vec<TreeSelectNode>>;
    async fn exclude_dept_tree(&self, id: &str) -> SystemResult<Vec<TreeSelectNode>>;
    async fn create_dept(&self, input: DeptInput) -> SystemResult<Dept>;
    async fn replace_dept(&self, id: &str, input: DeptInput) -> SystemResult<Dept>;
    async fn update_dept_sort(&self, id: &str, order_num: i64) -> SystemResult<Dept>;
    async fn update_dept_sorts(&self, input: SortBatchInput) -> SystemResult<Vec<Dept>>;
    async fn delete_dept(&self, id: &str) -> SystemResult<()>;
    async fn page_posts(&self, filter: PostListFilter) -> SystemResult<Page<Post>>;
    async fn get_post(&self, id: &str) -> SystemResult<Post>;
    async fn post_options(&self) -> SystemResult<Vec<Post>>;
    async fn create_post(&self, input: PostInput) -> SystemResult<Post>;
    async fn replace_post(&self, id: &str, input: PostInput) -> SystemResult<Post>;
    async fn delete_post(&self, id: &str) -> SystemResult<()>;
    async fn delete_posts(&self, ids: Vec<String>) -> SystemResult<()>;
    async fn page_dict_types(&self, filter: DictTypeListFilter) -> SystemResult<Page<DictType>>;
    async fn get_dict_type(&self, id: &str) -> SystemResult<DictType>;
    async fn dict_type_options(&self) -> SystemResult<Vec<DictType>>;
    async fn create_dict_type(&self, input: DictTypeInput) -> SystemResult<DictType>;
    async fn replace_dict_type(&self, id: &str, input: DictTypeInput) -> SystemResult<DictType>;
    async fn delete_dict_type(&self, id: &str) -> SystemResult<()>;
    async fn delete_dict_types(&self, ids: Vec<String>) -> SystemResult<()>;
    async fn refresh_dict_cache(&self) -> SystemResult<()>;
    async fn page_dict_data(&self, filter: DictDataListFilter) -> SystemResult<Page<DictData>>;
    async fn get_dict_data(&self, id: &str) -> SystemResult<DictData>;
    async fn dict_data_by_type(&self, dict_type: &str) -> SystemResult<Vec<DictData>>;
    async fn create_dict_data(&self, input: DictDataInput) -> SystemResult<DictData>;
    async fn replace_dict_data(&self, id: &str, input: DictDataInput) -> SystemResult<DictData>;
    async fn delete_dict_data(&self, id: &str) -> SystemResult<()>;
    async fn delete_dict_data_batch(&self, ids: Vec<String>) -> SystemResult<()>;
    async fn page_configs(&self, filter: ConfigListFilter) -> SystemResult<Page<ConfigItem>>;
    async fn get_config(&self, id: &str) -> SystemResult<ConfigItem>;
    async fn config_by_key(&self, key: &str) -> SystemResult<String>;
    async fn public_configs(&self, keys: Vec<String>) -> SystemResult<BTreeMap<String, String>>;
    async fn create_config(&self, input: ConfigInput) -> SystemResult<ConfigItem>;
    async fn replace_config(&self, id: &str, input: ConfigInput) -> SystemResult<ConfigItem>;
    async fn delete_config(&self, id: &str) -> SystemResult<()>;
    async fn delete_configs(&self, ids: Vec<String>) -> SystemResult<()>;
    async fn refresh_config_cache(&self) -> SystemResult<()>;
}

#[async_trait]
pub trait SystemRepository: Send + Sync + 'static {
    async fn page_depts(&self, filter: DeptListFilter) -> SystemResult<Page<Dept>>;
    async fn page_depts_scoped(&self, filter: DeptListFilter, scope: DataScopeFilter) -> SystemResult<Page<Dept>>;
    async fn list_depts(&self, filter: DeptListFilter) -> SystemResult<Vec<Dept>>;
    async fn list_depts_scoped(&self, filter: DeptListFilter, scope: DataScopeFilter) -> SystemResult<Vec<Dept>>;
    async fn list_depts_excluding(&self, id: &str) -> SystemResult<Vec<Dept>>;
    async fn find_dept(&self, id: &str) -> SystemResult<Option<Dept>>;
    async fn create_dept(&self, input: DeptInput) -> SystemResult<Dept>;
    async fn replace_dept(&self, id: &str, input: DeptInput) -> SystemResult<Dept>;
    async fn update_dept_sort(&self, id: &str, order_num: i64) -> SystemResult<Dept>;
    async fn delete_dept(&self, id: &str) -> SystemResult<()>;
    async fn dept_has_children(&self, id: &str) -> SystemResult<bool>;
    async fn dept_has_users(&self, id: &str) -> SystemResult<bool>;
    async fn dept_has_normal_children(&self, id: &str) -> SystemResult<bool>;
    async fn page_posts(&self, filter: PostListFilter) -> SystemResult<Page<Post>>;
    async fn find_post(&self, id: &str) -> SystemResult<Option<Post>>;
    async fn post_options(&self) -> SystemResult<Vec<Post>>;
    async fn post_code_exists(&self, code: &str, current_id: Option<&str>) -> SystemResult<bool>;
    async fn post_name_exists(&self, name: &str, current_id: Option<&str>) -> SystemResult<bool>;
    async fn create_post(&self, input: PostInput) -> SystemResult<Post>;
    async fn replace_post(&self, id: &str, input: PostInput) -> SystemResult<Post>;
    async fn delete_post(&self, id: &str) -> SystemResult<()>;
    async fn delete_posts(&self, ids: &[String]) -> SystemResult<()>;
    async fn post_has_users(&self, id: &str) -> SystemResult<bool>;
    async fn page_dict_types(&self, filter: DictTypeListFilter) -> SystemResult<Page<DictType>>;
    async fn list_dict_types(&self, filter: DictTypeListFilter) -> SystemResult<Vec<DictType>>;
    async fn find_dict_type(&self, id: &str) -> SystemResult<Option<DictType>>;
    async fn dict_type_options(&self) -> SystemResult<Vec<DictType>>;
    async fn dict_type_has_data(&self, dict_type: &str) -> SystemResult<bool>;
    async fn create_dict_type(&self, input: DictTypeInput) -> SystemResult<DictType>;
    async fn replace_dict_type(&self, id: &str, input: DictTypeInput) -> SystemResult<DictType>;
    async fn delete_dict_type(&self, id: &str) -> SystemResult<()>;
    async fn delete_dict_types(&self, ids: &[String]) -> SystemResult<()>;
    async fn page_dict_data(&self, filter: DictDataListFilter) -> SystemResult<Page<DictData>>;
    async fn find_dict_data(&self, id: &str) -> SystemResult<Option<DictData>>;
    async fn dict_data_by_type(&self, dict_type: &str) -> SystemResult<Vec<DictData>>;
    async fn create_dict_data(&self, input: DictDataInput) -> SystemResult<DictData>;
    async fn replace_dict_data(&self, id: &str, input: DictDataInput) -> SystemResult<DictData>;
    async fn delete_dict_data(&self, id: &str) -> SystemResult<()>;
    async fn delete_dict_data_batch(&self, ids: &[String]) -> SystemResult<()>;
    async fn page_configs(&self, filter: ConfigListFilter) -> SystemResult<Page<ConfigItem>>;
    async fn list_configs(&self, filter: ConfigListFilter) -> SystemResult<Vec<ConfigItem>>;
    async fn find_config(&self, id: &str) -> SystemResult<Option<ConfigItem>>;
    async fn find_config_by_key(&self, key: &str) -> SystemResult<Option<ConfigItem>>;
    async fn config_by_key(&self, key: &str) -> SystemResult<Option<String>>;
    async fn create_config(&self, input: ConfigInput) -> SystemResult<ConfigItem>;
    async fn replace_config(&self, id: &str, input: ConfigInput) -> SystemResult<ConfigItem>;
    async fn delete_config(&self, id: &str) -> SystemResult<()>;
    async fn delete_configs(&self, ids: &[String]) -> SystemResult<()>;
}

#[async_trait]
pub trait SystemCache: Send + Sync + 'static {
    async fn read_config(&self, key: &str) -> SystemResult<Option<String>>;
    async fn write_config(&self, item: &ConfigItem) -> SystemResult<()>;
    async fn clear_configs(&self) -> SystemResult<()>;
    async fn read_dict_data(&self, dict_type: &str) -> SystemResult<Option<Vec<DictData>>>;
    async fn write_dict_data(&self, dict_type: &str, items: &[DictData]) -> SystemResult<()>;
    async fn clear_dicts(&self) -> SystemResult<()>;
}

#[derive(Clone, Copy)]
pub struct NoSystemCache;

#[async_trait]
impl SystemCache for NoSystemCache {
    async fn read_config(&self, _key: &str) -> SystemResult<Option<String>> {
        Ok(None)
    }
    async fn write_config(&self, _item: &ConfigItem) -> SystemResult<()> {
        Ok(())
    }
    async fn clear_configs(&self) -> SystemResult<()> {
        Ok(())
    }
    async fn read_dict_data(&self, _dict_type: &str) -> SystemResult<Option<Vec<DictData>>> {
        Ok(None)
    }
    async fn write_dict_data(&self, _dict_type: &str, _items: &[DictData]) -> SystemResult<()> {
        Ok(())
    }
    async fn clear_dicts(&self) -> SystemResult<()> {
        Ok(())
    }
}

pub struct SystemService<R, C = NoSystemCache> {
    repository: R,
    cache: C,
}

impl<R: SystemRepository> SystemService<R> {
    pub const fn new(repository: R) -> Self {
        Self {
            repository,
            cache: NoSystemCache,
        }
    }
}

impl<R: SystemRepository, C: SystemCache> SystemService<R, C> {
    pub const fn with_cache(repository: R, cache: C) -> Self {
        Self { repository, cache }
    }
}

#[async_trait]
impl<R: SystemRepository, C: SystemCache> SystemUseCase for SystemService<R, C> {
    async fn page_depts(&self, filter: DeptListFilter) -> SystemResult<Page<Dept>> {
        validate_page(filter.page)?;
        self.repository.page_depts(sanitize_dept_filter(filter)).await
    }

    async fn page_depts_scoped(&self, filter: DeptListFilter, scope: DataScopeFilter) -> SystemResult<Page<Dept>> {
        validate_page(filter.page)?;
        self.repository.page_depts_scoped(sanitize_dept_filter(filter), scope).await
    }

    async fn get_dept(&self, id: &str) -> SystemResult<Dept> {
        self.repository.find_dept(id).await?.ok_or(SystemError::NotFound)
    }

    async fn dept_tree(&self, filter: DeptListFilter, scope: Option<DataScopeFilter>) -> SystemResult<Vec<TreeSelectNode>> {
        let filter = sanitize_dept_filter(filter);
        let depts = match scope {
            Some(scope) => self.repository.list_depts_scoped(filter, scope).await?,
            None => self.repository.list_depts(filter).await?,
        };
        Ok(dept_tree(depts))
    }

    async fn exclude_dept_tree(&self, id: &str) -> SystemResult<Vec<TreeSelectNode>> {
        self.get_dept(id).await?;
        Ok(dept_tree(self.repository.list_depts_excluding(id).await?))
    }

    async fn create_dept(&self, input: DeptInput) -> SystemResult<Dept> {
        reject_duplicate_dept(&self.repository, &input, None).await?;
        self.repository.create_dept(input).await
    }

    async fn replace_dept(&self, id: &str, input: DeptInput) -> SystemResult<Dept> {
        reject_invalid_dept_parent(id, &input)?;
        reject_duplicate_dept(&self.repository, &input, Some(id)).await?;
        if input.status != types::rbac::STATUS_NORMAL && self.repository.dept_has_normal_children(id).await? {
            return Err(SystemError::Conflict("department has active children".into()));
        }
        self.repository.replace_dept(id, input).await
    }

    async fn update_dept_sort(&self, id: &str, order_num: i64) -> SystemResult<Dept> {
        self.repository.update_dept_sort(id, order_num).await
    }

    async fn update_dept_sorts(&self, input: SortBatchInput) -> SystemResult<Vec<Dept>> {
        let mut items = Vec::with_capacity(input.items.len());
        for item in input.items {
            items.push(self.update_dept_sort(&item.id, item.order_num).await?);
        }
        Ok(items)
    }

    async fn delete_dept(&self, id: &str) -> SystemResult<()> {
        reject_dept_delete(&self.repository, id).await?;
        self.repository.delete_dept(id).await
    }

    async fn page_posts(&self, filter: PostListFilter) -> SystemResult<Page<Post>> {
        validate_page(filter.page)?;
        self.repository.page_posts(sanitize_post_filter(filter)).await
    }

    async fn get_post(&self, id: &str) -> SystemResult<Post> {
        self.repository.find_post(id).await?.ok_or(SystemError::NotFound)
    }

    async fn post_options(&self) -> SystemResult<Vec<Post>> {
        self.repository.post_options().await
    }

    async fn create_post(&self, input: PostInput) -> SystemResult<Post> {
        reject_duplicate_post(&self.repository, &input, None).await?;
        self.repository.create_post(input).await
    }

    async fn replace_post(&self, id: &str, input: PostInput) -> SystemResult<Post> {
        reject_duplicate_post(&self.repository, &input, Some(id)).await?;
        self.repository.replace_post(id, input).await
    }

    async fn delete_post(&self, id: &str) -> SystemResult<()> {
        reject_post_delete(&self.repository, id).await?;
        self.repository.delete_post(id).await
    }

    async fn delete_posts(&self, ids: Vec<String>) -> SystemResult<()> {
        reject_empty_ids(&ids)?;
        for id in &ids {
            reject_post_delete(&self.repository, id).await?;
        }
        self.repository.delete_posts(&ids).await
    }

    async fn page_dict_types(&self, filter: DictTypeListFilter) -> SystemResult<Page<DictType>> {
        validate_page(filter.page)?;
        self.repository.page_dict_types(sanitize_dict_type_filter(filter)).await
    }

    async fn get_dict_type(&self, id: &str) -> SystemResult<DictType> {
        self.repository.find_dict_type(id).await?.ok_or(SystemError::NotFound)
    }

    async fn dict_type_options(&self) -> SystemResult<Vec<DictType>> {
        self.repository.dict_type_options().await
    }

    async fn create_dict_type(&self, input: DictTypeInput) -> SystemResult<DictType> {
        reject_duplicate_dict_type(&self.repository, &input, None).await?;
        let item = self.repository.create_dict_type(input).await?;
        self.refresh_dict_cache().await?;
        Ok(item)
    }

    async fn replace_dict_type(&self, id: &str, input: DictTypeInput) -> SystemResult<DictType> {
        reject_duplicate_dict_type(&self.repository, &input, Some(id)).await?;
        let item = self.repository.replace_dict_type(id, input).await?;
        self.refresh_dict_cache().await?;
        Ok(item)
    }

    async fn delete_dict_type(&self, id: &str) -> SystemResult<()> {
        let item = self.get_dict_type(id).await?;
        if self.repository.dict_type_has_data(&item.dict_type).await? {
            return Err(SystemError::Conflict("dictionary type still has data".into()));
        }
        self.repository.delete_dict_type(id).await?;
        self.refresh_dict_cache().await
    }

    async fn delete_dict_types(&self, ids: Vec<String>) -> SystemResult<()> {
        reject_empty_ids(&ids)?;
        for id in &ids {
            let item = self.get_dict_type(id).await?;
            if self.repository.dict_type_has_data(&item.dict_type).await? {
                return Err(SystemError::Conflict("dictionary type still has data".into()));
            }
        }
        self.repository.delete_dict_types(&ids).await?;
        self.refresh_dict_cache().await
    }

    async fn refresh_dict_cache(&self) -> SystemResult<()> {
        self.cache.clear_dicts().await?;
        for item in self.repository.list_dict_types(all_dict_types_filter()).await? {
            let data = self.repository.dict_data_by_type(&item.dict_type).await?;
            self.cache.write_dict_data(&item.dict_type, &data).await?;
        }
        Ok(())
    }

    async fn page_dict_data(&self, filter: DictDataListFilter) -> SystemResult<Page<DictData>> {
        validate_page(filter.page)?;
        self.repository.page_dict_data(sanitize_dict_data_filter(filter)).await
    }

    async fn get_dict_data(&self, id: &str) -> SystemResult<DictData> {
        self.repository.find_dict_data(id).await?.ok_or(SystemError::NotFound)
    }

    async fn dict_data_by_type(&self, dict_type: &str) -> SystemResult<Vec<DictData>> {
        if let Some(items) = self.cache.read_dict_data(dict_type).await? {
            return Ok(items);
        }
        let items = self.repository.dict_data_by_type(dict_type).await?;
        self.cache.write_dict_data(dict_type, &items).await?;
        Ok(items)
    }

    async fn create_dict_data(&self, input: DictDataInput) -> SystemResult<DictData> {
        let item = self.repository.create_dict_data(input).await?;
        self.refresh_dict_cache().await?;
        Ok(item)
    }

    async fn replace_dict_data(&self, id: &str, input: DictDataInput) -> SystemResult<DictData> {
        let item = self.repository.replace_dict_data(id, input).await?;
        self.refresh_dict_cache().await?;
        Ok(item)
    }

    async fn delete_dict_data(&self, id: &str) -> SystemResult<()> {
        self.repository.delete_dict_data(id).await?;
        self.refresh_dict_cache().await
    }

    async fn delete_dict_data_batch(&self, ids: Vec<String>) -> SystemResult<()> {
        reject_empty_ids(&ids)?;
        self.repository.delete_dict_data_batch(&ids).await?;
        self.refresh_dict_cache().await
    }

    async fn page_configs(&self, filter: ConfigListFilter) -> SystemResult<Page<ConfigItem>> {
        validate_page(filter.page)?;
        self.repository.page_configs(sanitize_config_filter(filter)).await
    }

    async fn get_config(&self, id: &str) -> SystemResult<ConfigItem> {
        self.repository.find_config(id).await?.ok_or(SystemError::NotFound)
    }

    async fn config_by_key(&self, key: &str) -> SystemResult<String> {
        if let Some(value) = self.cache.read_config(key).await? {
            return Ok(value);
        }
        let value = self.repository.config_by_key(key).await?.ok_or(SystemError::NotFound)?;
        self.cache
            .write_config(&ConfigItem {
                config_id: String::new(),
                config_name: String::new(),
                config_key: key.into(),
                config_value: value.clone(),
                config_type: String::new(),
                public_read: false,
                remark: None,
                create_time: String::new(),
            })
            .await?;
        Ok(value)
    }

    async fn public_configs(&self, keys: Vec<String>) -> SystemResult<BTreeMap<String, String>> {
        let keys = clean_config_keys(keys)?;
        let mut values = BTreeMap::new();
        for key in keys {
            let item = self.repository.find_config_by_key(&key).await?.ok_or(SystemError::NotFound)?;
            if !item.public_read {
                return Err(SystemError::Forbidden(format!("config key {key} is not public")));
            }
            values.insert(item.config_key, item.config_value);
        }
        Ok(values)
    }

    async fn create_config(&self, input: ConfigInput) -> SystemResult<ConfigItem> {
        reject_sensitive_public_config(&input.config_key, input.public_read)?;
        reject_duplicate_config_key(&self.repository, &input, None).await?;
        let item = self.repository.create_config(input).await?;
        self.cache.write_config(&item).await?;
        Ok(item)
    }

    async fn replace_config(&self, id: &str, input: ConfigInput) -> SystemResult<ConfigItem> {
        let current = self.get_config(id).await?;
        reject_builtin_config_identity_change(&current, &input)?;
        reject_sensitive_public_config(&input.config_key, input.public_read)?;
        reject_duplicate_config_key(&self.repository, &input, Some(id)).await?;
        let item = self.repository.replace_config(id, input).await?;
        self.refresh_config_cache().await?;
        Ok(item)
    }

    async fn delete_config(&self, id: &str) -> SystemResult<()> {
        reject_builtin_config_delete(&self.get_config(id).await?)?;
        self.repository.delete_config(id).await?;
        self.refresh_config_cache().await
    }

    async fn delete_configs(&self, ids: Vec<String>) -> SystemResult<()> {
        reject_empty_ids(&ids)?;
        for id in &ids {
            reject_builtin_config_delete(&self.get_config(id).await?)?;
        }
        self.repository.delete_configs(&ids).await?;
        self.refresh_config_cache().await
    }

    async fn refresh_config_cache(&self) -> SystemResult<()> {
        self.cache.clear_configs().await?;
        for item in self.repository.list_configs(all_configs_filter()).await? {
            self.cache.write_config(&item).await?;
        }
        Ok(())
    }
}

fn all_configs_filter() -> ConfigListFilter {
    ConfigListFilter {
        page: PageRequest { page: 1, page_size: 100_000 },
        config_name: None,
        config_key: None,
        config_type: None,
        begin_time: None,
        end_time: None,
    }
}

fn all_dict_types_filter() -> DictTypeListFilter {
    DictTypeListFilter {
        page: PageRequest { page: 1, page_size: 100_000 },
        dict_name: None,
        dict_type: None,
        status: None,
        begin_time: None,
        end_time: None,
    }
}

async fn reject_duplicate_dept<R: SystemRepository>(repository: &R, input: &DeptInput, current_id: Option<&str>) -> SystemResult<()> {
    let depts = repository
        .list_depts(DeptListFilter {
            page: PageRequest { page: 1, page_size: 100_000 },
            dept_name: None,
            status: None,
            begin_time: None,
            end_time: None,
        })
        .await?;
    if depts
        .iter()
        .any(|dept| dept.parent_id == input.parent_id && dept.dept_name == input.dept_name && Some(dept.dept_id.as_str()) != current_id)
    {
        return Err(SystemError::Conflict("department name already exists".into()));
    }
    Ok(())
}

fn reject_invalid_dept_parent(id: &str, input: &DeptInput) -> SystemResult<()> {
    if input.parent_id == id {
        return Err(SystemError::Conflict("department parent cannot be itself".into()));
    }
    Ok(())
}

async fn reject_duplicate_dict_type<R: SystemRepository>(repository: &R, input: &DictTypeInput, current_id: Option<&str>) -> SystemResult<()> {
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
        return Err(SystemError::Conflict("dictionary type already exists".into()));
    }
    Ok(())
}

async fn reject_duplicate_config_key<R: SystemRepository>(repository: &R, input: &ConfigInput, current_id: Option<&str>) -> SystemResult<()> {
    let items = repository
        .page_configs(ConfigListFilter {
            page: PageRequest { page: 1, page_size: 100_000 },
            config_name: None,
            config_key: Some(input.config_key.clone()),
            config_type: None,
            begin_time: None,
            end_time: None,
        })
        .await?;
    if items
        .items
        .iter()
        .any(|item| item.config_key == input.config_key && Some(item.config_id.as_str()) != current_id)
    {
        return Err(SystemError::Conflict("config key already exists".into()));
    }
    Ok(())
}

fn clean_config_keys(keys: Vec<String>) -> SystemResult<Vec<String>> {
    let keys = keys
        .into_iter()
        .map(|key| key.trim().to_owned())
        .filter(|key| !key.is_empty())
        .collect::<Vec<_>>();
    if keys.is_empty() {
        return Err(SystemError::InvalidInput("keys are required".into()));
    }
    Ok(keys)
}

fn reject_builtin_config_delete(item: &ConfigItem) -> SystemResult<()> {
    if item.config_type == "Y" {
        return Err(SystemError::Conflict("built-in config cannot be deleted".into()));
    }
    Ok(())
}

fn reject_builtin_config_identity_change(current: &ConfigItem, input: &ConfigInput) -> SystemResult<()> {
    if current.config_type == "Y" && current.config_key != input.config_key {
        return Err(SystemError::Conflict("built-in config key cannot be changed".into()));
    }
    if current.config_type == "Y" && input.config_type != "Y" {
        return Err(SystemError::Conflict("built-in config type cannot be changed".into()));
    }
    Ok(())
}

fn reject_sensitive_public_config(key: &str, public_read: bool) -> SystemResult<()> {
    if key == "sys.user.initPassword" && public_read {
        return Err(SystemError::Conflict("initial password config cannot be public".into()));
    }
    if key == "sys.account.captchaPrivateConfig" && public_read {
        return Err(SystemError::Conflict("captcha private config cannot be public".into()));
    }
    Ok(())
}

async fn reject_dept_delete<R: SystemRepository>(repository: &R, id: &str) -> SystemResult<()> {
    if repository.dept_has_children(id).await? || repository.dept_has_users(id).await? {
        return Err(SystemError::Conflict("department still has children or users".into()));
    }
    Ok(())
}

async fn reject_post_delete<R: SystemRepository>(repository: &R, id: &str) -> SystemResult<()> {
    if repository.post_has_users(id).await? {
        return Err(SystemError::Conflict("post is still assigned to users".into()));
    }
    Ok(())
}

async fn reject_duplicate_post<R: SystemRepository>(repository: &R, input: &PostInput, current_id: Option<&str>) -> SystemResult<()> {
    if repository.post_code_exists(&input.post_code, current_id).await? {
        return Err(SystemError::Conflict("post code already exists".into()));
    }
    if repository.post_name_exists(&input.post_name, current_id).await? {
        return Err(SystemError::Conflict("post name already exists".into()));
    }
    Ok(())
}

fn validate_page(page: PageRequest) -> SystemResult<()> {
    if page.page == 0 || page.page_size == 0 {
        return Err(SystemError::InvalidInput("page and page_size must be greater than 0".into()));
    }
    Ok(())
}

fn sanitize_dept_filter(input: DeptListFilter) -> DeptListFilter {
    DeptListFilter {
        page: input.page,
        dept_name: trim(input.dept_name),
        status: trim(input.status),
        begin_time: trim(input.begin_time),
        end_time: trim(input.end_time),
    }
}

fn sanitize_post_filter(input: PostListFilter) -> PostListFilter {
    PostListFilter {
        page: input.page,
        post_code: trim(input.post_code),
        post_name: trim(input.post_name),
        status: trim(input.status),
    }
}

fn sanitize_dict_type_filter(input: DictTypeListFilter) -> DictTypeListFilter {
    DictTypeListFilter {
        page: input.page,
        dict_name: trim(input.dict_name),
        dict_type: trim(input.dict_type),
        status: trim(input.status),
        begin_time: trim(input.begin_time),
        end_time: trim(input.end_time),
    }
}

fn sanitize_dict_data_filter(input: DictDataListFilter) -> DictDataListFilter {
    DictDataListFilter {
        page: input.page,
        dict_type: trim(input.dict_type),
        dict_label: trim(input.dict_label),
        status: trim(input.status),
    }
}

fn sanitize_config_filter(input: ConfigListFilter) -> ConfigListFilter {
    ConfigListFilter {
        page: input.page,
        config_name: trim(input.config_name),
        config_key: trim(input.config_key),
        config_type: trim(input.config_type),
        begin_time: trim(input.begin_time),
        end_time: trim(input.end_time),
    }
}

fn trim(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().into()).filter(|item: &String| !item.is_empty())
}

fn dept_tree(depts: Vec<Dept>) -> Vec<TreeSelectNode> {
    depts.iter().filter(|dept| is_root(dept, &depts)).map(|dept| dept_node(dept, &depts)).collect()
}

fn dept_node(dept: &Dept, depts: &[Dept]) -> TreeSelectNode {
    TreeSelectNode {
        id: dept.dept_id.clone(),
        label: dept.dept_name.clone(),
        parent_id: dept.parent_id.clone(),
        disabled: dept.status != types::rbac::STATUS_NORMAL,
        children: depts
            .iter()
            .filter(|child| child.parent_id == dept.dept_id)
            .map(|child| dept_node(child, depts))
            .collect(),
    }
}

fn is_root(dept: &Dept, depts: &[Dept]) -> bool {
    dept.parent_id == "0" || !depts.iter().any(|item| item.dept_id == dept.parent_id)
}

fn reject_empty_ids(ids: &[String]) -> SystemResult<()> {
    if ids.is_empty() {
        return Err(SystemError::InvalidInput("ids are required".into()));
    }
    Ok(())
}
