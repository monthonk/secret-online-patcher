use clap::Parser;
use secret_online_patcher::{
    cli::{Args, Operation},
    storage::{application_data::Application, patcher_db::PatcherDatabase},
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
            if args.app_name.is_none() || args.app_version.is_none() || args.app_path.is_none() {
                eprintln!("Error: --app-name, --app-version, and --app-path are required for add-app operation.");
                return;
            }
            // Call the function to add an app
            let app = Application {
                id: 0, // ID will be auto-generated
                name: args.app_name.clone().unwrap(),
                version: args.app_version.clone().unwrap(),
                hash_code: args.app_path.clone().unwrap(),
            };
            secret_online_patcher::cli::add_app(app, &patcher_db).await;
        }
    }
}
