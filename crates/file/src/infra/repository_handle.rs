use storage::Database;

#[derive(Clone)]
pub struct StorageFileRepository {
    pub(super) database: Database,
}

impl StorageFileRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub fn database(&self) -> &Database {
        &self.database
    }
}
