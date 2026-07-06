use async_trait::async_trait;
use redis::AsyncCommands;

use crate::{
    application::{CaptchaError, CaptchaResult},
    providers::cap::{CapChallengeRecord, CapStore},
};

#[derive(Clone)]
pub struct RedisCaptchaStore {
    connection: redis::aio::ConnectionManager,
    key_prefix: String,
}

impl RedisCaptchaStore {
    pub fn new(connection: redis::aio::ConnectionManager, key_prefix: String) -> Self {
        Self { connection, key_prefix }
    }

    pub async fn connect(url: &str, key_prefix: String) -> CaptchaResult<Self> {
        let client = redis::Client::open(url).map_err(redis_error)?;
        let connection = client.get_connection_manager().await.map_err(redis_error)?;
        Ok(Self { connection, key_prefix })
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
        connection
            .set_ex::<_, _, ()>(self.challenge_key(token), value, ttl_seconds)
            .await
            .map_err(redis_error)
    }

    async fn consume_challenge(&self, token: &str) -> CaptchaResult<Option<CapChallengeRecord>> {
        let mut connection = self.connection.clone();
        let value: Option<String> = redis::cmd("GETDEL")
            .arg(self.challenge_key(token))
            .query_async(&mut connection)
            .await
            .map_err(redis_error)?;
        value.map(|item| serde_json::from_str(&item).map_err(json_error)).transpose()
    }

    async fn save_redeemed(&self, token_key: &str, expires: i64, ttl_seconds: u64) -> CaptchaResult<()> {
        let mut connection = self.connection.clone();
        connection
            .set_ex::<_, _, ()>(self.redeemed_key(token_key), expires.to_string(), ttl_seconds)
            .await
            .map_err(redis_error)
    }

    async fn consume_redeemed(&self, token_key: &str) -> CaptchaResult<bool> {
        let mut connection = self.connection.clone();
        let value: Option<String> = redis::cmd("GETDEL")
            .arg(self.redeemed_key(token_key))
            .query_async(&mut connection)
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
