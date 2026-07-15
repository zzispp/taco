use crate::application::{SystemCache, SystemRepository, SystemResult};

use super::{
    SystemService,
    validation::{all_configs_filter, all_dict_types_filter},
};

impl<R: SystemRepository, C: SystemCache> SystemService<R, C> {
    pub(super) async fn refresh_config_cache_after_write(&self) -> SystemResult<()> {
        self.cache.clear_configs().await?;
        for item in self.repository.list_configs(all_configs_filter()).await? {
            self.cache.write_config(&item).await?;
        }
        Ok(())
    }

    pub(super) async fn refresh_dict_cache_after_write(&self) -> SystemResult<()> {
        self.cache.clear_dicts().await?;
        for item in self.repository.list_dict_types(all_dict_types_filter()).await? {
            let data = self.repository.dict_data_by_type(&item.dict_type).await?;
            self.cache.write_dict_data(&item.dict_type, &data).await?;
        }
        Ok(())
    }
}
