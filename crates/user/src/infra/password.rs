use argon2::{Argon2, PasswordHash, PasswordHasher as ArgonPasswordHasher, PasswordVerifier, password_hash::SaltString};
use rand_core::OsRng;

use crate::application::{AppError, AppResult, PasswordHasher};

#[derive(Clone, Default)]
pub struct Argon2PasswordHasher;

impl PasswordHasher for Argon2PasswordHasher {
    fn hash(&self, password: &str) -> AppResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(password_error)
    }

    fn verify(&self, password: &str, password_hash: &str) -> AppResult<bool> {
        let parsed_hash = PasswordHash::new(password_hash).map_err(password_error)?;
        Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
}

fn password_error(error: argon2::password_hash::Error) -> AppError {
    AppError::Infrastructure(error.to_string())
}
