use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;

use crate::{StorageError, StorageResult};

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub fn next_id(&self) -> String {
        Uuid::now_v7().to_string()
    }

    pub fn into_inner(self) -> PgPool {
        self.pool
    }
}

pub async fn connect_database(database_url: &str) -> StorageResult<Database> {
    let pool = PgPoolOptions::new().connect(database_url).await?;
    Ok(Database::new(pool))
}

pub fn to_i64(value: u64) -> StorageResult<i64> {
    i64::try_from(value).map_err(numeric_error)
}

pub fn to_u64(value: i64) -> StorageResult<u64> {
    u64::try_from(value).map_err(numeric_error)
}

fn numeric_error(error: impl std::fmt::Display) -> StorageError {
    StorageError::Database(format!("numeric conversion error: {error}"))
}
