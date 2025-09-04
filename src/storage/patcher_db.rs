use std::path::Path;

use chrono::NaiveDateTime;
use sqlx::{Executor, SqlitePool};

use crate::storage::{application_data::Application, file_index::FileIndex};

#[derive(Clone)]
pub struct PatcherDatabase {
    db_pool: SqlitePool,
}

impl PatcherDatabase {
    pub fn new(db_pool: SqlitePool) -> Self {
        PatcherDatabase { db_pool }
    }

    /// Initialize tables in the database
    pub async fn initialize(&self) {
        let application_table = "
            CREATE TABLE IF NOT EXISTS applications (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                hash_code TEXT,
                install_path TEXT NOT NULL
            );

            CREATE UNIQUE INDEX IF NOT EXISTS ux_app_name ON applications (name);
        ";
        self.db_pool.execute(application_table).await.unwrap();

        let file_index_table = "
            CREATE TABLE IF NOT EXISTS file_index (
                app_id INTEGER NOT NULL,
                file_path TEXT NOT NULL,
                file_type TEXT CHECK( file_type IN ('FILE','DIRECTORY') ) NOT NULL,
                hash_code TEXT NOT NULL,
                modified_time TIMESTAMP,
                PRIMARY KEY (app_id, file_path),
                FOREIGN KEY (app_id) REFERENCES applications (id) ON DELETE CASCADE
            );
        ";
        self.db_pool.execute(file_index_table).await.unwrap();
    }

    pub async fn add_application(
        &self,
        name: &str,
        version: &str,
        install_path: &Path,
    ) -> Result<Application, sqlx::Error> {
        let install_path = install_path.to_string_lossy();
        let query = "
            INSERT INTO applications (name, version, install_path)
            VALUES (?, ?, ?)
            RETURNING *
        ";
        sqlx::query_as(query)
            .bind(name)
            .bind(version)
            .bind(install_path.as_ref())
            .fetch_one(&self.db_pool)
            .await
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

    pub async fn get_application(&self, name: &str) -> Result<Option<Application>, sqlx::Error> {
        let query = "
            SELECT id, name, version, hash_code, install_path
            FROM applications
            WHERE name = ?;
        ";
        sqlx::query_as(query)
            .bind(name)
            .fetch_optional(&self.db_pool)
            .await
            .inspect_err(|e| println!("Error fetching application: {}", e))
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

    pub async fn upsert_file_index(
        &self,
        app_id: i64,
        file_path: &str,
        file_type: &str,
        hash_code: &str,
        modified_time: &NaiveDateTime,
    ) -> Result<FileIndex, sqlx::Error> {
        let query = "
            INSERT INTO file_index (app_id, file_path, file_type, hash_code, modified_time)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (app_id, file_path) DO UPDATE
            SET file_type = $3, hash_code = $4, modified_time = $5
            RETURNING *;
        ";
        sqlx::query_as(query)
            .bind(app_id)
            .bind(file_path)
            .bind(file_type)
            .bind(hash_code)
            .bind(modified_time)
            .fetch_one(&self.db_pool)
            .await
    }

    pub async fn get_file_index(
        &self,
        app_id: i64,
        file_path: &str,
    ) -> Result<Option<FileIndex>, sqlx::Error> {
        let query = "
            SELECT app_id, file_path, file_type, hash_code, modified_time
            FROM file_index
            WHERE app_id = ? AND file_path = ?;
        ";
        sqlx::query_as(query)
            .bind(app_id)
            .bind(file_path)
            .fetch_optional(&self.db_pool)
            .await
            .inspect_err(|e| println!("Error fetching file index: {}", e))
    }
}
