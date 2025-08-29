use clap::Parser;
use secret_online_patcher::{
    cli::{Args, Operation},
    storage::patcher_db::PatcherDatabase,
};
use sqlx::SqlitePool;
use std::path::Path;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Ensure resources directory exists
    let resources_dir = Path::new("resources");
    if !resources_dir.exists() {
        std::fs::create_dir_all(resources_dir).unwrap();
    }

    // Initialize database connection with file-based storage
    // Use create flag to ensure database file is created if it doesn't exist
    let db_path = "sqlite:resources/app_data.db?mode=rwc";
    let db_pool = SqlitePool::connect(db_path).await.unwrap();
    let patcher_db = PatcherDatabase::new(db_pool);
    patcher_db.initialize().await;

    match args.op {
        Operation::List => {
            // Call the function to list applications
            secret_online_patcher::cli::list_apps(&patcher_db).await;
        }
        Operation::AddApp => {
            // Call the function to add an app
            secret_online_patcher::cli::add_app();
        }
    }
}
