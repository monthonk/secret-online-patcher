use chrono::NaiveDateTime;
use sqlx::{FromRow, Row, sqlite::SqliteRow};

pub struct PatchInfo {
    pub id: i64,
    pub app_name: String,
    pub base_version: String,
    pub patch_version: String,
    pub created_at: NaiveDateTime,
}

impl FromRow<'_, SqliteRow> for PatchInfo {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(PatchInfo {
            id: row.try_get("id")?,
            app_name: row.try_get("app_name")?,
            base_version: row.try_get("base_version")?,
            patch_version: row.try_get("patch_version")?,
            created_at: row.try_get("created_at")?,
        })
    }
}
