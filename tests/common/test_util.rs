use std::{fs, path::Path};

use secret_online_patcher::storage::{application_data::Application, patcher_db::PatcherDatabase};
use sqlx::SqlitePool;

pub fn initialize_test_dir(test_name: &str) -> String {
    let test_dir = format!("fs_tests/{}", test_name);

    // Clean up any existing test directory
    if Path::new(&test_dir).exists() {
        fs::remove_dir_all(&test_dir).unwrap();
    }

    // Create test directory
    fs::create_dir_all(&test_dir).unwrap();
    test_dir
}

pub async fn initialize_test_db(db_pool: &SqlitePool) -> PatcherDatabase {
    let db = PatcherDatabase::new(db_pool.clone());
    db.initialize().await;
    db
}

pub async fn initialize_test_app(app_path: &str, db: &PatcherDatabase) -> Application {
    // Initialise application in the database
    db.add_application("Test App", "0.0.1", Path::new(app_path))
        .await
        .unwrap()
}
