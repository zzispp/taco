use std::{
    fmt,
    ops::Deref,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use futures_util::StreamExt;
use sqlx::{
    Either, Execute, Executor, PgPool, Postgres, Transaction,
    pool::PoolConnection,
    postgres::{PgPoolOptions, PgQueryResult, PgRow},
    query,
};
use uuid::Uuid;

use crate::{StorageError, StorageResult};

#[derive(Clone)]
pub struct Database {
    inner: Arc<DatabaseInner>,
}

struct DatabaseInner {
    pool: PgPool,
    observer: RwLock<Option<Arc<dyn PostgresOperationObserver>>>,
}

/// Receives only safe PostgreSQL operation metadata from the shared SQLx boundary.
pub trait PostgresOperationObserver: Send + Sync + 'static {
    fn record(&self, operation: &'static str, elapsed: Duration, succeeded: bool);
}

#[derive(Clone)]
pub struct ObservedPgPool {
    pool: PgPool,
    observer: Option<Arc<dyn PostgresOperationObserver>>,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Self {
            inner: Arc::new(DatabaseInner {
                pool,
                observer: RwLock::new(None),
            }),
        }
    }

    pub fn pool(&self) -> ObservedPgPool {
        ObservedPgPool {
            pool: self.inner.pool.clone(),
            observer: self.inner.observer.read().unwrap().clone(),
        }
    }

    pub fn raw_pool(&self) -> &PgPool {
        &self.inner.pool
    }

    pub fn set_postgres_observer(&self, observer: Arc<dyn PostgresOperationObserver>) {
        *self.inner.observer.write().unwrap() = Some(observer);
    }

    pub fn next_id(&self) -> String {
        Uuid::now_v7().to_string()
    }

    pub fn into_inner(self) -> PgPool {
        self.inner.pool.clone()
    }

    /// Starts one read-only repeatable-read transaction for a multi-batch export.
    pub async fn begin_consistent_snapshot(&self) -> StorageResult<Transaction<'static, Postgres>> {
        let mut transaction = self.inner.pool.begin().await?;
        query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
            .execute(&mut *transaction)
            .await?;
        Ok(transaction)
    }
}

impl ObservedPgPool {
    pub async fn begin(&self) -> Result<Transaction<'static, Postgres>, sqlx::Error> {
        let started = Instant::now();
        let result = self.pool.begin().await;
        self.record("postgres_begin", started, result.is_ok());
        result
    }

    pub async fn begin_with(&self, statement: &'static str) -> Result<Transaction<'static, Postgres>, sqlx::Error> {
        let started = Instant::now();
        let result = self.pool.begin_with(statement).await;
        self.record("postgres_begin", started, result.is_ok());
        result
    }

    pub async fn acquire(&self) -> Result<PoolConnection<Postgres>, sqlx::Error> {
        let started = Instant::now();
        let result = self.pool.acquire().await;
        self.record("postgres_acquire", started, result.is_ok());
        result
    }

    fn record(&self, operation: &'static str, started: Instant, succeeded: bool) {
        self.recorder().record(operation, started, succeeded);
    }

    fn recorder(&self) -> OperationRecorder {
        OperationRecorder::new(self.observer.clone())
    }
}

impl Deref for ObservedPgPool {
    type Target = PgPool;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

impl fmt::Debug for ObservedPgPool {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("ObservedPgPool").finish_non_exhaustive()
    }
}

impl<'c> Executor<'c> for ObservedPgPool {
    type Database = Postgres;

    fn fetch_many<'e, 'q: 'e, E>(self, query: E) -> futures_util::stream::BoxStream<'e, Result<Either<PgQueryResult, PgRow>, sqlx::Error>>
    where
        E: 'q + Execute<'q, Self::Database>,
    {
        let pool = self.pool;
        let recorder = OperationRecorder::new(self.observer);
        Box::pin(async_stream::try_stream! {
            let started = Instant::now();
            let mut stream = pool.fetch_many(query);
            while let Some(result) = stream.next().await {
                match result {
                    Ok(value) => yield value,
                    Err(error) => {
                        recorder.record("postgres_fetch_many", started, false);
                        Err(error)?;
                    }
                }
            }
            recorder.record("postgres_fetch_many", started, true);
        })
    }

    fn fetch_optional<'e, 'q: 'e, E>(self, query: E) -> futures_util::future::BoxFuture<'e, Result<Option<PgRow>, sqlx::Error>>
    where
        E: 'q + Execute<'q, Self::Database>,
    {
        let pool = self.pool;
        let recorder = OperationRecorder::new(self.observer);
        Box::pin(async move {
            let started = Instant::now();
            let result = pool.fetch_optional(query).await;
            recorder.record("postgres_fetch_optional", started, result.is_ok());
            result
        })
    }

    fn prepare_with<'e>(
        self,
        sql: sqlx::SqlStr,
        parameters: &'e [<Self::Database as sqlx::Database>::TypeInfo],
    ) -> futures_util::future::BoxFuture<'e, Result<<Self::Database as sqlx::Database>::Statement, sqlx::Error>>
    where
        'c: 'e,
    {
        let pool = self.pool;
        let recorder = OperationRecorder::new(self.observer);
        Box::pin(async move {
            let started = Instant::now();
            let result = pool.prepare_with(sql, parameters).await;
            recorder.record("postgres_prepare", started, result.is_ok());
            result
        })
    }
}

#[derive(Clone)]
struct OperationRecorder {
    observer: Option<Arc<dyn PostgresOperationObserver>>,
}

impl OperationRecorder {
    fn new(observer: Option<Arc<dyn PostgresOperationObserver>>) -> Self {
        Self { observer }
    }

    fn record(&self, operation: &'static str, started: Instant, succeeded: bool) {
        if let Some(observer) = &self.observer {
            observer.record(operation, started.elapsed(), succeeded);
        }
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
