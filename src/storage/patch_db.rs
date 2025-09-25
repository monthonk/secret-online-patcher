use sqlx::{Executor, SqlitePool};

use crate::storage::patch_info::PatchInfo;

/// Database containing information about created patches.
/// This database should be attached to the zip file for the patch.
pub struct PatchDatabase {
    db_pool: SqlitePool,
}

impl PatchDatabase {
    pub fn new(db_pool: SqlitePool) -> Self {
        PatchDatabase { db_pool }
    }

    /// Initialize tables in the database
    pub async fn initialize(&self) {
        let patch_info_table = "
            CREATE TABLE IF NOT EXISTS patch_info (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                app_name TEXT NOT NULL,
                base_version TEXT NOT NULL,
                patch_version TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
        ";
        self.db_pool.execute(patch_info_table).await.unwrap();

        let file_changes_table = "
            CREATE TABLE IF NOT EXISTS file_changes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                patch_id INTEGER NOT NULL,
                file_path TEXT NOT NULL,
                file_type TEXT CHECK( file_type IN ('FILE','DIRECTORY') ) NOT NULL,
                change_type TEXT CHECK( change_type IN ('CREATED','MODIFIED','DELETED') ) NOT NULL,
                FOREIGN KEY (patch_id) REFERENCES patch_info (id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS ix_patch_file_path ON file_changes (file_path);
        ";
        self.db_pool.execute(file_changes_table).await.unwrap();
    }

    pub async fn create_patch(
        &self,
        app_name: &str,
        base_version: &str,
        patch_version: &str,
    ) -> Result<PatchInfo, sqlx::Error> {
        let query = "
            INSERT INTO patch_info (app_name, base_version, patch_version)
            VALUES (?, ?, ?)
            RETURNING *
        ";
        sqlx::query_as(query)
            .bind(app_name)
            .bind(base_version)
            .bind(patch_version)
            .fetch_one(&self.db_pool)
            .await
    }

    pub async fn add_file_change(
        &self,
        patch_id: i64,
        file_path: &str,
        file_type: &str,
        change_type: &str,
    ) -> Result<bool, sqlx::Error> {
        let query = "
            INSERT INTO file_changes (patch_id, file_path, file_type, change_type)
            VALUES (?, ?, ?, ?)
        ";
        sqlx::query(query)
            .bind(patch_id)
            .bind(file_path)
            .bind(file_type)
            .bind(change_type)
            .execute(&self.db_pool)
            .await
            .map(|result| result.rows_affected() == 1)
    }
}
