use chrono::NaiveDateTime;
use sqlx::{FromRow, Row, sqlite::SqliteRow};

pub struct FileIndex {
    pub app_id: i64,
    pub file_path: String,
    pub file_type: String,
    pub hash_code: Option<String>,
    // Modified time should be stored in UTC
    pub modified_time: NaiveDateTime,
}

impl FromRow<'_, SqliteRow> for FileIndex {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(FileIndex {
            app_id: row.try_get("app_id")?,
            file_path: row.try_get("file_path")?,
            file_type: row.try_get("file_type")?,
            hash_code: row.try_get("hash_code").ok(),
            modified_time: row.try_get("modified_time")?,
        })
    }
}
