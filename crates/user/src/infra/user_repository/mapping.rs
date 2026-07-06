use constants::system::{ALL_PERMISSION, SUPER_ADMIN_ROLE_KEY, SUPER_ADMIN_USER_ID};
use kernel::error::LocalizedError;
use storage::StorageError;
use types::{
    rbac::RoleSummary,
    user::{User, UserId},
};

use crate::application::{AppError, UserAuthRecord};

use super::record::{RoleSummaryRecord, UserRecord};

pub fn user_auth_record(record: (User, String)) -> UserAuthRecord {
    UserAuthRecord {
        user: record.0,
        password_hash: record.1,
    }
}

pub fn role_summary(record: RoleSummaryRecord) -> RoleSummary {
    RoleSummary {
        role_id: record.role_id,
        role_name: record.role_name,
        role_key: record.role_key,
    }
}

pub fn user(record: UserRecord, relations: UserRelations) -> User {
    let system = record.user_id == SUPER_ADMIN_USER_ID;
    let roles = relations.roles;
    let permissions = user_permissions(system, &roles, relations.permissions);
    User {
        id: UserId(record.user_id),
        username: record.user_name,
        nick_name: record.nick_name,
        dept_id: record.dept_id,
        email: record.email,
        phonenumber: record.phonenumber,
        sex: record.sex,
        avatar: record.avatar,
        status: record.status,
        auth_source: record.auth_source,
        email_verified: record.email_verified,
        system,
        remark: record.remark,
        roles,
        role_ids: relations.role_ids,
        post_ids: relations.post_ids,
        permissions,
        create_time: record.create_time,
    }
}

fn user_permissions(system: bool, roles: &[RoleSummary], permissions: Vec<String>) -> Vec<String> {
    if system || roles.iter().any(|role| role.role_key == SUPER_ADMIN_ROLE_KEY) {
        return vec![ALL_PERMISSION.into()];
    }

    permissions
}

pub fn storage_error(error: StorageError) -> AppError {
    match error {
        StorageError::NotFound => AppError::NotFound,
        StorageError::Conflict(_) => AppError::Conflict(LocalizedError::new("errors.common.conflict")),
        StorageError::Database(message) => AppError::Infrastructure(message),
    }
}

pub struct UserRelations {
    pub roles: Vec<RoleSummary>,
    pub role_ids: Vec<String>,
    pub post_ids: Vec<String>,
    pub permissions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn super_admin_user_gets_taco_wildcard_permission() {
        let user = user(
            user_record(constants::system::SUPER_ADMIN_USER_ID),
            relations(vec![admin_role()], vec!["system:user:list"]),
        );

        assert_eq!(user.permissions, vec![constants::system::ALL_PERMISSION]);
        assert!(user.system);
    }

    #[test]
    fn admin_role_gets_taco_wildcard_permission() {
        let user = user(user_record("2"), relations(vec![admin_role()], vec!["system:user:list"]));

        assert_eq!(user.permissions, vec![constants::system::ALL_PERMISSION]);
        assert!(!user.system);
    }

    #[test]
    fn common_role_keeps_explicit_permissions() {
        let user = user(user_record("2"), relations(vec![common_role()], vec!["system:user:list"]));

        assert_eq!(user.permissions, vec!["system:user:list"]);
    }

    fn user_record(user_id: &str) -> UserRecord {
        UserRecord {
            user_id: user_id.into(),
            dept_id: Some("103".into()),
            user_name: "admin".into(),
            nick_name: "taco".into(),
            email: "admin@taco.local".into(),
            phonenumber: None,
            sex: "2".into(),
            avatar: None,
            password: "hash".into(),
            status: "0".into(),
            auth_source: "local".into(),
            email_verified: true,
            remark: None,
            create_time: "2026-01-01 00:00:00".into(),
        }
    }

    fn relations(roles: Vec<RoleSummary>, permissions: Vec<&str>) -> UserRelations {
        UserRelations {
            role_ids: roles.iter().map(|role| role.role_id.clone()).collect(),
            roles,
            post_ids: vec!["1".into()],
            permissions: permissions.into_iter().map(String::from).collect(),
        }
    }

    fn admin_role() -> RoleSummary {
        role("1", constants::system::SUPER_ADMIN_ROLE_KEY)
    }

    fn common_role() -> RoleSummary {
        role("2", "common")
    }

    fn role(role_id: &str, role_key: &str) -> RoleSummary {
        RoleSummary {
            role_id: role_id.into(),
            role_name: role_key.into(),
            role_key: role_key.into(),
        }
    }
}
