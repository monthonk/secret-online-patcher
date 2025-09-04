use std::path::PathBuf;

use crate::{
    indexer::dir_hasher::DirHasher,
    storage::{application_data::Application, patcher_db::PatcherDatabase},
};

pub struct AppManager {
    db: PatcherDatabase,
}

impl AppManager {
    pub fn new(database: PatcherDatabase) -> Self {
        AppManager { db: database }
    }

    pub async fn create_application(
        &self,
        name: &str,
        version: &str,
        path: &PathBuf,
    ) -> Result<Application, anyhow::Error> {
        // Compute hash code for the app
        // Hash code is the CRC32 hash of the hash from all files in the app directory
        // order by their names.
        let hasher = DirHasher::new(None, self.db.clone());
        let app_hash = hasher.dir_hash(path).await?;
        println!("Application hash is {}", app_hash);

        // Implementation for adding an app
        let app = self
            .db
            .add_application(name, version, &app_hash, path)
            .await?;
        Ok(app)
    }
}
