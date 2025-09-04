use clap::Parser;
use secret_online_patcher::{
    cli::{self, Args, Operation},
    service::app_manager::AppManager,
    storage::patcher_db::PatcherDatabase,
};
use sqlx::SqlitePool;
use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

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

    let app_manager = AppManager::new(patcher_db.clone());

    let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    match args.op {
        Operation::List => {
            // Call the function to list applications
            cli::list_apps(&patcher_db).await;
        }
        Operation::AddApp => {
            if args.app_name.is_none() || args.app_version.is_none() || args.app_path.is_none() {
                eprintln!(
                    "Error: --app-name, --app-version, and --app-path are required for add-app operation."
                );
                return;
            }
            // Call the function to add an app
            if let Err(e) = cli::add_app(
                args.app_name.as_ref().unwrap(),
                args.app_version.as_ref().unwrap(),
                args.app_path.as_ref().unwrap(),
                &app_manager,
            )
            .await
            {
                eprintln!("Error adding application: {}", e);
            }
        }
        Operation::RemoveApp => {
            if args.app_name.is_none() {
                eprintln!("Error: --app-name is required for remove-app operation.");
                return;
            }

            cli::remove_app(args.app_name.as_ref().unwrap(), &patcher_db).await;
        }
        Operation::Check => {
            if args.app_name.is_none() {
                eprintln!("Error: --app-name is required for check operation.");
                return;
            }
            // Call the function to check an app
            if let Err(e) = cli::check_app(args.app_name.as_ref().unwrap(), &patcher_db).await {
                eprintln!("Error checking application: {}", e);
            }
        }
        Operation::Update => {
            if args.app_name.is_none() || args.app_version.is_none() {
                eprintln!("Error: --app-name and --app-version are required for update operation.");
                return;
            }

            let app_name = args.app_name.as_ref().unwrap();
            let new_version = args.app_version.as_ref().unwrap();
            if let Err(e) = cli::update_app(app_name, new_version, &patcher_db).await {
                eprintln!("Error updating application: {}", e);
            }
        }
    }
    let end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    println!("Operation took {} milliseconds", (end - start).as_millis());
}
