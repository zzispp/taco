use async_trait::async_trait;
use sqlx::{PgPool, postgres::PgListener};

use crate::application::{ChangeListener, ChangeListenerFactory, SchedulerError, SchedulerResult};

const SCHEDULER_CHANGE_CHANNEL: &str = "scheduler_changed";

#[derive(Clone)]
pub struct PostgresChangeListenerFactory {
    pool: PgPool,
}

impl PostgresChangeListenerFactory {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

struct PostgresChangeListener {
    listener: PgListener,
}

#[async_trait]
impl ChangeListenerFactory for PostgresChangeListenerFactory {
    async fn connect(&self) -> SchedulerResult<Box<dyn ChangeListener>> {
        let mut listener = PgListener::connect_with(&self.pool).await.map_err(infrastructure)?;
        listener.listen(SCHEDULER_CHANGE_CHANNEL).await.map_err(infrastructure)?;
        Ok(Box::new(PostgresChangeListener { listener }))
    }
}

#[async_trait]
impl ChangeListener for PostgresChangeListener {
    async fn wait(&mut self) -> SchedulerResult<()> {
        self.listener.recv().await.map(|_| ()).map_err(infrastructure)
    }
}

fn infrastructure(error: sqlx::Error) -> SchedulerError {
    SchedulerError::Infrastructure(format!("scheduler notification failure: {error}"))
}
