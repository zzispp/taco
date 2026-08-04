use std::collections::BTreeMap;

use kernel::pagination::CursorPage;

use crate::{
    application::{ConfigListFilter, SystemCache, SystemCursorCodec, SystemError, SystemRepository, SystemResult},
    domain::{ConfigInput, ConfigItem},
};

use super::{SystemService, validation::*};

impl<R: SystemRepository, C: SystemCache> SystemService<R, C> {
    pub(super) async fn page_configs_impl(&self, filter: ConfigListFilter) -> SystemResult<CursorPage<ConfigItem>> {
        let filter = sanitize_config_filter(filter);
        validate_page(filter.page.clone())?;
        SystemCursorCodec::config(&filter)?.decode(&filter.page)?;
        self.repository.page_configs(filter).await
    }

    pub(super) async fn get_config_impl(&self, id: &str) -> SystemResult<ConfigItem> {
        self.repository.find_config(id).await?.ok_or(SystemError::NotFound)
    }

    pub(super) async fn config_by_key_impl(&self, key: &str) -> SystemResult<String> {
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

    pub(super) async fn public_configs_impl(&self, keys: Vec<String>) -> SystemResult<BTreeMap<String, String>> {
        let keys = clean_config_keys(keys)?;
        let mut values = BTreeMap::new();
        for key in keys {
            let item = self.repository.find_config_by_key(&key).await?.ok_or(SystemError::NotFound)?;
            if !item.public_read {
                return Err(SystemError::Forbidden(localized_param("errors.system.config_not_public", "key", key)));
            }
            values.insert(item.config_key, item.config_value);
        }
        Ok(values)
    }

    pub(super) async fn create_config_impl(&self, input: ConfigInput) -> SystemResult<ConfigItem> {
        validate_runtime_config(&input)?;
        reject_sensitive_public_config(&input.config_key, input.public_read)?;
        reject_duplicate_config_key(&self.repository, &input, None).await?;
        let item = self.repository.create_config(input).await?;
        self.cache.write_config(&item).await?;
        Ok(item)
    }

    pub(super) async fn replace_config_impl(&self, id: &str, input: ConfigInput) -> SystemResult<ConfigItem> {
        let current = self.get_config_impl(id).await?;
        reject_builtin_config_identity_change(&current, &input)?;
        validate_runtime_config(&input)?;
        reject_sensitive_public_config(&input.config_key, input.public_read)?;
        reject_duplicate_config_key(&self.repository, &input, Some(id)).await?;
        let item = self.repository.replace_config(id, input).await?;
        self.refresh_config_cache_impl().await?;
        Ok(item)
    }

    pub(super) async fn delete_config_impl(&self, id: &str) -> SystemResult<()> {
        reject_builtin_config_delete(&self.get_config_impl(id).await?)?;
        self.repository.delete_config(id).await?;
        self.refresh_config_cache_impl().await
    }

    pub(super) async fn delete_configs_impl(&self, ids: Vec<String>) -> SystemResult<()> {
        reject_empty_ids(&ids)?;
        for id in &ids {
            reject_builtin_config_delete(&self.get_config_impl(id).await?)?;
        }
        self.repository.delete_configs(&ids).await?;
        self.refresh_config_cache_impl().await
    }

    pub(super) async fn refresh_config_cache_impl(&self) -> SystemResult<()> {
        self.refresh_config_cache_after_write().await
    }
}
