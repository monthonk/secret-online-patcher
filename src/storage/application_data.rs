use sqlx::{FromRow, Row, sqlite::SqliteRow};

// Store application information
pub struct Application {
    pub id: i64,
    pub name: String,
    pub version: String,
    pub hash_code: String,
}

impl FromRow<'_, SqliteRow> for Application {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Application {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            version: row.try_get("version")?,
            hash_code: row.try_get("hash_code")?,
        })
    }
}
