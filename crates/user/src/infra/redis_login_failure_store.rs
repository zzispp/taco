use async_trait::async_trait;
use redis::AsyncCommands;
use taco_tracing::{InfrastructureDependency, InfrastructureObserver};

use crate::{
    application::{AppError, AppResult, LoginFailureStore},
    domain::UserId,
};

const INCREMENT_WITH_TTL: &str = r#"
local count = redis.call('INCR', KEYS[1])
redis.call('EXPIRE', KEYS[1], ARGV[1])
return count
"#;

#[derive(Clone)]
pub struct RedisLoginFailureStore {
    connection: redis::aio::ConnectionManager,
    key_prefix: String,
    observer: InfrastructureObserver,
}

impl RedisLoginFailureStore {
    pub async fn connect(url: &str, key_prefix: String, observer: InfrastructureObserver) -> AppResult<Self> {
        let started = std::time::Instant::now();
        let client = redis::Client::open(url);
        observer.record(InfrastructureDependency::Redis, "login_failure_connect", started.elapsed(), client.is_ok());
        let client = client.map_err(redis_error)?;
        let connection = observer
            .observe(InfrastructureDependency::Redis, "login_failure_connect", client.get_connection_manager())
            .await
            .map_err(redis_error)?;
        Ok(Self {
            connection,
            key_prefix,
            observer,
        })
    }

    fn failure_key(&self, user_id: &UserId) -> String {
        format!("{}:auth:password-failures:{}", self.key_prefix, user_id.0)
    }
}

#[async_trait]
impl LoginFailureStore for RedisLoginFailureStore {
    async fn failure_count(&self, user_id: &UserId) -> AppResult<u32> {
        let mut connection = self.connection.clone();
        let count: Option<u32> = self
            .observer
            .observe(InfrastructureDependency::Redis, "login_failure_read", connection.get(self.failure_key(user_id)))
            .await
            .map_err(redis_error)?;
        Ok(count.unwrap_or(0))
    }

    async fn record_failure(&self, user_id: &UserId, ttl_seconds: u64) -> AppResult<u32> {
        let mut connection = self.connection.clone();
        self.observer
            .observe(
                InfrastructureDependency::Redis,
                "login_failure_increment",
                redis::Script::new(INCREMENT_WITH_TTL)
                    .key(self.failure_key(user_id))
                    .arg(ttl_seconds)
                    .invoke_async(&mut connection),
            )
            .await
            .map_err(redis_error)
    }

    async fn clear_failures(&self, user_id: &UserId) -> AppResult<()> {
        let mut connection = self.connection.clone();
        self.observer
            .observe(
                InfrastructureDependency::Redis,
                "login_failure_clear",
                connection.del::<_, ()>(self.failure_key(user_id)),
            )
            .await
            .map_err(redis_error)
    }
}

fn redis_error(error: redis::RedisError) -> AppError {
    AppError::Infrastructure(format!("redis login failure store error: {error}"))
}
