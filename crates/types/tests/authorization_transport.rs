use serde_json::{Value, json};
use types::{
    rbac::{PermissionSnapshot, RolePermissionSnapshot, RoleSummary},
    user::{User, UserId},
};

#[test]
fn authorization_transport_serializes_only_rbac_identity_and_grants() {
    assert_eq!(serde_json::to_value(sample_user()).unwrap(), expected_user_json());
    assert_eq!(serde_json::to_value(sample_snapshot()).unwrap(), expected_snapshot_json());
}

fn sample_user() -> User {
    User {
        id: UserId("user-1".into()),
        username: "admin".into(),
        nick_name: "System Administrator".into(),
        dept_id: None,
        email: "admin@taco.local".into(),
        phonenumber: None,
        sex: "0".into(),
        avatar: None,
        status: "0".into(),
        auth_source: "local".into(),
        email_verified: false,
        remark: None,
        roles: vec![RoleSummary {
            role_id: "role-1".into(),
            role_name: "Administrator".into(),
            role_key: "admin".into(),
        }],
        role_ids: vec!["role-1".into()],
        post_ids: Vec::new(),
        permissions: vec!["system:user:list".into()],
        create_time: "2026-07-22T00:00:00Z".into(),
    }
}

fn sample_snapshot() -> PermissionSnapshot {
    PermissionSnapshot {
        roles: vec![RolePermissionSnapshot {
            role_key: "admin".into(),
            status: "0".into(),
            permissions: vec!["system:user:list".into()],
            data_scope: "1".into(),
            dept_ids: Vec::new(),
        }],
        menus: Vec::new(),
    }
}

fn expected_user_json() -> Value {
    json!({
        "id": "user-1",
        "username": "admin",
        "nick_name": "System Administrator",
        "dept_id": null,
        "email": "admin@taco.local",
        "phonenumber": null,
        "sex": "0",
        "avatar": null,
        "status": "0",
        "auth_source": "local",
        "email_verified": false,
        "remark": null,
        "roles": [{
            "role_id": "role-1",
            "role_name": "Administrator",
            "role_key": "admin"
        }],
        "role_ids": ["role-1"],
        "post_ids": [],
        "permissions": ["system:user:list"],
        "create_time": "2026-07-22T00:00:00Z"
    })
}

fn expected_snapshot_json() -> Value {
    json!({
        "roles": [{
            "role_key": "admin",
            "status": "0",
            "permissions": ["system:user:list"],
            "data_scope": "1",
            "dept_ids": []
        }],
        "menus": []
    })
}
