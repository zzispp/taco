use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use super::super::*;

#[tokio::test]
async fn fifth_password_failure_locks_account_until_cleared() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let failures = MemoryLoginFailureStore::default();
    let service = UserService::new(repository, TestPasswordHasher).with_login_security(failures.clone(), TestLoginLockConfigProvider::default());
    let invalid = Credentials {
        identifier: "alice".into(),
        password: "bad-password".into(),
    };

    for _ in 0..4 {
        assert!(matches!(service.sign_in(invalid.clone()).await, Err(AppError::Unauthorized)));
    }
    assert!(matches!(service.sign_in(invalid).await, Err(AppError::AccountLocked { lock_minutes: 10 })));
    assert!(matches!(
        service
            .sign_in(Credentials {
                identifier: "alice".into(),
                password: VALID_PASSWORD.into(),
            },)
            .await,
        Err(AppError::AccountLocked { lock_minutes: 10 })
    ));

    service.unlock_login("alice").await.unwrap();
    assert_eq!(failures.count(&user_id(1)), 0);
}

#[tokio::test]
async fn email_identifier_unlocks_the_same_stable_user_counter() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let failures = MemoryLoginFailureStore::default();
    let service = UserService::new(repository, TestPasswordHasher).with_login_security(failures.clone(), TestLoginLockConfigProvider::default());
    let invalid = Credentials {
        identifier: "alice@example.com".into(),
        password: "bad-password".into(),
    };

    for _ in 0..5 {
        let _ = service.sign_in(invalid.clone()).await;
    }
    assert_eq!(failures.count(&user_id(1)), 5);

    service.unlock_login("alice@example.com").await.unwrap();
    assert_eq!(failures.count(&user_id(1)), 0);
}

#[tokio::test]
async fn successful_login_clears_previous_failures() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let failures = MemoryLoginFailureStore::default();
    let service = UserService::new(repository, TestPasswordHasher).with_login_security(failures.clone(), TestLoginLockConfigProvider::default());

    let _ = service
        .sign_in(Credentials {
            identifier: "alice".into(),
            password: "bad-password".into(),
        })
        .await;
    let login = service
        .sign_in(Credentials {
            identifier: "alice".into(),
            password: VALID_PASSWORD.into(),
        })
        .await
        .unwrap();

    assert_eq!(failures.count(&user_id(1)), 1);
    service.complete_sign_in(login, "203.0.113.7".into()).await.unwrap();
    assert_eq!(failures.count(&user_id(1)), 0);
}

#[tokio::test]
async fn failed_counter_cleanup_does_not_record_login_metadata() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let failures = MemoryLoginFailureStore::default();
    failures.fail_clear_with("login counter cleanup failed");
    let service = UserService::new(repository.clone(), TestPasswordHasher).with_login_security(failures, TestLoginLockConfigProvider::default());
    let login = service
        .sign_in(Credentials {
            identifier: "alice".into(),
            password: VALID_PASSWORD.into(),
        })
        .await
        .unwrap();

    let result = service.complete_sign_in(login, "203.0.113.7".into()).await;

    assert!(matches!(result, Err(AppError::Infrastructure(message)) if message == "login counter cleanup failed"));
    assert_eq!(repository.login_records(), Vec::new());
    assert_eq!(repository.login_ip_records(), Vec::new());
}

#[tokio::test]
async fn disabled_user_cannot_sign_in() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123").with_status("1"));
    let service = user_service_with_login_security(repository.clone());

    let result = service
        .sign_in(Credentials {
            identifier: "alice".into(),
            password: VALID_PASSWORD.into(),
        })
        .await;

    assert!(matches!(result, Err(AppError::AccountDisabled)));
    assert!(repository.login_records().is_empty());
}

#[tokio::test]
async fn unknown_account_does_not_create_failure_counter() {
    let failures = MemoryLoginFailureStore::default();
    let service =
        UserService::new(MemoryUserRepository::default(), TestPasswordHasher).with_login_security(failures.clone(), TestLoginLockConfigProvider::default());

    let result = service
        .sign_in(Credentials {
            identifier: "missing".into(),
            password: "bad-password".into(),
        })
        .await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
    assert_eq!(failures.count(&user_id(1)), 0);
}

#[tokio::test]
async fn unknown_account_executes_password_hash_cost() {
    let hash_calls = Arc::new(AtomicUsize::new(0));
    let hasher = CostTrackingPasswordHasher {
        hash_calls: hash_calls.clone(),
    };
    let service = UserService::new(MemoryUserRepository::default(), hasher);

    let result = service
        .sign_in(Credentials {
            identifier: "missing".into(),
            password: "bad-password".into(),
        })
        .await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
    assert_eq!(hash_calls.load(Ordering::SeqCst), 1);
}

struct CostTrackingPasswordHasher {
    hash_calls: Arc<AtomicUsize>,
}

impl crate::application::PasswordHasher for CostTrackingPasswordHasher {
    fn hash(&self, password: &str) -> AppResult<String> {
        self.hash_calls.fetch_add(1, Ordering::SeqCst);
        Ok(format!("hashed:{password}"))
    }

    fn verify(&self, password: &str, password_hash: &str) -> AppResult<bool> {
        Ok(password_hash == format!("hashed:{password}"))
    }
}
