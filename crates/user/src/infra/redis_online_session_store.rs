use async_trait::async_trait;
use redis::AsyncCommands;

use crate::application::{AppError, AppResult, OnlineSession, OnlineSessionStore};

#[derive(Clone)]
pub struct RedisOnlineSessionStore {
    connection: redis::aio::ConnectionManager,
    key_prefix: String,
}

impl RedisOnlineSessionStore {
    pub async fn connect(url: &str, key_prefix: String) -> AppResult<Self> {
        let client = redis::Client::open(url).map_err(redis_error)?;
        let connection = client.get_connection_manager().await.map_err(redis_error)?;
        Ok(Self { connection, key_prefix })
    }

    fn session_key(&self, token_id: &str) -> String {
        format!("{}:user:online:{token_id}", self.key_prefix)
    }

    fn session_pattern(&self) -> String {
        format!("{}:user:online:*", self.key_prefix)
    }
}

#[async_trait]
impl OnlineSessionStore for RedisOnlineSessionStore {
    async fn save(&self, session: &OnlineSession, ttl_seconds: u64) -> AppResult<()> {
        let mut connection = self.connection.clone();
        let value = serde_json::to_string(session).map_err(json_error)?;
        connection
            .set_ex::<_, _, ()>(self.session_key(&session.token_id), value, ttl_seconds)
            .await
            .map_err(redis_error)
    }

    async fn find(&self, token_id: &str) -> AppResult<Option<OnlineSession>> {
        let mut connection = self.connection.clone();
        let value: Option<String> = connection.get(self.session_key(token_id)).await.map_err(redis_error)?;
        value.map(|item| serde_json::from_str(&item).map_err(json_error)).transpose()
    }

    async fn delete(&self, token_id: &str) -> AppResult<()> {
        let mut connection = self.connection.clone();
        connection.del::<_, ()>(self.session_key(token_id)).await.map_err(redis_error)
    }

    async fn list(&self) -> AppResult<Vec<OnlineSession>> {
        let mut connection = self.connection.clone();
        let keys: Vec<String> = connection.keys(self.session_pattern()).await.map_err(redis_error)?;
        read_sessions(&mut connection, keys).await
    }
}

async fn read_sessions(connection: &mut redis::aio::ConnectionManager, keys: Vec<String>) -> AppResult<Vec<OnlineSession>> {
    let mut sessions = Vec::with_capacity(keys.len());
    for key in keys {
        if let Some(value) = connection.get::<_, Option<String>>(key).await.map_err(redis_error)? {
            sessions.push(serde_json::from_str(&value).map_err(json_error)?);
        }
    }
    Ok(sessions)
}

fn redis_error(error: redis::RedisError) -> AppError {
    AppError::Infrastructure(format!("redis error: {error}"))
}

fn json_error(error: serde_json::Error) -> AppError {
    AppError::Infrastructure(format!("online session json error: {error}"))
}
