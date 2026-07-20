use kernel::error::LocalizedError;
use storage::StorageError;
use types::{
    rbac::RoleSummary,
    user::{User, UserId},
};

use crate::application::{AppError, AuthorizationUser, UserAuthRecord};

use super::record::{AuthorizationUserRecord, UserRecord};

const USERNAME_UNIQUE_CONSTRAINT: &str = "idx_sys_user_name_active";
const EMAIL_UNIQUE_CONSTRAINT: &str = "idx_sys_user_email_active_ci";
const PHONE_UNIQUE_CONSTRAINT: &str = "idx_sys_user_phone_active";

pub fn user_auth_record(record: (User, String)) -> UserAuthRecord {
    UserAuthRecord {
        user: record.0,
        password_hash: record.1,
    }
}

pub fn authorization_user(record: AuthorizationUserRecord) -> AuthorizationUser {
    AuthorizationUser {
        id: UserId(record.user_id),
        username: record.user_name,
        dept_id: record.dept_id,
        status: record.status,
        is_installation_owner: record.is_installation_owner,
        role_keys: record.role_keys,
        permissions: record.permissions,
    }
}

pub fn user(record: UserRecord, relations: UserRelations) -> Result<User, StorageError> {
    let roles = relations.roles;
    let permissions = relations.permissions;
    let create_time = types::http::format_utc_rfc3339_millis(record.create_time).map_err(|error| StorageError::Database(error.to_string()))?;
    Ok(User {
        id: UserId(record.user_id),
        username: record.user_name,
        nick_name: record.nick_name,
        dept_id: record.dept_id,
        email: record.email,
        phonenumber: record.phonenumber,
        sex: record.sex,
        avatar: record.avatar,
        status: record.status,
        is_installation_owner: record.is_installation_owner,
        auth_source: record.auth_source,
        email_verified: record.email_verified,
        remark: record.remark,
        roles,
        role_ids: relations.role_ids,
        post_ids: relations.post_ids,
        permissions,
        create_time,
    })
}

pub fn storage_error(error: StorageError) -> AppError {
    match error {
        StorageError::NotFound => AppError::NotFound,
        StorageError::Conflict(_) => AppError::Conflict(LocalizedError::new("errors.common.conflict")),
        StorageError::UniqueViolation { constraint, message } => user_unique_violation(constraint.as_deref(), message),
        StorageError::Database(message) => AppError::Infrastructure(message),
    }
}

fn user_unique_violation(constraint: Option<&str>, message: String) -> AppError {
    let field = match constraint {
        Some(USERNAME_UNIQUE_CONSTRAINT) => "username",
        Some(EMAIL_UNIQUE_CONSTRAINT) => "email",
        Some(PHONE_UNIQUE_CONSTRAINT) => "phonenumber",
        _ => return AppError::Infrastructure(message),
    };
    AppError::Conflict(LocalizedError::new("errors.user.duplicate_field").with_param("field", field))
}

#[derive(Default)]
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
    fn every_role_keeps_only_explicit_permissions() {
        let user = user(user_record("2"), relations(vec![role("1", "business-admin")], vec!["system:user:list"])).unwrap();

        assert_eq!(user.permissions, vec!["system:user:list"]);
    }

    #[test]
    fn authorization_projection_preserves_explicit_permissions_and_owner_marker() {
        let user = authorization_user(authorization_record("1", true, vec!["business-admin"], vec!["system:user:list"]));

        assert_eq!(user.permissions, vec!["system:user:list"]);
        assert!(user.is_installation_owner);
    }

    #[test]
    fn user_unique_constraints_map_to_field_conflicts() {
        for (constraint, field) in [
            ("idx_sys_user_name_active", "username"),
            ("idx_sys_user_email_active_ci", "email"),
            ("idx_sys_user_phone_active", "phonenumber"),
        ] {
            let error = storage_error(unique_violation(constraint));
            let AppError::Conflict(message) = error else {
                panic!("known user unique constraint must map to conflict");
            };
            assert_eq!(message.key(), "errors.user.duplicate_field");
            assert_eq!(message.params().len(), 1);
            assert_eq!(message.params()[0].key(), "field");
            assert_eq!(message.params()[0].value(), field);
        }
    }

    #[test]
    fn unknown_unique_constraint_remains_infrastructure_error() {
        assert!(matches!(storage_error(unique_violation("unknown_unique_index")), AppError::Infrastructure(message) if message == "duplicate key"));
        assert!(matches!(storage_error(StorageError::Database("connection lost".into())), AppError::Infrastructure(message) if message == "connection lost"));
    }

    fn unique_violation(constraint: &str) -> StorageError {
        StorageError::UniqueViolation {
            constraint: Some(constraint.into()),
            message: "duplicate key".into(),
        }
    }

    fn user_record(user_id: &str) -> UserRecord {
        UserRecord {
            user_id: user_id.into(),
            dept_id: Some("103".into()),
            user_name: "fixture-user".into(),
            nick_name: "Fixture User".into(),
            email: "fixture-user@example.test".into(),
            phonenumber: None,
            sex: "2".into(),
            avatar: None,
            password: "hash".into(),
            status: "0".into(),
            is_installation_owner: false,
            auth_source: "local".into(),
            email_verified: true,
            remark: None,
            create_time: time::OffsetDateTime::UNIX_EPOCH,
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

    fn authorization_record(user_id: &str, is_installation_owner: bool, role_keys: Vec<&str>, permissions: Vec<&str>) -> AuthorizationUserRecord {
        AuthorizationUserRecord {
            user_id: user_id.into(),
            user_name: "fixture-user".into(),
            dept_id: None,
            status: "0".into(),
            is_installation_owner,
            role_keys: role_keys.into_iter().map(String::from).collect(),
            permissions: permissions.into_iter().map(String::from).collect(),
        }
    }

    fn role(role_id: &str, role_key: &str) -> RoleSummary {
        RoleSummary {
            role_id: role_id.into(),
            role_name: role_key.into(),
            role_key: role_key.into(),
        }
    }
}
