use async_trait::async_trait;

use crate::application::{SystemCache, SystemResult};
use crate::domain::{ConfigItem, DictData};

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
