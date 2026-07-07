use std::collections::HashMap;

use async_trait::async_trait;
use redis::AsyncCommands;

use crate::application::{RbacCache, RbacError, RbacResult};
use crate::domain::{NavItemResponse, NavResponse, NavSectionResponse, PermissionSnapshot};

const RBAC_CACHE_MISSING_ERROR: &str = "infra.rbac.cache_missing";

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
        format!("{}:rbac:snapshot:v2", self.key_prefix)
    }
}

#[async_trait]
impl RbacCache for RedisRbacCache {
    async fn write_snapshot(&self, snapshot: &PermissionSnapshot) -> RbacResult<()> {
        let mut connection = self.connection.clone();
        let snapshot_json = serde_json::to_string(snapshot).map_err(json_error)?;
        connection.set(self.snapshot_key(), snapshot_json).await.map_err(redis_error)
    }

    async fn read_snapshot(&self) -> RbacResult<PermissionSnapshot> {
        let mut connection = self.connection.clone();
        let value: Option<String> = connection.get(self.snapshot_key()).await.map_err(redis_error)?;
        let value = value.ok_or_else(|| RbacError::Infrastructure(RBAC_CACHE_MISSING_ERROR.into()))?;
        serde_json::from_str(&value).map_err(json_error)
    }

    async fn read_nav(&self, role_keys: &[String], admin: bool) -> RbacResult<NavResponse> {
        let snapshot = self.read_snapshot().await?;
        let sections = snapshot
            .menus
            .into_iter()
            .filter(|menu| admin || role_keys.contains(&menu.role_key))
            .flat_map(|menu| menu.sections);
        Ok(NavResponse {
            nav_items: merge_sections(sections),
        })
    }
}

fn merge_sections(sections: impl IntoIterator<Item = NavSectionResponse>) -> Vec<NavSectionResponse> {
    let mut merged = Vec::<NavSectionResponse>::new();
    let mut indexes = HashMap::<String, usize>::new();

    for section in sections {
        let key = section_key(&section);
        match indexes.get(&key).copied() {
            Some(index) => merge_items(&mut merged[index].items, section.items),
            None => {
                indexes.insert(key, merged.len());
                merged.push(section);
            }
        }
    }

    merged
}

fn merge_items(target: &mut Vec<NavItemResponse>, items: Vec<NavItemResponse>) {
    let mut indexes = target
        .iter()
        .enumerate()
        .map(|(index, item)| (item_key(item), index))
        .collect::<HashMap<_, _>>();

    for item in items {
        let key = item_key(&item);
        match indexes.get(&key).copied() {
            Some(index) => merge_items(&mut target[index].children, item.children),
            None => {
                indexes.insert(key, target.len());
                target.push(item);
            }
        }
    }
}

fn section_key(section: &NavSectionResponse) -> String {
    if !section.code.is_empty() {
        return section.code.clone();
    }
    section.subheader.clone()
}

fn item_key(item: &NavItemResponse) -> String {
    if !item.code.is_empty() {
        return item.code.clone();
    }
    item.path.clone()
}

fn redis_error(error: redis::RedisError) -> RbacError {
    RbacError::Infrastructure(format!("redis error: {error}"))
}

fn json_error(error: serde_json::Error) -> RbacError {
    RbacError::Infrastructure(format!("rbac cache json error: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_sections_deduplicates_admin_role_nav_groups() {
        let sections = vec![
            section("system_management", vec![item("100", "用户管理")]),
            section("system_management", vec![item("100", "用户管理"), item("101", "角色管理")]),
        ];

        let merged = merge_sections(sections);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].code, "system_management");
        assert_eq!(merged[0].items.iter().map(|item| item.code.as_str()).collect::<Vec<_>>(), vec!["100", "101"]);
    }

    fn section(code: &str, items: Vec<NavItemResponse>) -> NavSectionResponse {
        NavSectionResponse {
            code: code.into(),
            subheader: "System Management".into(),
            items,
        }
    }

    fn item(code: &str, title: &str) -> NavItemResponse {
        NavItemResponse {
            code: code.into(),
            title: title.into(),
            path: format!("/dashboard/{code}"),
            icon: None,
            caption: None,
            deep_match: true,
            children: vec![],
        }
    }
}
