use std::path::PathBuf;

use crate::{
    indexer::{dir_hasher::DirHasher, indexer_config::IndexerConfig},
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
        // Add new app to db
        let app = self.db.add_application(name, version, path).await?;

        // Compute hash code for the app
        // Hash code is the SHA256 hash of the hash from all files in the app directory
        // order by their names.
        let indexer_config = IndexerConfig::new(app.id, self.db.clone(), true);
        let hasher = DirHasher::new(indexer_config);
        let app_hash = hasher.dir_hash(path).await?;
        println!("Application hash is {}", app_hash);

        // Update the application with the computed hash
        self.db
            .update_application(&app.id, version, &app_hash)
            .await;

        Ok(app)
    }
}
