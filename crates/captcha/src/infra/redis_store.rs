use async_trait::async_trait;
use redis::AsyncCommands;
use taco_tracing::{InfrastructureDependency, InfrastructureObserver};

use crate::{
    application::{CaptchaError, CaptchaResult},
    providers::cap::{CapChallengeRecord, CapStore},
};

#[derive(Clone)]
pub struct RedisCaptchaStore {
    connection: redis::aio::ConnectionManager,
    key_prefix: String,
    observer: InfrastructureObserver,
}

impl RedisCaptchaStore {
    pub fn new(connection: redis::aio::ConnectionManager, key_prefix: String, observer: InfrastructureObserver) -> Self {
        Self {
            connection,
            key_prefix,
            observer,
        }
    }

    pub async fn connect(url: &str, key_prefix: String, observer: InfrastructureObserver) -> CaptchaResult<Self> {
        let started = std::time::Instant::now();
        let client = redis::Client::open(url);
        observer.record(InfrastructureDependency::Redis, "captcha_connect", started.elapsed(), client.is_ok());
        let client = client.map_err(redis_error)?;
        let connection = observer
            .observe(InfrastructureDependency::Redis, "captcha_connect", client.get_connection_manager())
            .await
            .map_err(redis_error)?;
        Ok(Self::new(connection, key_prefix, observer))
    }

    fn challenge_key(&self, token: &str) -> String {
        format!("{}:captcha:cap:challenge:{token}", self.key_prefix)
    }

    fn redeemed_key(&self, token_key: &str) -> String {
        format!("{}:captcha:cap:redeemed:{token_key}", self.key_prefix)
    }
}

#[async_trait]
impl CapStore for RedisCaptchaStore {
    async fn save_challenge(&self, token: &str, record: &CapChallengeRecord, ttl_seconds: u64) -> CaptchaResult<()> {
        let mut connection = self.connection.clone();
        let value = serde_json::to_string(record).map_err(json_error)?;
        self.observer
            .observe(
                InfrastructureDependency::Redis,
                "captcha_save_challenge",
                connection.set_ex::<_, _, ()>(self.challenge_key(token), value, ttl_seconds),
            )
            .await
            .map_err(redis_error)
    }

    async fn consume_challenge(&self, token: &str) -> CaptchaResult<Option<CapChallengeRecord>> {
        let mut connection = self.connection.clone();
        let value: Option<String> = self
            .observer
            .observe(
                InfrastructureDependency::Redis,
                "captcha_consume_challenge",
                redis::cmd("GETDEL").arg(self.challenge_key(token)).query_async(&mut connection),
            )
            .await
            .map_err(redis_error)?;
        value.map(|item| serde_json::from_str(&item).map_err(json_error)).transpose()
    }

    async fn save_redeemed(&self, token_key: &str, expires: i64, ttl_seconds: u64) -> CaptchaResult<()> {
        let mut connection = self.connection.clone();
        self.observer
            .observe(
                InfrastructureDependency::Redis,
                "captcha_save_redeemed",
                connection.set_ex::<_, _, ()>(self.redeemed_key(token_key), expires.to_string(), ttl_seconds),
            )
            .await
            .map_err(redis_error)
    }

    async fn consume_redeemed(&self, token_key: &str) -> CaptchaResult<bool> {
        let mut connection = self.connection.clone();
        let value: Option<String> = self
            .observer
            .observe(
                InfrastructureDependency::Redis,
                "captcha_consume_redeemed",
                redis::cmd("GETDEL").arg(self.redeemed_key(token_key)).query_async(&mut connection),
            )
            .await
            .map_err(redis_error)?;
        Ok(value.is_some())
    }
}

fn redis_error(error: redis::RedisError) -> CaptchaError {
    CaptchaError::Infrastructure(format!("redis error: {error}"))
}

fn json_error(error: serde_json::Error) -> CaptchaError {
    CaptchaError::Infrastructure(format!("captcha store json error: {error}"))
}
