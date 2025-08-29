use sqlx::{Executor, SqlitePool};

use crate::storage::application_data::Application;

pub struct PatcherDatabase {
    db_pool: SqlitePool,
}

impl PatcherDatabase {
    pub fn new(db_pool: SqlitePool) -> Self {
        PatcherDatabase { db_pool }
    }

    /// Initialize tables in the database
    pub async fn initialize(&self) {
        let query = "
            CREATE TABLE IF NOT EXISTS applications (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                hash_code TEXT NOT NULL
            );
        ";
        self.db_pool.execute(query).await.unwrap();
    }

    pub async fn add_application(&self, name: &str, version: &str, hash_code: &str) {
        let query = "
            INSERT INTO applications (name, version, hash_code)
            VALUES (?, ?, ?);
        ";
        let _result = self
            .db_pool
            .execute(sqlx::query(query).bind(name).bind(version).bind(hash_code))
            .await
            .inspect_err(|e| println!("Error adding application: {}", e));
    }

    pub async fn list_applications(&self) -> Vec<Application> {
        let query = "
            SELECT id, name, version, hash_code
            FROM applications
        ";
        sqlx::query_as(query)
            .fetch_all(&self.db_pool)
            .await
            .inspect_err(|e| println!("Error listing applications: {e}"))
            .unwrap_or_default()
    }
}
