use async_trait::async_trait;
use redis::AsyncCommands;

use crate::application::{RbacCache, RbacError, RbacResult};
use crate::domain::{NavResponse, PermissionSnapshot, RoleMenuSnapshot};

#[derive(Clone)]
pub struct RedisRbacCache {
    connection: redis::aio::ConnectionManager,
    key_prefix: String,
}

impl RedisRbacCache {
    pub async fn connect(url: &str, key_prefix: String) -> RbacResult<Self> {
        let client = redis::Client::open(url).map_err(redis_error)?;
        let connection = client.get_connection_manager().await.map_err(redis_error)?;
        Ok(Self { connection, key_prefix })
    }

    fn snapshot_key(&self) -> String {
        format!("{}:rbac:api_policy:v1", self.key_prefix)
    }

    fn role_menu_key(&self, role_code: &str) -> String {
        format!("{}:rbac:role_menu:v1:{role_code}", self.key_prefix)
    }
}

#[async_trait]
impl RbacCache for RedisRbacCache {
    async fn write_snapshot(&self, snapshot: &PermissionSnapshot) -> RbacResult<()> {
        let mut connection = self.connection.clone();
        let snapshot_json = serde_json::to_string(snapshot).map_err(json_error)?;
        let _: () = connection.set(self.snapshot_key(), snapshot_json).await.map_err(redis_error)?;
        for menu in &snapshot.menus {
            write_role_menu(&mut connection, self.role_menu_key(&menu.role_code), menu).await?;
        }
        Ok(())
    }

    async fn read_snapshot(&self) -> RbacResult<PermissionSnapshot> {
        let mut connection = self.connection.clone();
        let value: Option<String> = connection.get(self.snapshot_key()).await.map_err(redis_error)?;
        let value = value.ok_or_else(|| RbacError::Infrastructure("rbac permission cache is missing".into()))?;
        serde_json::from_str(&value).map_err(json_error)
    }

    async fn read_nav(&self, role_code: &str) -> RbacResult<NavResponse> {
        let mut connection = self.connection.clone();
        let value: Option<String> = connection.get(self.role_menu_key(role_code)).await.map_err(redis_error)?;
        let value = value.ok_or_else(|| RbacError::Infrastructure(format!("rbac menu cache is missing for role {role_code}")))?;
        serde_json::from_str(&value).map_err(json_error)
    }
}

async fn write_role_menu(connection: &mut redis::aio::ConnectionManager, key: String, menu: &RoleMenuSnapshot) -> RbacResult<()> {
    let value = serde_json::to_string(&NavResponse {
        nav_items: menu.sections.clone(),
    })
    .map_err(json_error)?;
    let _: () = connection.set(key, value).await.map_err(redis_error)?;
    Ok(())
}

fn redis_error(error: redis::RedisError) -> RbacError {
    RbacError::Infrastructure(format!("redis error: {error}"))
}

fn json_error(error: serde_json::Error) -> RbacError {
    RbacError::Infrastructure(format!("rbac cache json error: {error}"))
}
