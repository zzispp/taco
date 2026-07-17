mod audited;
mod command;
mod execution_store;
mod export_session;
mod mapping;
mod query;
mod records;
mod runtime_store;
#[cfg(test)]
mod runtime_store_tests;
mod sql;
mod write_support;

use storage::{Database, ObservedPgPool};

#[derive(Clone)]
pub struct StorageSchedulerRepository {
    pub(super) database: Database,
}

impl StorageSchedulerRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub(crate) fn pool(&self) -> ObservedPgPool {
        self.database.pool()
    }
}
