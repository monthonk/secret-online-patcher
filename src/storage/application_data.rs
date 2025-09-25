use std::path::PathBuf;

use sqlx::{FromRow, Row, sqlite::SqliteRow};

/// Store application information
#[derive(Clone)]
pub struct Application {
    pub id: i64,
    pub name: String,
    pub version: String,
    pub hash_code: Option<String>,
    pub install_path: PathBuf,
}

impl FromRow<'_, SqliteRow> for Application {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Application {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            version: row.try_get("version")?,
            hash_code: row.try_get("hash_code").ok(),
            install_path: PathBuf::from(row.try_get::<String, _>("install_path")?),
        })
    }
}
