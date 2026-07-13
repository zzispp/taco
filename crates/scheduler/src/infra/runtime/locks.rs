use async_trait::async_trait;
use sqlx::{PgPool, Postgres, pool::PoolConnection, query, query_scalar};

use crate::application::{ExecutionLease, ExecutionLeaseSession, LeaderLease, LeaderSession, SchedulerError, SchedulerResult};

const LEADER_LOCK_KEY: i64 = 0x5343_4845_4455_4c45;
const EXECUTION_LOCK_SEED: i64 = 0x4558_4543;

#[derive(Clone)]
pub struct PostgresLeaderLease {
    pool: PgPool,
}

impl PostgresLeaderLease {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

struct PostgresLeaderSession {
    connection: PoolConnection<Postgres>,
    released: bool,
}

#[async_trait]
impl LeaderLease for PostgresLeaderLease {
    async fn try_acquire(&self) -> SchedulerResult<Option<Box<dyn LeaderSession>>> {
        let mut connection = self.pool.acquire().await.map_err(infrastructure)?;
        let acquired = query_scalar("SELECT pg_try_advisory_lock($1)")
            .bind(LEADER_LOCK_KEY)
            .fetch_one(&mut *connection)
            .await
            .map_err(infrastructure)?;
        if acquired {
            connection.close_on_drop();
        }
        Ok(acquired.then(|| Box::new(PostgresLeaderSession { connection, released: false }) as Box<dyn LeaderSession>))
    }
}

#[async_trait]
impl LeaderSession for PostgresLeaderSession {
    async fn is_alive(&mut self) -> SchedulerResult<bool> {
        query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&mut *self.connection)
            .await
            .map(|_| true)
            .map_err(infrastructure)
    }

    async fn release(&mut self) -> SchedulerResult<()> {
        if self.released {
            return Ok(());
        }
        let released: bool = query_scalar("SELECT pg_advisory_unlock($1)")
            .bind(LEADER_LOCK_KEY)
            .fetch_one(&mut *self.connection)
            .await
            .map_err(infrastructure)?;
        if !released {
            return Err(SchedulerError::Infrastructure("scheduler leader lock was not held".into()));
        }
        self.released = true;
        Ok(())
    }
}

#[derive(Clone)]
pub struct PostgresExecutionLease {
    pool: PgPool,
}

impl PostgresExecutionLease {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

struct PostgresExecutionSession {
    connection: PoolConnection<Postgres>,
}

#[async_trait]
impl ExecutionLease for PostgresExecutionLease {
    async fn open_session(&self) -> SchedulerResult<Box<dyn ExecutionLeaseSession>> {
        let mut connection = self.pool.acquire().await.map_err(infrastructure)?;
        connection.close_on_drop();
        Ok(Box::new(PostgresExecutionSession { connection }))
    }

    async fn is_owned(&self, execution_id: &str) -> SchedulerResult<bool> {
        let mut connection = self.pool.acquire().await.map_err(infrastructure)?;
        let acquired = try_execution_lock(&mut connection, execution_id).await?;
        if !acquired {
            return Ok(true);
        }
        unlock_execution(&mut connection, execution_id).await?;
        Ok(false)
    }
}

#[async_trait]
impl ExecutionLeaseSession for PostgresExecutionSession {
    async fn try_acquire(&mut self, execution_id: &str) -> SchedulerResult<bool> {
        try_execution_lock(&mut self.connection, execution_id).await
    }

    async fn release(&mut self, execution_id: &str) -> SchedulerResult<()> {
        unlock_execution(&mut self.connection, execution_id).await
    }

    async fn is_alive(&mut self) -> SchedulerResult<bool> {
        query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&mut *self.connection)
            .await
            .map(|_| true)
            .map_err(infrastructure)
    }

    async fn release_all(&mut self) -> SchedulerResult<()> {
        query("SELECT pg_advisory_unlock_all()")
            .execute(&mut *self.connection)
            .await
            .map_err(infrastructure)?;
        Ok(())
    }
}

async fn try_execution_lock(connection: &mut PoolConnection<Postgres>, execution_id: &str) -> SchedulerResult<bool> {
    query_scalar("SELECT pg_try_advisory_lock(hashtextextended($1, $2))")
        .bind(execution_id)
        .bind(EXECUTION_LOCK_SEED)
        .fetch_one(&mut **connection)
        .await
        .map_err(infrastructure)
}

async fn unlock_execution(connection: &mut PoolConnection<Postgres>, execution_id: &str) -> SchedulerResult<()> {
    let released: bool = query_scalar("SELECT pg_advisory_unlock(hashtextextended($1, $2))")
        .bind(execution_id)
        .bind(EXECUTION_LOCK_SEED)
        .fetch_one(&mut **connection)
        .await
        .map_err(infrastructure)?;
    if released {
        return Ok(());
    }
    Err(SchedulerError::Infrastructure(format!("execution advisory lock was not held: {execution_id}")))
}

fn infrastructure(error: sqlx::Error) -> SchedulerError {
    SchedulerError::Infrastructure(format!("scheduler advisory lock failure: {error}"))
}
