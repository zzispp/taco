use thiserror::Error;

pub type StorageResult<T> = Result<T, StorageError>;

const POSTGRES_UNIQUE_VIOLATION_CODE: &str = "23505";

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("resource not found")]
    NotFound,
    #[error("resource conflict: {0}")]
    Conflict(String),
    #[error("database error: {message}")]
    UniqueViolation { constraint: Option<String>, message: String },
    #[error("database error: {0}")]
    Database(String),
}

impl From<sqlx::Error> for StorageError {
    fn from(value: sqlx::Error) -> Self {
        let message = value.to_string();
        let Some(database) = value.as_database_error() else {
            return Self::Database(message);
        };
        let code = database.code();
        classify_database_error(code.as_deref(), database.constraint(), message)
    }
}

fn classify_database_error(code: Option<&str>, constraint: Option<&str>, message: String) -> StorageError {
    if code == Some(POSTGRES_UNIQUE_VIOLATION_CODE) {
        return StorageError::UniqueViolation {
            constraint: constraint.map(str::to_owned),
            message,
        };
    }
    StorageError::Database(message)
}

impl From<serde_json::Error> for StorageError {
    fn from(value: serde_json::Error) -> Self {
        Self::Database(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{StorageError, classify_database_error};

    #[test]
    fn only_postgres_unique_violation_is_classified_separately() {
        let unique = classify_database_error(Some("23505"), Some("idx_sys_user_name_active"), "duplicate key".into());
        let foreign_key = classify_database_error(Some("23503"), Some("fk_sys_user_dept"), "foreign key".into());

        assert!(matches!(
            unique,
            StorageError::UniqueViolation { constraint: Some(constraint), message }
                if constraint == "idx_sys_user_name_active" && message == "duplicate key"
        ));
        assert!(matches!(foreign_key, StorageError::Database(message) if message == "foreign key"));
    }
}
