use kernel::pagination::CursorPage;

use crate::{
    application::{DictDataListFilter, DictTypeListFilter, SystemCache, SystemCursorCodec, SystemError, SystemRepository, SystemResult},
    domain::{DictData, DictDataInput, DictType, DictTypeInput},
};

use super::{SystemService, validation::*};

impl<R: SystemRepository, C: SystemCache> SystemService<R, C> {
    pub(super) async fn page_dict_types_impl(&self, filter: DictTypeListFilter) -> SystemResult<CursorPage<DictType>> {
        let filter = sanitize_dict_type_filter(filter);
        validate_page(filter.page.clone())?;
        SystemCursorCodec::dict_type(&filter)?.decode(&filter.page)?;
        self.repository.page_dict_types(filter).await
    }

    pub(super) async fn get_dict_type_impl(&self, id: &str) -> SystemResult<DictType> {
        self.repository.find_dict_type(id).await?.ok_or(SystemError::NotFound)
    }

    pub(super) async fn dict_type_options_impl(&self) -> SystemResult<Vec<DictType>> {
        self.repository.dict_type_options().await
    }

    pub(super) async fn create_dict_type_impl(&self, input: DictTypeInput) -> SystemResult<DictType> {
        reject_duplicate_dict_type(&self.repository, &input, None).await?;
        let item = self.repository.create_dict_type(input).await?;
        self.refresh_dict_cache_impl().await?;
        Ok(item)
    }

    pub(super) async fn replace_dict_type_impl(&self, id: &str, input: DictTypeInput) -> SystemResult<DictType> {
        reject_duplicate_dict_type(&self.repository, &input, Some(id)).await?;
        let item = self.repository.replace_dict_type(id, input).await?;
        self.refresh_dict_cache_impl().await?;
        Ok(item)
    }

    pub(super) async fn delete_dict_type_impl(&self, id: &str) -> SystemResult<()> {
        let item = self.get_dict_type_impl(id).await?;
        if self.repository.dict_type_has_data(&item.dict_type).await? {
            return Err(SystemError::Conflict(localized("errors.system.dict_type_has_data")));
        }
        self.repository.delete_dict_type(id).await?;
        self.refresh_dict_cache_impl().await
    }

    pub(super) async fn delete_dict_types_impl(&self, ids: Vec<String>) -> SystemResult<()> {
        reject_empty_ids(&ids)?;
        for id in &ids {
            let item = self.get_dict_type_impl(id).await?;
            if self.repository.dict_type_has_data(&item.dict_type).await? {
                return Err(SystemError::Conflict(localized("errors.system.dict_type_has_data")));
            }
        }
        self.repository.delete_dict_types(&ids).await?;
        self.refresh_dict_cache_impl().await
    }

    pub(super) async fn refresh_dict_cache_impl(&self) -> SystemResult<()> {
        self.refresh_dict_cache_after_write().await
    }

    pub(super) async fn page_dict_data_impl(&self, filter: DictDataListFilter) -> SystemResult<CursorPage<DictData>> {
        let filter = sanitize_dict_data_filter(filter);
        validate_page(filter.page.clone())?;
        SystemCursorCodec::dict_data(&filter)?.decode(&filter.page)?;
        self.repository.page_dict_data(filter).await
    }

    pub(super) async fn get_dict_data_impl(&self, id: &str) -> SystemResult<DictData> {
        self.repository.find_dict_data(id).await?.ok_or(SystemError::NotFound)
    }

    pub(super) async fn dict_data_by_type_impl(&self, dict_type: &str) -> SystemResult<Vec<DictData>> {
        if let Some(items) = self.cache.read_dict_data(dict_type).await? {
            return Ok(items);
        }
        let items = self.repository.dict_data_by_type(dict_type).await?;
        self.cache.write_dict_data(dict_type, &items).await?;
        Ok(items)
    }

    pub(super) async fn create_dict_data_impl(&self, input: DictDataInput) -> SystemResult<DictData> {
        let item = self.repository.create_dict_data(input).await?;
        self.refresh_dict_cache_impl().await?;
        Ok(item)
    }

    pub(super) async fn replace_dict_data_impl(&self, id: &str, input: DictDataInput) -> SystemResult<DictData> {
        let item = self.repository.replace_dict_data(id, input).await?;
        self.refresh_dict_cache_impl().await?;
        Ok(item)
    }

    pub(super) async fn delete_dict_data_impl(&self, id: &str) -> SystemResult<()> {
        self.repository.delete_dict_data(id).await?;
        self.refresh_dict_cache_impl().await
    }

    pub(super) async fn delete_dict_data_batch_impl(&self, ids: Vec<String>) -> SystemResult<()> {
        reject_empty_ids(&ids)?;
        self.repository.delete_dict_data_batch(&ids).await?;
        self.refresh_dict_cache_impl().await
    }
}
