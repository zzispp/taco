use async_trait::async_trait;
use redis::AsyncCommands;
use taco_tracing::{InfrastructureDependency, InfrastructureObserver};

use crate::{
    application::{SystemCache, SystemError, SystemResult},
    domain::{ConfigItem, DictData},
};

#[derive(Clone)]
pub struct RedisSystemCache {
    connection: redis::aio::ConnectionManager,
    key_prefix: String,
    observer: InfrastructureObserver,
}

impl RedisSystemCache {
    pub async fn connect(url: &str, key_prefix: String, observer: InfrastructureObserver) -> SystemResult<Self> {
        let started = std::time::Instant::now();
        let client = redis::Client::open(url);
        observer.record(InfrastructureDependency::Redis, "system_cache_connect", started.elapsed(), client.is_ok());
        let client = client.map_err(redis_error)?;
        let connection = observer
            .observe(InfrastructureDependency::Redis, "system_cache_connect", client.get_connection_manager())
            .await
            .map_err(redis_error)?;
        Ok(Self {
            connection,
            key_prefix,
            observer,
        })
    }

    fn config_key(&self, key: &str) -> String {
        format!("{}:system:config:{key}", self.key_prefix)
    }

    fn dict_key(&self, dict_type: &str) -> String {
        format!("{}:system:dict:{dict_type}", self.key_prefix)
    }

    fn config_pattern(&self) -> String {
        format!("{}:system:config:*", self.key_prefix)
    }

    fn dict_pattern(&self) -> String {
        format!("{}:system:dict:*", self.key_prefix)
    }
}

#[async_trait]
impl SystemCache for RedisSystemCache {
    async fn read_config(&self, key: &str) -> SystemResult<Option<String>> {
        let mut connection = self.connection.clone();
        self.observer
            .observe(
                InfrastructureDependency::Redis,
                "system_cache_read_config",
                connection.get(self.config_key(key)),
            )
            .await
            .map_err(redis_error)
    }

    async fn write_config(&self, item: &ConfigItem) -> SystemResult<()> {
        let mut connection = self.connection.clone();
        self.observer
            .observe(
                InfrastructureDependency::Redis,
                "system_cache_write_config",
                connection.set(self.config_key(&item.config_key), &item.config_value),
            )
            .await
            .map_err(redis_error)
    }

    async fn clear_configs(&self) -> SystemResult<()> {
        clear_keys(&self.observer, self.connection.clone(), self.config_pattern()).await
    }

    async fn read_dict_data(&self, dict_type: &str) -> SystemResult<Option<Vec<DictData>>> {
        let mut connection = self.connection.clone();
        let value: Option<String> = self
            .observer
            .observe(
                InfrastructureDependency::Redis,
                "system_cache_read_dict",
                connection.get(self.dict_key(dict_type)),
            )
            .await
            .map_err(redis_error)?;
        value.map(|item| serde_json::from_str(&item).map_err(json_error)).transpose()
    }

    async fn write_dict_data(&self, dict_type: &str, items: &[DictData]) -> SystemResult<()> {
        let mut connection = self.connection.clone();
        let value = serde_json::to_string(items).map_err(json_error)?;
        self.observer
            .observe(
                InfrastructureDependency::Redis,
                "system_cache_write_dict",
                connection.set(self.dict_key(dict_type), value),
            )
            .await
            .map_err(redis_error)
    }

    async fn clear_dicts(&self) -> SystemResult<()> {
        clear_keys(&self.observer, self.connection.clone(), self.dict_pattern()).await
    }
}

async fn clear_keys(observer: &InfrastructureObserver, mut connection: redis::aio::ConnectionManager, pattern: String) -> SystemResult<()> {
    let keys: Vec<String> = observer
        .observe(InfrastructureDependency::Redis, "system_cache_list_keys", connection.keys(pattern))
        .await
        .map_err(redis_error)?;
    if keys.is_empty() {
        return Ok(());
    }
    observer
        .observe(InfrastructureDependency::Redis, "system_cache_delete_keys", connection.del(keys))
        .await
        .map_err(redis_error)
}

fn redis_error(error: redis::RedisError) -> SystemError {
    SystemError::Infrastructure(format!("redis error: {error}"))
}

fn json_error(error: serde_json::Error) -> SystemError {
    SystemError::Infrastructure(format!("system cache json error: {error}"))
}
