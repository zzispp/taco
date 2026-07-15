use constants::system::{ALL_PERMISSION, SUPER_ADMIN_ROLE_KEY};
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
    let admin = record.role_keys.iter().any(|role| role == SUPER_ADMIN_ROLE_KEY);
    AuthorizationUser {
        id: UserId(record.user_id),
        username: record.user_name,
        dept_id: record.dept_id,
        status: record.status,
        role_keys: record.role_keys,
        permissions: if admin { vec![ALL_PERMISSION.into()] } else { record.permissions },
    }
}

pub fn user(record: UserRecord, relations: UserRelations) -> Result<User, StorageError> {
    let roles = relations.roles;
    let permissions = user_permissions(&roles, relations.permissions);
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

fn user_permissions(roles: &[RoleSummary], permissions: Vec<String>) -> Vec<String> {
    if roles.iter().any(|role| role.role_key == SUPER_ADMIN_ROLE_KEY) {
        return vec![ALL_PERMISSION.into()];
    }

    permissions
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
    fn fixed_user_id_with_common_role_keeps_explicit_permissions() {
        let user = user(user_record("1"), relations(vec![common_role()], vec!["system:user:list"])).unwrap();

        assert_eq!(user.permissions, vec!["system:user:list"]);
    }

    #[test]
    fn admin_role_gets_taco_wildcard_permission() {
        let user = user(user_record("2"), relations(vec![admin_role()], vec!["system:user:list"])).unwrap();

        assert_eq!(user.permissions, vec![constants::system::ALL_PERMISSION]);
    }

    #[test]
    fn fixed_user_id_authorization_keeps_explicit_permissions() {
        let user = authorization_user(authorization_record("1", vec!["common"], vec!["system:user:list"]));

        assert_eq!(user.permissions, vec!["system:user:list"]);
    }

    #[test]
    fn admin_role_authorization_gets_wildcard_permission() {
        let user = authorization_user(authorization_record("2", vec![SUPER_ADMIN_ROLE_KEY], vec!["system:user:list"]));

        assert_eq!(user.permissions, vec![ALL_PERMISSION]);
    }

    #[test]
    fn common_role_keeps_explicit_permissions() {
        let user = user(user_record("2"), relations(vec![common_role()], vec!["system:user:list"])).unwrap();

        assert_eq!(user.permissions, vec!["system:user:list"]);
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

    fn authorization_record(user_id: &str, role_keys: Vec<&str>, permissions: Vec<&str>) -> AuthorizationUserRecord {
        AuthorizationUserRecord {
            user_id: user_id.into(),
            user_name: "fixture-user".into(),
            dept_id: None,
            status: "0".into(),
            role_keys: role_keys.into_iter().map(String::from).collect(),
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
