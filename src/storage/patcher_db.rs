use std::path::Path;

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
                hash_code TEXT NOT NULL,
                install_path TEXT NOT NULL
            );

            CREATE UNIQUE INDEX IF NOT EXISTS ux_app_name ON applications (name);
        ";
        self.db_pool.execute(query).await.unwrap();
    }

    pub async fn add_application(
        &self,
        name: &str,
        version: &str,
        hash_code: &str,
        install_path: &Path,
    ) {
        let install_path = install_path.to_string_lossy();
        let query = "
            INSERT INTO applications (name, version, hash_code, install_path)
            VALUES (?, ?, ?, ?);
        ";
        let _result = self
            .db_pool
            .execute(
                sqlx::query(query)
                    .bind(name)
                    .bind(version)
                    .bind(hash_code)
                    .bind(install_path),
            )
            .await
            .inspect_err(|e| println!("Error adding application: {}", e));
    }

    pub async fn update_application(&self, id: &i64, version: &str, hash_code: &str) {
        let query = "
            UPDATE applications
            SET version = ?, hash_code = ?
            WHERE id = ?;
        ";
        let _result = self
            .db_pool
            .execute(sqlx::query(query).bind(version).bind(hash_code).bind(id))
            .await
            .inspect_err(|e| println!("Error updating application: {}", e));
    }

    pub async fn remove_application(&self, name: &str) {
        let query = "
            DELETE FROM applications
            WHERE name = ?;
        ";
        let _result = self
            .db_pool
            .execute(sqlx::query(query).bind(name))
            .await
            .inspect_err(|e| println!("Error removing application: {}", e));
    }

    pub async fn get_application(&self, name: &str) -> Option<Application> {
        let query = "
            SELECT id, name, version, hash_code, install_path
            FROM applications
            WHERE name = ?;
        ";
        sqlx::query_as(query)
            .bind(name)
            .fetch_one(&self.db_pool)
            .await
            .inspect_err(|e| println!("Error fetching application: {}", e))
            .ok()
    }

    pub async fn list_applications(&self) -> Vec<Application> {
        let query = "
            SELECT id, name, version, hash_code, install_path
            FROM applications
        ";
        sqlx::query_as(query)
            .fetch_all(&self.db_pool)
            .await
            .inspect_err(|e| println!("Error listing applications: {e}"))
            .unwrap_or_default()
    }
}
